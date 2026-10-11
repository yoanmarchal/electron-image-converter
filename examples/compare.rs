// Temps et tailles de sortie de chaque format, pour comparer des réglages d'encodeur.
// cargo run --release --example compare -- <dossier de sortie> <images…>
// La variable d'environnement QUALITY règle la qualité (80 par défaut).
use std::path::PathBuf;
use std::time::Instant;

use image_converter::conversion::{
    ConvertOptions, FormatChoice, OutputFormat, convert_image, image_info,
};
use image_converter::format::{format_dimensions, format_file_size};

fn main() {
    let mut args = std::env::args().skip(1);
    let output_dir = PathBuf::from(args.next().expect("dossier de sortie manquant"));
    let quality = std::env::var("QUALITY").map_or(80, |value| value.parse().expect("QUALITY"));

    for input in args {
        // Les chemins relatifs sont refusés par la conversion
        let input = std::path::absolute(&input).expect("chemin");
        let input = input.as_path();

        let start = Instant::now();
        let info = image_info(input).expect("lecture");
        println!(
            "{} : {}, {} (lecture et miniature en {:?})",
            input.display(),
            format_dimensions((info.width, info.height)),
            format_file_size(info.size),
            start.elapsed()
        );

        for format in OutputFormat::ALL {
            let options = ConvertOptions {
                output_dir: Some(output_dir.clone()),
                quality,
                format: FormatChoice::Fixed(format),
                resize: None,
            };
            let start = Instant::now();
            let result = convert_image(input, &options).expect("conversion");
            println!(
                "  {} : {} octets en {:?}",
                format.extension(),
                result.new_size,
                start.elapsed()
            );
        }
    }
}
