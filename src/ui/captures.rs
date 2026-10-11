// Rendu hors écran de chaque écran, en clair et en sombre. Les images sont écrites dans
// target/captures pour la revue visuelle ; le test vérifie que tout se dessine sans erreur
// et que les boutons envoient les bons messages.
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use iced::widget::image;
use iced::{Settings, Size};
use iced_test::simulator::Simulator;

use super::*;

const WINDOW: Size = Size::new(1000.0, 700.0);
// Dossier fictif des images d'exemple : rien n'est lu ni écrit sur le disque
const SAMPLE_DIR: &str = "/photos";

fn settings() -> Settings {
    Settings {
        default_font: theme::FONT,
        fonts: vec![
            include_bytes!("../../assets/fonts/Inter-Regular.ttf")
                .as_slice()
                .into(),
            include_bytes!("../../assets/fonts/Inter-Medium.ttf")
                .as_slice()
                .into(),
            include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")
                .as_slice()
                .into(),
            lucide_icons::LUCIDE_FONT_BYTES.into(),
        ],
        ..Settings::default()
    }
}

fn captures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("captures")
}

fn capture(app: &App, name: &str) {
    let mut ui = Simulator::with_size(settings(), WINDOW, app.view());
    let snapshot = ui.snapshot(&app.theme()).expect("rendu de l'interface");
    assert!(
        snapshot
            .matches_image(captures_dir().join(name))
            .expect("écriture de la capture")
    );
}

fn messages(app: &App, label: &str) -> Vec<Message> {
    let mut ui = Simulator::with_size(settings(), WINDOW, app.view());
    ui.click(label)
        .unwrap_or_else(|_| panic!("« {label} » introuvable"));
    ui.into_messages().collect()
}

fn app(dark_mode: bool) -> (App, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    Preferences { dark_mode }
        .save(&dir.path().join("preferences.json"))
        .unwrap();
    (App::with_data_dir(dir.path()), dir)
}

// Dégradé de couleur en guise de miniature
fn thumbnail(hue: u8) -> image::Handle {
    let pixels = (0..THUMBNAIL_SIZE * THUMBNAIL_SIZE)
        .flat_map(|index| {
            let x = (index % THUMBNAIL_SIZE) as u8;
            let y = (index / THUMBNAIL_SIZE) as u8;
            [hue.wrapping_add(x), 120u8.wrapping_add(y), 255 - hue, 255]
        })
        .collect::<Vec<u8>>();
    image::Handle::from_rgba(THUMBNAIL_SIZE, THUMBNAIL_SIZE, pixels)
}

fn result(name: &str, new_size: u64, new_dimensions: (u32, u32)) -> ConversionResult {
    let original_size = 2_818_000;
    ConversionResult {
        output_path: PathBuf::from(format!("{SAMPLE_DIR}/{name}")),
        original_size,
        new_size,
        compression_ratio: (1.0 - new_size as f64 / original_size as f64) * 100.0,
        original_dimensions: (4000, 3000),
        new_dimensions,
    }
}

fn sample_images() -> Vec<ImageEntry> {
    let entry = |id, name: &str, size, dimensions, status| ImageEntry {
        id,
        name: name.to_owned(),
        path: PathBuf::from(format!("{SAMPLE_DIR}/{name}")),
        size,
        dimensions,
        thumbnail: thumbnail((id * 60) as u8),
        status,
    };
    vec![
        entry(
            0,
            "vacances-plage.jpg",
            2_818_000,
            (4000, 3000),
            Status::Converted(result("vacances-plage.webp", 212_400, (1920, 1440))),
        ),
        entry(
            1,
            "logo-transparent.png",
            56_309,
            (2000, 2000),
            Status::Converted(result("logo-transparent.webp", 3_100_000, (1920, 1920))),
        ),
        entry(
            2,
            "panorama-montagne.tiff",
            48_120_000,
            (8000, 3000),
            Status::Converting,
        ),
        entry(
            3,
            "un-nom-de-fichier-vraiment-tres-long-pour-verifier-la-troncature-dans-la-liste.jpg",
            1_250_000,
            (3000, 4000),
            Status::Pending,
        ),
        entry(4, "icone.png", 4_096, (512, 512), Status::Pending),
        entry(
            5,
            "scan-abime.gif",
            820_000,
            (1200, 900),
            Status::Failed("Image illisible : fin de fichier inattendue".to_owned()),
        ),
    ]
}

