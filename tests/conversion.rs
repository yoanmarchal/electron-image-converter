use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use image::codecs::gif::{GifEncoder, Repeat};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPDecoder;
use image::{
    AnimationDecoder, Delay, Frame, ImageEncoder, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage,
};
use image_converter::conversion::{
    ConversionError, ConvertOptions, FormatChoice, OutputFormat, Resize, ResizeMode,
    THUMBNAIL_SIZE, assert_image_file, convert_image, image_info, target_dimensions,
    write_new_file,
};
use tempfile::TempDir;

const BLUE: Rgb<u8> = Rgb([0x33, 0x66, 0xcc]);

fn temp_dir() -> TempDir {
    tempfile::Builder::new()
        .prefix("image-converter-test-")
        .tempdir()
        .unwrap()
}

fn create_image(dir: &Path, name: &str, width: u32, height: u32) -> PathBuf {
    let path = dir.join(name);
    RgbImage::from_pixel(width, height, BLUE)
        .save(&path)
        .unwrap();
    path
}

fn create_transparent_image(dir: &Path, name: &str, width: u32, height: u32) -> PathBuf {
    let path = dir.join(name);
    RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]))
        .save(&path)
        .unwrap();
    path
}

// JPEG de 60 × 40 dont l'EXIF demande une rotation de 90° (orientation 6)
fn create_portrait_jpeg(dir: &Path) -> PathBuf {
    #[rustfmt::skip]
    let exif = vec![
        0x49, 0x49, 0x2a, 0, 8, 0, 0, 0, // en-tête TIFF petit-boutiste, répertoire à l'octet 8
        1, 0, // une entrée
        0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, // Orientation (0x0112), SHORT, valeur 6
        0, 0, 0, 0, // pas de répertoire suivant
    ];
    let path = dir.join("portrait.jpg");
    let mut encoder = JpegEncoder::new(File::create(&path).unwrap());
    encoder.set_exif_metadata(exif).unwrap();
    encoder
        .write_image(
            &RgbImage::from_pixel(60, 40, BLUE),
            60,
            40,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();
    path
}

fn create_animated_gif(dir: &Path) -> PathBuf {
    create_gif(dir, "anim.gif", &[100, 100, 100])
}

// GIF de 16 × 16 avec une image par délai (en millisecondes), chacune d'une couleur unie
fn create_gif(dir: &Path, name: &str, delays_ms: &[u32]) -> PathBuf {
    const COLORS: [[u8; 4]; 3] = [[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]];
    let path = dir.join(name);
    let frames = delays_ms.iter().enumerate().map(|(index, delay)| {
        Frame::from_parts(
            RgbaImage::from_pixel(16, 16, Rgba(COLORS[index % COLORS.len()])),
            0,
            0,
            Delay::from_numer_denom_ms(*delay, 1),
        )
    });
    let mut encoder = GifEncoder::new(File::create(&path).unwrap());
    encoder.set_repeat(Repeat::Infinite).unwrap();
    encoder.encode_frames(frames).unwrap();
    path
}

// PNG d'une couleur unie, étiqueté avec le profil de couleur donné
fn create_profiled_png(dir: &Path, name: &str, color: Rgba<u8>, icc_profile: Vec<u8>) -> PathBuf {
    let path = dir.join(name);
    let mut encoder = PngEncoder::new(File::create(&path).unwrap());
    encoder.set_icc_profile(icc_profile).unwrap();
    encoder
        .write_image(
            &RgbaImage::from_pixel(8, 8, color),
            8,
            8,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    path
}

fn options(format: OutputFormat) -> ConvertOptions {
    ConvertOptions {
        output_dir: None,
        quality: 80,
        format: FormatChoice::Fixed(format),
        resize: None,
    }
}

fn resized(format: OutputFormat, mode: ResizeMode, value: u32) -> ConvertOptions {
    ConvertOptions {
        resize: Some(Resize {
            mode,
            value,
            allow_upscale: false,
        }),
        ..options(format)
    }
}

fn dimensions(path: &Path) -> (u32, u32) {
    image::image_dimensions(path).unwrap()
}

fn webp_frame_delays_ms(path: &Path) -> Vec<u32> {
    WebPDecoder::new(BufReader::new(File::open(path).unwrap()))
        .unwrap()
        .into_frames()
        .collect_frames()
        .unwrap()
        .iter()
        .map(|frame| {
            let (numerator, denominator) = frame.delay().numer_denom_ms();
            numerator / denominator
        })
        .collect()
}

fn webp_frame_count(path: &Path) -> usize {
    webp_frame_delays_ms(path).len()
}

fn file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

mod conversion {
    use super::*;

    #[test]
    fn converts_to_each_format_next_to_the_original() {
        for format in OutputFormat::ALL {
            let dir = temp_dir();
            let input = create_image(dir.path(), "photo.tiff", 20, 20);

            let result = convert_image(&input, &options(format)).unwrap();

            assert_eq!(
                result.output_path,
                dir.path().join(format!("photo.{}", format.extension()))
            );
            assert!(result.new_size > 0);
            assert_eq!(
                result.new_size,
                fs::metadata(&result.output_path).unwrap().len()
            );

            let bytes = fs::read(&result.output_path).unwrap();
            let expected = match format {
                OutputFormat::Webp => ImageFormat::WebP,
                OutputFormat::Jpg => ImageFormat::Jpeg,
                OutputFormat::Png => ImageFormat::Png,
                OutputFormat::Avif => ImageFormat::Avif,
            };
            assert_eq!(image::guess_format(&bytes).unwrap(), expected);
        }
    }

    #[test]
    fn writes_to_the_requested_output_dir() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "photo.png", 20, 20);
        let output_dir = dir.path().join("sortie");
        fs::create_dir(&output_dir).unwrap();

        let result = convert_image(
            &input,
            &ConvertOptions {
                output_dir: Some(output_dir.clone()),
                ..options(OutputFormat::Webp)
            },
        )
        .unwrap();

        assert_eq!(result.output_path, output_dir.join("photo.webp"));
    }

    #[test]
    fn applies_exif_orientation_to_pixels() {
        let dir = temp_dir();
        let input = create_portrait_jpeg(dir.path());

        let result = convert_image(&input, &options(OutputFormat::Webp)).unwrap();

        assert_eq!(dimensions(&result.output_path), (40, 60));
        assert_eq!(result.original_dimensions, (40, 60));
    }

    #[test]
    fn flattens_transparency_on_white_in_jpeg() {
        let dir = temp_dir();
        let input = create_transparent_image(dir.path(), "transparent.png", 20, 20);

        let result = convert_image(
            &input,
            &ConvertOptions {
                quality: 90,
                ..options(OutputFormat::Jpg)
            },
        )
        .unwrap();

        let pixel = image::open(&result.output_path).unwrap().to_rgb8()[(0, 0)];
        assert!(pixel.0.iter().all(|&value| value > 245), "{pixel:?}");
    }

    #[test]
    fn keeps_transparency_in_png() {
        let dir = temp_dir();
        let input = create_transparent_image(dir.path(), "transparent.png", 20, 20);

        let result = convert_image(&input, &options(OutputFormat::Png)).unwrap();

        assert_eq!(result.output_path, dir.path().join("transparent-1.png"));
        let image = image::open(&result.output_path).unwrap().to_rgba8();
        assert_eq!(image[(0, 0)][3], 0);
    }

    #[test]
    fn keeps_gif_animation_in_webp() {
        let dir = temp_dir();
        let input = create_animated_gif(dir.path());

        let result = convert_image(&input, &options(OutputFormat::Webp)).unwrap();

        assert_eq!(webp_frame_count(&result.output_path), 3);
    }

    #[test]
    fn keeps_first_gif_frame_in_other_formats() {
        let dir = temp_dir();
        let input = create_animated_gif(dir.path());

        let result = convert_image(&input, &options(OutputFormat::Png)).unwrap();

        let image = image::open(&result.output_path).unwrap().to_rgb8();
        assert_eq!(image.dimensions(), (16, 16));
        assert_eq!(image[(0, 0)], Rgb([255, 0, 0]));
    }

    #[test]
    fn never_overwrites_the_original_or_a_previous_output() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "photo.png", 20, 20);
        let original = fs::read(&input).unwrap();

        let first = convert_image(&input, &options(OutputFormat::Png)).unwrap();
        let second = convert_image(&input, &options(OutputFormat::Png)).unwrap();

        assert_eq!(first.output_path, dir.path().join("photo-1.png"));
        assert_eq!(second.output_path, dir.path().join("photo-2.png"));
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(
            file_names(dir.path()),
            ["photo-1.png", "photo-2.png", "photo.png"]
        );
    }

    #[test]
    fn rejects_out_of_range_quality() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "photo.png", 20, 20);

        for quality in [0, 101] {
            let result = convert_image(
                &input,
                &ConvertOptions {
                    quality,
                    ..options(OutputFormat::Webp)
                },
            );
            assert!(matches!(result, Err(ConversionError::InvalidQuality)));
        }
        assert_eq!(file_names(dir.path()), ["photo.png"]);
    }

    #[test]
    fn computes_compression_ratio() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "photo.tiff", 200, 200);

        let result = convert_image(&input, &options(OutputFormat::Webp)).unwrap();

        let expected = (1.0 - result.new_size as f64 / result.original_size as f64) * 100.0;
        assert_eq!(result.compression_ratio, expected);
        assert!(result.compression_ratio > 90.0);
    }
}

