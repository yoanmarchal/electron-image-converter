// Conversion et redimensionnement d'images, sans dépendance à l'interface (testable avec cargo test)
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, BufReader, Cursor, Write};
use std::path::{Path, PathBuf};

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::codecs::gif::GifDecoder;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::metadata::LoopCount;
use image::{
    AnimationDecoder, DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageError,
    ImageFormat, ImageReader, Limits, RgbImage, RgbaImage,
};
use moxcms::{ColorProfile, Layout, TransformOptions};
use thiserror::Error;

use crate::animated_webp::AnimatedWebpEncoder;

// L'AVIF n'est pas accepté en entrée : il est déjà optimisé, et son décodage imposerait dav1d
pub const INPUT_EXTENSIONS: [&str; 7] = ["jpg", "jpeg", "png", "gif", "tif", "tiff", "webp"];

// Côté du carré affiché dans la liste (48 px), doublé pour les écrans haute densité
pub const THUMBNAIL_SIZE: u32 = 96;

pub const MAX_RESIZE_PIXELS: u32 = 16_384;
pub const MAX_RESIZE_PERCENT: u32 = 400;

// Un agrandissement au-delà de 512 Mio de pixels RGBA est refusé plutôt que tenté : c'est aussi
// la limite appliquée au décodage
const MAX_OUTPUT_PIXELS: u64 = 512 * 1024 * 1024 / 4;