fn fill_history(app: &mut App) {
    let conversions = [
        (
            "vacances-plage.jpg",
            "vacances-plage.webp",
            212_400,
            (1920, 1440),
        ),
        (
            "logo-transparent.png",
            "logo-transparent.webp",
            3_100_000,
            (4000, 3000),
        ),
        ("portrait.jpeg", "portrait.avif", 96_000, (4000, 3000)),
        (
            "capture-ecran.png",
            "capture-ecran-1.png",
            1_900_000,
            (2000, 1500),
        ),
    ];
    for (index, (source, output, new_size, new_dimensions)) in conversions.into_iter().enumerate() {
        let timestamp = Utc
            .with_ymd_and_hms(2026, 10, 8 + index as u32, 9 + index as u32, 5, 0)
            .unwrap();
        let item = HistoryItem::new(
            PathBuf::from(format!("{SAMPLE_DIR}/{source}")),
            &result(output, new_size, new_dimensions),
            timestamp,
        );
        app.history.add(item).unwrap();
    }
}

#[test]
fn renders_every_screen_in_light_and_dark() {
    let _ = std::fs::remove_dir_all(captures_dir());

    for (dark_mode, suffix) in [(false, "clair"), (true, "sombre")] {
        let (mut app, _dir) = app(dark_mode);
        capture(&app, &format!("1-vide-{suffix}"));

        app.images = sample_images();
        app.images[2].status = Status::Pending;
        app.settings.resize_enabled = true;
        app.settings.output_dir = Some(PathBuf::from(format!("{SAMPLE_DIR}/converted")));
        app.notice =
            Some("2 fichiers ignorés : format non pris en charge ou fichier illisible".to_owned());
        capture(&app, &format!("2-liste-{suffix}"));

        app.settings.resize_value = String::new();
        capture(&app, &format!("3-taille-invalide-{suffix}"));

        app.settings.resize_value = DEFAULT_RESIZE_PIXELS.to_owned();
        app.images = sample_images();
        app.is_converting = true;
        app.batch = (2, 5);
        app.notice = None;
        capture(&app, &format!("4-conversion-{suffix}"));

        app.is_converting = false;
        app.tab = Tab::History;
        capture(&app, &format!("5-historique-vide-{suffix}"));

        fill_history(&mut app);
        capture(&app, &format!("6-historique-{suffix}"));

        app.is_confirming_clear = true;
        capture(&app, &format!("7-confirmation-{suffix}"));
    }
}

#[test]
fn buttons_send_the_expected_messages() {
    let (mut app, _dir) = app(false);
    app.images = sample_images();

    assert!(matches!(
        messages(&app, "Historique")[..],
        [Message::TabSelected(Tab::History)]
    ));
    assert!(matches!(
        messages(&app, "Convertir les images")[..],
        [Message::Convert]
    ));
    assert!(matches!(
        messages(&app, "Tout retirer")[..],
        [Message::RemoveAllImages]
    ));
    // Le bouton est dans la zone de dépôt cliquable : un seul sélecteur doit s'ouvrir
    assert!(matches!(
        messages(&app, "Choisir des images")[..],
        [Message::SelectFiles]
    ));

    fill_history(&mut app);
    app.tab = Tab::History;
    assert!(matches!(
        messages(&app, "Vider l'historique")[..],
        [Message::ClearHistory]
    ));

    app.is_confirming_clear = true;
    assert!(matches!(
        messages(&app, "Vider")[..],
        [Message::ClearHistoryConfirmed(true)]
    ));
    assert!(matches!(
        messages(&app, "Annuler")[..],
        [Message::ClearHistoryConfirmed(false)]
    ));
}

#[test]
fn convert_button_is_disabled_without_pending_images() {
    let (app, _dir) = app(false);

    assert!(messages(&app, "Convertir les images").is_empty());
}