mod original_format {
    use super::*;

    fn original() -> ConvertOptions {
        ConvertOptions {
            format: FormatChoice::Original,
            ..options(OutputFormat::Webp)
        }
    }

    #[test]
    fn keeps_the_source_format() {
        let dir = temp_dir();
        for (name, expected) in [
            ("a.jpg", "a-1.jpg"),
            ("b.jpeg", "b.jpg"),
            ("c.png", "c-1.png"),
        ] {
            let input = create_image(dir.path(), name, 20, 20);

            let result = convert_image(&input, &original()).unwrap();

            assert_eq!(result.output_path, dir.path().join(expected));
        }
    }

    #[test]
    fn falls_back_to_png_for_gif_and_tiff() {
        let dir = temp_dir();
        let tiff = create_image(dir.path(), "photo.tiff", 20, 20);
        let gif = create_animated_gif(dir.path());

        assert_eq!(
            convert_image(&tiff, &original()).unwrap().output_path,
            dir.path().join("photo.png")
        );
        assert_eq!(
            convert_image(&gif, &original()).unwrap().output_path,
            dir.path().join("anim.png")
        );
    }
}

mod resizing {
    use super::*;

    fn resize(mode: ResizeMode, value: u32, allow_upscale: bool) -> Resize {
        Resize {
            mode,
            value,
            allow_upscale,
        }
    }