const MIN_FRAME_DELAY_MS: u32 = 10;
const DEFAULT_FRAME_DELAY_MS: u32 = 100;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Le chemin du fichier source est invalide ou manquant")]
    InvalidPath,
    #[error("Extension non supportée : {0}")]
    UnsupportedExtension(String),
    #[error("Le fichier source n'existe pas : {}", .0.display())]
    SourceNotFound(PathBuf),
    #[error("Le chemin ne désigne pas un fichier : {}", .0.display())]
    NotAFile(PathBuf),
    #[error("La qualité doit être un nombre entre 1 et 100")]
    InvalidQuality,
    #[error("La taille demandée doit être comprise entre 1 et {max} {unit}")]
    InvalidResize { max: u32, unit: &'static str },
    #[error(
        "L'image ferait {width} × {height} px : le format {format} est limité à {max} px de côté"
    )]
    TooLarge {
        width: u32,
        height: u32,
        format: &'static str,
        max: u32,
    },
    #[error(
        "L'image ferait {width} × {height} px : c'est trop pour être traité (maximum {max} mégapixels)"
    )]
    TooManyPixels { width: u32, height: u32, max: u64 },
    #[error("Le fichier de sortie est vide")]
    EmptyOutput,
    #[error("Image illisible : {0}")]
    Decode(#[from] ImageError),
    #[error("Échec du redimensionnement : {0}")]
    Resize(String),
    #[error("Échec de l'encodage : {0}")]
    Encode(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Webp,
    Jpg,
    Png,
    Avif,
}

impl OutputFormat {
    pub const ALL: [OutputFormat; 4] = [Self::Webp, Self::Jpg, Self::Png, Self::Avif];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Avif => "avif",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Webp => "WebP",
            Self::Jpg => "JPEG",
            Self::Png => "PNG",
            Self::Avif => "AVIF",
        }
    }

    /// Plus grand côté que le format sait stocker.
    fn max_dimension(self) -> u32 {
        match self {
            Self::Webp => 16_383,
            Self::Jpg | Self::Avif => 65_535,
            Self::Png => u32::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatChoice {
    /// Garde le format du fichier source (utile pour seulement redimensionner).
    Original,
    Fixed(OutputFormat),
}

impl FormatChoice {
    pub fn resolve(self, path: &Path) -> OutputFormat {
        match self {
            Self::Fixed(format) => format,
            Self::Original => match extension(path).as_str() {
                "jpg" | "jpeg" => OutputFormat::Jpg,
                "webp" => OutputFormat::Webp,
                // Le GIF et le TIFF ne sont pas des formats de sortie : repli sans perte
                _ => OutputFormat::Png,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeMode {
    Width,
    Height,
    LongestSide,
    Percent,
}

/// Redimensionnement à ratio conservé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resize {
    pub mode: ResizeMode,
    /// Pixels, ou pourcentage en mode `Percent`.
    pub value: u32,
    /// Sans cette option, une image plus petite que la cible garde sa taille.
    pub allow_upscale: bool,
}

impl Resize {
    pub fn validate(&self) -> Result<(), ConversionError> {
        let (max, unit) = match self.mode {
            ResizeMode::Percent => (MAX_RESIZE_PERCENT, "%"),
            _ => (MAX_RESIZE_PIXELS, "px"),
        };
        if (1..=max).contains(&self.value) {
            Ok(())
        } else {
            Err(ConversionError::InvalidResize { max, unit })
        }
    }
}

/// Dimensions de sortie d'une image de `width` × `height` pixels (orientation EXIF déjà appliquée).
pub fn target_dimensions(width: u32, height: u32, resize: &Resize) -> (u32, u32) {
    let value = f64::from(resize.value);
    let scale = match resize.mode {
        ResizeMode::Width => value / f64::from(width),
        ResizeMode::Height => value / f64::from(height),
        ResizeMode::LongestSide => value / f64::from(width.max(height)),
        ResizeMode::Percent => value / 100.0,
    };
    let scale = if resize.allow_upscale {
        scale
    } else {
        scale.min(1.0)
    };
    let scaled = |side: u32| ((f64::from(side) * scale).round() as u32).max(1);
    (scaled(width), scaled(height))
}

#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// Sans dossier de sortie, le fichier est écrit à côté de l'original.
    pub output_dir: Option<PathBuf>,
    /// De 1 à 100 ; ignorée en PNG, qui est sans perte.
    pub quality: u8,
    pub format: FormatChoice,
    pub resize: Option<Resize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionResult {
    pub output_path: PathBuf,
    pub original_size: u64,
    pub new_size: u64,
    /// Gain en pourcentage ; négatif si le fichier converti est plus lourd.
    pub compression_ratio: f64,
    pub original_dimensions: (u32, u32),
    pub new_dimensions: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub thumbnail: RgbaImage,
}

pub fn extension(path: &Path) -> String {
    path.extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

pub fn assert_image_file(path: &Path) -> Result<Metadata, ConversionError> {
    if !path.is_absolute() {
        return Err(ConversionError::InvalidPath);
    }
    let extension = extension(path);
    if !INPUT_EXTENSIONS.contains(&extension.as_str()) {
        return Err(ConversionError::UnsupportedExtension(
            if extension.is_empty() {
                "(aucune)".to_owned()
            } else {
                format!(".{extension}")
            },
        ));
    }
    let metadata = fs::metadata(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => ConversionError::SourceNotFound(path.to_owned()),
        _ => ConversionError::Io(error),
    })?;
    if !metadata.is_file() {
        return Err(ConversionError::NotAFile(path.to_owned()));
    }
    Ok(metadata)
}

/// Décode l'image et applique l'orientation EXIF aux pixels : les métadonnées ne sont pas
/// copiées, les photos prises en portrait ressortiraient couchées.
fn load_oriented(path: &Path) -> Result<RgbaImage, ConversionError> {
    let mut decoder = ImageReader::open(path)?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    // Un profil illisible ne doit pas empêcher la conversion
    let icc_profile = decoder.icc_profile().ok().flatten();
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);

    let mut image = image.into_rgba8();
    if let Some(icc_profile) = icc_profile {
        convert_to_srgb(&mut image, &icc_profile);
    }
    Ok(image)
}

/// Le profil de couleur n'est pas copié dans le fichier de sortie : les pixels d'une photo
/// Display P3 ou Adobe RGB sont ramenés en sRGB, sinon ses couleurs ressortiraient ternes.
/// Les profils non RVB (niveaux de gris, CMJN) ou invalides sont ignorés.
fn convert_to_srgb(image: &mut RgbaImage, icc_profile: &[u8]) {
    let Ok(source) = ColorProfile::new_from_slice(icc_profile) else {
        return;
    };
    let Ok(transform) = source.create_transform_8bit(
        Layout::Rgba,
        &ColorProfile::new_srgb(),
        Layout::Rgba,
        TransformOptions::default(),
    ) else {
        return;
    };
    let mut converted = vec![0; image.as_raw().len()];
    if transform.transform(image.as_raw(), &mut converted).is_ok() {
        image.copy_from_slice(&converted);
    }
}

fn resize_with(
    image: &RgbaImage,
    width: u32,
    height: u32,
    options: &ResizeOptions,
) -> Result<RgbaImage, ConversionError> {
    let resize_error = |error: &dyn std::fmt::Display| ConversionError::Resize(error.to_string());

    let source = ImageRef::new(
        image.width(),
        image.height(),
        image.as_raw(),
        PixelType::U8x4,
    )
    .map_err(|error| resize_error(&error))?;
    let mut target = Image::new(width, height, PixelType::U8x4);
    // L'alpha est prémultiplié pendant le calcul : pas de frange sombre autour des zones transparentes
    Resizer::new()
        .resize(&source, &mut target, options)
        .map_err(|error| resize_error(&error))?;

    RgbaImage::from_raw(width, height, target.into_vec())
        .ok_or_else(|| resize_error(&"tampon de sortie invalide"))
}

fn lanczos() -> ResizeOptions {
    ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3))
}

/// Dimensions de sortie, refusées avant toute allocation si le format ou la mémoire ne
/// peuvent pas les accueillir.
fn planned_dimensions(
    original: (u32, u32),
    resize: Option<&Resize>,
    format: OutputFormat,
) -> Result<(u32, u32), ConversionError> {
    let (width, height) = resize.map_or(original, |resize| {
        target_dimensions(original.0, original.1, resize)
    });
    assert_fits(width, height, format)?;
    if (width, height) != original && u64::from(width) * u64::from(height) > MAX_OUTPUT_PIXELS {
        return Err(ConversionError::TooManyPixels {
            width,
            height,
            max: MAX_OUTPUT_PIXELS / 1_000_000,
        });
    }
    Ok((width, height))
}

fn resize_to(image: RgbaImage, (width, height): (u32, u32)) -> Result<RgbaImage, ConversionError> {
    if (width, height) == image.dimensions() {
        return Ok(image);
    }
    resize_with(&image, width, height, &lanczos())
}

/// Miniature carrée, recadrée au centre.
fn create_thumbnail(image: &RgbaImage) -> Result<RgbaImage, ConversionError> {
    let options = lanczos().fit_into_destination(Some((0.5, 0.5)));
    resize_with(image, THUMBNAIL_SIZE, THUMBNAIL_SIZE, &options)
}

/// Dimensions (orientation EXIF appliquée), poids et miniature d'une image à ajouter à la liste.
pub fn image_info(path: &Path) -> Result<ImageInfo, ConversionError> {
    let metadata = assert_image_file(path)?;
    let image = load_oriented(path)?;
    Ok(ImageInfo {
        width: image.width(),
        height: image.height(),
        size: metadata.len(),
        thumbnail: create_thumbnail(&image)?,
    })
}

// Le JPEG n'a pas de transparence : fond blanc plutôt que noir
fn flatten_on_white(image: &RgbaImage) -> RgbImage {
    RgbImage::from_fn(image.width(), image.height(), |x, y| {
        let [r, g, b, a] = image.get_pixel(x, y).0;
        let alpha = u32::from(a);
        let blend =
            |channel: u8| ((u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
        image::Rgb([blend(r), blend(g), blend(b)])
    })
}

fn is_opaque(image: &RgbaImage) -> bool {
    image.pixels().all(|pixel| pixel[3] == u8::MAX)
}

fn assert_fits(width: u32, height: u32, format: OutputFormat) -> Result<(), ConversionError> {
    let max = format.max_dimension();
    if width > max || height > max {
        return Err(ConversionError::TooLarge {
            width,
            height,
            format: format.name(),
            max,
        });
    }
    Ok(())
}

fn encode(
    image: &RgbaImage,
    format: OutputFormat,
    quality: u8,
) -> Result<Vec<u8>, ConversionError> {
    let (width, height) = image.dimensions();
    let encode_error = |error: &dyn std::fmt::Display| ConversionError::Encode(error.to_string());

    match format {
        OutputFormat::Webp => {
            let encoder = webp::Encoder::from_rgba(image, width, height);
            Ok(encoder.encode(f32::from(quality)).to_vec())
        }
        OutputFormat::Jpg => {
            // Sous-échantillonnage 4:2:0 et tables de Huffman optimisées : fichiers environ un tiers plus légers
            let mut bytes = Vec::new();
            let mut encoder = jpeg_encoder::Encoder::new(&mut bytes, quality);
            encoder.set_sampling_factor(jpeg_encoder::SamplingFactor::F_2_2);
            encoder.set_optimized_huffman_tables(true);
            encoder
                .encode(
                    &flatten_on_white(image),
                    width as u16,
                    height as u16,
                    jpeg_encoder::ColorType::Rgb,
                )
                .map_err(|error| encode_error(&error))?;
            Ok(bytes)
        }
        OutputFormat::Png => {
            // PNG sans perte : la qualité ne s'applique pas. Le niveau maximal (Best) est
            // deux fois plus lent pour un gain de taille de 1 à 2 %
            let mut bytes = Cursor::new(Vec::new());
            let encoder = PngEncoder::new_with_quality(
                &mut bytes,
                CompressionType::Default,
                PngFilter::Adaptive,
            );
            if is_opaque(image) {
                let rgb = DynamicImage::ImageRgba8(image.clone()).into_rgb8();
                encoder.write_image(&rgb, width, height, ExtendedColorType::Rgb8)
            } else {
                encoder.write_image(image, width, height, ExtendedColorType::Rgba8)
            }
            .map_err(|error| encode_error(&error))?;
            Ok(bytes.into_inner())
        }
        OutputFormat::Avif => {
            let pixels: Vec<ravif::RGBA8> = image
                .pixels()
                .map(|pixel| ravif::RGBA8::new(pixel[0], pixel[1], pixel[2], pixel[3]))
                .collect();
            let encoded = ravif::Encoder::new()
                .with_quality(f32::from(quality))
                .with_speed(6)
                .encode_rgba(ravif::Img::new(&pixels, width as usize, height as usize))
                .map_err(|error| encode_error(&error))?;
            Ok(encoded.avif_file)
        }
    }
}

struct Encoded {
    bytes: Vec<u8>,
    original_dimensions: (u32, u32),
    new_dimensions: (u32, u32),
}

fn encode_still(
    image: RgbaImage,
    format: OutputFormat,
    options: &ConvertOptions,
) -> Result<Encoded, ConversionError> {
    let original_dimensions = image.dimensions();
    let target = planned_dimensions(original_dimensions, options.resize.as_ref(), format)?;
    let image = resize_to(image, target)?;
    Ok(Encoded {
        bytes: encode(&image, format, options.quality)?,
        original_dimensions,
        new_dimensions: image.dimensions(),
    })
}

/// Convertit un GIF en WebP animé, une image à la fois pour borner la mémoire. Un GIF d'une
/// seule image donne un WebP fixe.
fn encode_animated_webp(
    mut decoder: GifDecoder<BufReader<File>>,
    options: &ConvertOptions,
) -> Result<Encoded, ConversionError> {
    let encode_error = ConversionError::Encode;
    decoder.set_limits(Limits::default())?;

    let original_dimensions = decoder.dimensions();
    let (width, height) = planned_dimensions(
        original_dimensions,
        options.resize.as_ref(),
        OutputFormat::Webp,
    )?;
    let loop_count = match decoder.loop_count() {
        LoopCount::Infinite => 0,
        LoopCount::Finite(count) => i32::try_from(count.get()).unwrap_or(i32::MAX),
    };
    let mut encoder =
        AnimatedWebpEncoder::new(width, height, f32::from(options.quality), loop_count)
            .map_err(encode_error)?;

    // Instant d'affichage de chaque image, en millisecondes depuis le début
    let mut timestamp: i32 = 0;
    let mut frame_count = 0;
    for frame in decoder.into_frames() {
        let frame = frame?;
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let delay = numerator / denominator.max(1);
        // Comme les navigateurs : un délai nul ou quasi nul vaut un dixième de seconde
        let delay = if delay <= MIN_FRAME_DELAY_MS {
            DEFAULT_FRAME_DELAY_MS
        } else {
            delay
        };

        let frame = resize_to(frame.into_buffer(), (width, height))?;
        encoder
            .add_frame(frame.as_raw(), timestamp)
            .map_err(encode_error)?;
        timestamp = timestamp.saturating_add(i32::try_from(delay).unwrap_or(i32::MAX));
        frame_count += 1;
    }
    if frame_count == 0 {
        return Err(ConversionError::EmptyOutput);
    }

    Ok(Encoded {
        bytes: encoder.finish(timestamp).map_err(encode_error)?,
        original_dimensions,
        new_dimensions: (width, height),
    })
}

/// Ne jamais écraser un fichier existant (ni l'original) : photo.webp, puis photo-1.webp,
/// photo-2.webp… La création est atomique : deux conversions simultanées ne peuvent pas
/// choisir le même nom.
pub fn write_new_file(
    dir: &Path,
    name: &str,
    extension: &str,
    bytes: &[u8],
) -> Result<PathBuf, ConversionError> {
    for index in 0u32.. {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("-{index}")
        };
        let candidate = dir.join(format!("{name}{suffix}.{extension}"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(bytes) {
                    drop(file);
                    // Ne pas laisser un fichier tronqué derrière soi
                    let _ = fs::remove_file(&candidate);
                    return Err(error.into());
                }
                return Ok(candidate);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!("la boucle ne se termine que par un retour")
}

/// Convertit une image, en la redimensionnant si les options le demandent.
pub fn convert_image(
    path: &Path,
    options: &ConvertOptions,
) -> Result<ConversionResult, ConversionError> {
    if !(1..=100).contains(&options.quality) {
        return Err(ConversionError::InvalidQuality);
    }
    if let Some(resize) = &options.resize {
        resize.validate()?;
    }
    let original_size = assert_image_file(path)?.len();
    let format = options.format.resolve(path);

    // Seul le WebP conserve l'animation d'un GIF ; les autres formats gardent la première image.
    // Le GIF est reconnu à son contenu, comme pour la miniature, pas à son extension
    let reader = ImageReader::open(path)?.with_guessed_format()?;
    let encoded = if format == OutputFormat::Webp && reader.format() == Some(ImageFormat::Gif) {
        encode_animated_webp(GifDecoder::new(reader.into_inner())?, options)?
    } else {
        encode_still(load_oriented(path)?, format, options)?
    };
    if encoded.bytes.is_empty() {
        return Err(ConversionError::EmptyOutput);
    }

    let output_dir = match &options.output_dir {
        Some(dir) => dir.as_path(),
        None => path.parent().ok_or(ConversionError::InvalidPath)?,
    };
    let name = path
        .file_stem()
        .ok_or(ConversionError::InvalidPath)?
        .to_string_lossy();
    let output_path = write_new_file(output_dir, &name, format.extension(), &encoded.bytes)?;

    let new_size = encoded.bytes.len() as u64;
    Ok(ConversionResult {
        output_path,
        original_size,
        new_size,
        compression_ratio: (1.0 - new_size as f64 / original_size as f64) * 100.0,
        original_dimensions: encoded.original_dimensions,
        new_dimensions: encoded.new_dimensions,
    })
}