    #[test]
    fn computes_dimensions_for_each_mode() {
        for (source, mode, value, expected) in [
            ((4000, 3000), ResizeMode::Width, 1920, (1920, 1440)),
            ((4000, 3000), ResizeMode::Height, 1500, (2000, 1500)),
            ((4000, 3000), ResizeMode::LongestSide, 1000, (1000, 750)),
            ((3000, 4000), ResizeMode::LongestSide, 1000, (750, 1000)),
            ((4000, 3000), ResizeMode::Percent, 25, (1000, 750)),
        ] {
            assert_eq!(
                target_dimensions(source.0, source.1, &resize(mode, value, false)),
                expected,
                "{mode:?} {value}"
            );
        }
    }

    #[test]
    fn rounds_without_going_below_one_pixel() {
        let width = resize(ResizeMode::Width, 100, false);

        assert_eq!(target_dimensions(1000, 333, &width), (100, 33));
        assert_eq!(target_dimensions(4000, 10, &width), (100, 1));
    }

    #[test]
    fn upscales_only_when_allowed() {
        for (mode, value) in [(ResizeMode::Width, 1600), (ResizeMode::Percent, 200)] {
            assert_eq!(
                target_dimensions(800, 600, &resize(mode, value, false)),
                (800, 600)
            );
            assert_eq!(
                target_dimensions(800, 600, &resize(mode, value, true)),
                (1600, 1200)
            );
        }
    }

    #[test]
    fn resizes_in_each_format() {
        for format in [OutputFormat::Webp, OutputFormat::Jpg, OutputFormat::Png] {
            let dir = temp_dir();
            let input = create_image(dir.path(), "photo.tiff", 400, 300);

            let result = convert_image(&input, &resized(format, ResizeMode::Width, 200)).unwrap();

            assert_eq!(dimensions(&result.output_path), (200, 150));
            assert_eq!(result.original_dimensions, (400, 300));
            assert_eq!(result.new_dimensions, (200, 150));
        }
    }

    #[test]
    fn resizes_to_avif() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "photo.png", 400, 300);

        let result = convert_image(
            &input,
            &resized(OutputFormat::Avif, ResizeMode::Percent, 50),
        )
        .unwrap();

        assert_eq!(result.new_dimensions, (200, 150));
    }

    #[test]
    fn resizes_after_exif_orientation() {
        let dir = temp_dir();
        let input = create_portrait_jpeg(dir.path());

        let result =
            convert_image(&input, &resized(OutputFormat::Png, ResizeMode::Width, 20)).unwrap();

        assert_eq!(dimensions(&result.output_path), (20, 30));
    }

    #[test]
    fn leaves_smaller_images_untouched() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "photo.tiff", 40, 30);

        let result =
            convert_image(&input, &resized(OutputFormat::Png, ResizeMode::Width, 400)).unwrap();

        assert_eq!(dimensions(&result.output_path), (40, 30));
    }

    #[test]
    fn keeps_transparency_when_resizing() {
        for format in [OutputFormat::Png, OutputFormat::Webp] {
            let dir = temp_dir();
            let input = create_transparent_image(dir.path(), "transparent.tiff", 40, 40);

            let result = convert_image(&input, &resized(format, ResizeMode::Width, 20)).unwrap();

            let image = image::open(&result.output_path).unwrap().to_rgba8();
            assert_eq!(image.dimensions(), (20, 20));
            assert_eq!(image[(10, 10)][3], 0);
        }
    }

    #[test]
    fn resizes_every_frame_of_an_animated_gif() {
        let dir = temp_dir();
        let input = create_animated_gif(dir.path());

        let result =
            convert_image(&input, &resized(OutputFormat::Webp, ResizeMode::Width, 8)).unwrap();

        assert_eq!(webp_frame_count(&result.output_path), 3);
        assert_eq!(dimensions(&result.output_path), (8, 8));
        assert_eq!(result.original_dimensions, (16, 16));
        assert_eq!(result.new_dimensions, (8, 8));
    }

    #[test]
    fn rejects_out_of_range_sizes() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "photo.png", 20, 20);

        for (mode, value) in [
            (ResizeMode::Width, 0),
            (ResizeMode::Height, 16_385),
            (ResizeMode::Percent, 0),
            (ResizeMode::Percent, 401),
        ] {
            let result = convert_image(&input, &resized(OutputFormat::Webp, mode, value));
            assert!(
                matches!(result, Err(ConversionError::InvalidResize { .. })),
                "{mode:?} {value}"
            );
        }
        assert_eq!(file_names(dir.path()), ["photo.png"]);
    }

    #[test]
    fn rejects_images_too_large_for_webp() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "bandeau.png", 16_384, 1);

        let error = convert_image(&input, &options(OutputFormat::Webp)).unwrap_err();

        assert_eq!(
            error.to_string(),
            "L'image ferait 16384 × 1 px : le format WebP est limité à 16383 px de côté"
        );
        assert_eq!(file_names(dir.path()), ["bandeau.png"]);
    }
}

mod source_file {
    use super::*;

    #[test]
    fn rejects_relative_paths() {
        let error = assert_image_file(Path::new("photo.png")).unwrap_err();
        assert!(error.to_string().contains("invalide"));
    }

    #[test]
    fn rejects_unsupported_extensions() {
        let dir = temp_dir();
        let bmp = dir.path().join("image.bmp");
        fs::write(&bmp, "BM").unwrap();

        let error = assert_image_file(&bmp).unwrap_err();
        assert_eq!(error.to_string(), "Extension non supportée : .bmp");
    }

    #[test]
    fn rejects_avif_input() {
        let dir = temp_dir();
        let error = assert_image_file(&dir.path().join("photo.avif")).unwrap_err();
        assert_eq!(error.to_string(), "Extension non supportée : .avif");
    }

    #[test]
    fn reports_missing_files() {
        let dir = temp_dir();
        let path = dir.path().join("absent.png");

        let error = assert_image_file(&path).unwrap_err();
        assert!(matches!(error, ConversionError::SourceNotFound(_)));
        assert_eq!(
            error.to_string(),
            format!("Le fichier source n'existe pas : {}", path.display())
        );
    }

    #[test]
    fn rejects_directories() {
        let dir = temp_dir();
        let folder = dir.path().join("dossier.png");
        fs::create_dir(&folder).unwrap();

        assert!(matches!(
            assert_image_file(&folder),
            Err(ConversionError::NotAFile(_))
        ));
    }
}

mod writing {
    use super::*;

    #[test]
    fn adds_a_suffix_while_the_name_is_taken() {
        let dir = temp_dir();
        fs::write(dir.path().join("photo.webp"), "").unwrap();
        fs::write(dir.path().join("photo-1.webp"), "").unwrap();

        let path = write_new_file(dir.path(), "photo", "webp", b"contenu").unwrap();

        assert_eq!(path, dir.path().join("photo-2.webp"));
        assert_eq!(fs::read(path).unwrap(), b"contenu");
    }
}

mod info {
    use super::*;

    #[test]
    fn returns_a_square_thumbnail_and_dimensions() {
        for name in ["photo.png", "photo.tiff"] {
            let dir = temp_dir();
            let input = create_image(dir.path(), name, 300, 200);

            let info = image_info(&input).unwrap();

            assert_eq!((info.width, info.height), (300, 200));
            assert_eq!(info.size, fs::metadata(&input).unwrap().len());
            assert_eq!(
                info.thumbnail.dimensions(),
                (THUMBNAIL_SIZE, THUMBNAIL_SIZE)
            );
            assert_eq!(info.thumbnail[(48, 48)], Rgba([0x33, 0x66, 0xcc, 255]));
        }
    }

    #[test]
    fn returns_dimensions_after_exif_orientation() {
        let dir = temp_dir();
        let input = create_portrait_jpeg(dir.path());

        let info = image_info(&input).unwrap();

        assert_eq!((info.width, info.height), (40, 60));
    }

    #[test]
    fn rejects_unsupported_extensions() {
        let dir = temp_dir();
        let error = image_info(&dir.path().join("image.bmp")).unwrap_err();
        assert!(error.to_string().contains("Extension non supportée"));
    }
}

mod animation {
    use super::*;

    #[test]
    fn keeps_every_frame_delay_including_the_last() {
        let dir = temp_dir();
        let input = create_gif(dir.path(), "pause.gif", &[100, 200, 3000]);

        let result = convert_image(&input, &options(OutputFormat::Webp)).unwrap();

        assert_eq!(webp_frame_delays_ms(&result.output_path), [100, 200, 3000]);
    }

    #[test]
    fn gives_near_zero_delays_a_tenth_of_a_second() {
        let dir = temp_dir();
        let input = create_gif(dir.path(), "rapide.gif", &[0, 10, 0]);

        let result = convert_image(&input, &options(OutputFormat::Webp)).unwrap();

        assert_eq!(webp_frame_delays_ms(&result.output_path), [100, 100, 100]);
    }

    #[test]
    fn converts_a_single_frame_gif_to_a_still_webp() {
        let dir = temp_dir();
        let input = create_gif(dir.path(), "fixe.gif", &[100]);

        let result = convert_image(&input, &options(OutputFormat::Webp)).unwrap();

        let image = image::open(&result.output_path).unwrap().to_rgb8();
        assert_eq!(image.dimensions(), (16, 16));
        assert!(image[(8, 8)][0] > 200, "{:?}", image[(8, 8)]);
        // Un WebP fixe ne contient aucune image d'animation
        assert_eq!(webp_frame_count(&result.output_path), 0);
    }

    #[test]
    fn detects_gif_by_content_not_by_extension() {
        let dir = temp_dir();
        let gif = create_gif(dir.path(), "anim.gif", &[100, 100, 100]);
        let gif_named_png = dir.path().join("deguise.png");
        fs::copy(&gif, &gif_named_png).unwrap();
        let png = create_image(dir.path(), "photo.png", 20, 20);
        let png_named_gif = dir.path().join("faux.gif");
        fs::copy(&png, &png_named_gif).unwrap();

        let animated = convert_image(&gif_named_png, &options(OutputFormat::Webp)).unwrap();
        let still = convert_image(&png_named_gif, &options(OutputFormat::Webp)).unwrap();

        assert_eq!(webp_frame_count(&animated.output_path), 3);
        assert_eq!(dimensions(&still.output_path), (20, 20));
    }
}

mod color_profile {
    use super::*;

    #[test]
    fn converts_wide_gamut_pixels_to_srgb() {
        let dir = temp_dir();
        let adobe_rgb = moxcms::ColorProfile::new_adobe_rgb().encode().unwrap();
        let input = create_profiled_png(
            dir.path(),
            "adobe.png",
            Rgba([100, 200, 50, 255]),
            adobe_rgb,
        );

        let result = convert_image(&input, &options(OutputFormat::Png)).unwrap();

        // Ce vert Adobe RGB est plus saturé que le même triplet en sRGB : le rouge baisse
        let pixel = image::open(&result.output_path).unwrap().to_rgba8()[(4, 4)];
        assert!(pixel[0] < 80, "{pixel:?}");
        assert!(pixel[1].abs_diff(200) <= 6, "{pixel:?}");
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn leaves_srgb_pixels_and_transparency_unchanged() {
        let dir = temp_dir();
        let srgb = moxcms::ColorProfile::new_srgb().encode().unwrap();
        let input = create_profiled_png(dir.path(), "srgb.png", Rgba([100, 200, 50, 128]), srgb);

        let result = convert_image(&input, &options(OutputFormat::Png)).unwrap();

        let pixel = image::open(&result.output_path).unwrap().to_rgba8()[(4, 4)];
        for (channel, expected) in pixel.0.iter().zip([100u8, 200, 50]) {
            assert!(channel.abs_diff(expected) <= 1, "{pixel:?}");
        }
        assert_eq!(pixel[3], 128);
    }

    #[test]
    fn ignores_an_invalid_profile() {
        let dir = temp_dir();
        let input = create_profiled_png(
            dir.path(),
            "casse.png",
            Rgba([100, 200, 50, 255]),
            b"pas un profil".to_vec(),
        );

        let result = convert_image(&input, &options(OutputFormat::Png)).unwrap();

        let pixel = image::open(&result.output_path).unwrap().to_rgba8()[(4, 4)];
        assert_eq!(pixel, Rgba([100, 200, 50, 255]));
    }
}

mod limits {
    use super::*;

    #[test]
    fn rejects_a_huge_upscale_before_allocating() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "bande.png", 10, 16_000);
        let upscale = ConvertOptions {
            resize: Some(Resize {
                mode: ResizeMode::Width,
                value: 16_384,
                allow_upscale: true,
            }),
            ..options(OutputFormat::Png)
        };

        // 16 384 × 26 214 400 px : plus de 1 500 Go si l'allocation était tentée
        let error = convert_image(&input, &upscale).unwrap_err();

        assert!(
            matches!(error, ConversionError::TooManyPixels { .. }),
            "{error}"
        );
        assert_eq!(file_names(dir.path()), ["bande.png"]);
    }

    #[test]
    fn rejects_an_upscale_beyond_the_format_limit_before_resizing() {
        let dir = temp_dir();
        let input = create_image(dir.path(), "photo.png", 5000, 3000);
        let upscale = ConvertOptions {
            resize: Some(Resize {
                mode: ResizeMode::Percent,
                value: 400,
                allow_upscale: true,
            }),
            ..options(OutputFormat::Webp)
        };

        let start = std::time::Instant::now();
        let error = convert_image(&input, &upscale).unwrap_err();

        assert!(matches!(error, ConversionError::TooLarge { .. }), "{error}");
        // Un redimensionnement vers 20 000 × 12 000 px prendrait plusieurs secondes
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
    }
}
