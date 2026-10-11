// Interface Iced : état de l'application, messages et mise à jour
#[cfg(test)]
mod captures;
mod convert;
mod history;
mod theme;
mod widgets;

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use chrono::Utc;
use iced::widget::{column, container, image};
use iced::{Element, Event, Length, Size, Subscription, Task, Theme, event, window};
use tokio::sync::Semaphore;

use crate::conversion::{
    self, ConversionResult, ConvertOptions, FormatChoice, INPUT_EXTENSIONS, OutputFormat, Resize,
    ResizeMode, THUMBNAIL_SIZE,
};
use crate::history::{History, HistoryItem};
use crate::preferences::Preferences;
use crate::storage;
use theme::Tokens;

const DEFAULT_RESIZE_PIXELS: &str = "1920";
const DEFAULT_RESIZE_PERCENT: &str = "50";

// Une photo de 50 mégapixels décodée occupe 200 Mo, et autant pendant son redimensionnement
const MAX_PARALLEL_CONVERSIONS: usize = 4;

// Chaque image décodée occupe plusieurs dizaines de Mo : on borne le nombre de lectures
// simultanées quand l'utilisateur dépose un dossier entier
static LOAD_LIMIT: LazyLock<Semaphore> = LazyLock::new(|| {
    Semaphore::new(std::thread::available_parallelism().map_or(2, |count| count.get()))
});

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Image Converter")
        .subscription(App::subscription)
        .theme(App::theme)
        .font(include_bytes!("../../assets/fonts/Inter-Regular.ttf").as_slice())
        .font(include_bytes!("../../assets/fonts/Inter-Medium.ttf").as_slice())
        .font(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf").as_slice())
        .font(lucide_icons::LUCIDE_FONT_BYTES)
        .default_font(theme::FONT)
        .window(window::Settings {
            size: Size::new(1000.0, 700.0),
            min_size: Some(Size::new(800.0, 600.0)),
            icon: window::icon::from_file_data(
                include_bytes!("../../assets/icons/256x256.png"),
                None,
            )
            .ok(),
            ..window::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Convert,
    History,
}

type ImageId = u64;

#[derive(Debug, Clone)]
pub struct ImageEntry {
    id: ImageId,
    name: String,
    path: PathBuf,
    size: u64,
    dimensions: (u32, u32),
    thumbnail: image::Handle,
    status: Status,
}

#[derive(Debug, Clone)]
enum Status {
    Pending,
    Converting,
    Converted(ConversionResult),
    Failed(String),
}

#[derive(Debug, Clone)]
struct Settings {
    format: FormatChoice,
    quality: u8,
    /// Sans dossier de sortie, les fichiers sont écrits à côté des originaux.
    output_dir: Option<PathBuf>,
    resize_enabled: bool,
    resize_mode: ResizeMode,
    /// Saisie brute du champ : validée par `resize`.
    resize_value: String,
    allow_upscale: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format: FormatChoice::Fixed(OutputFormat::Webp),
            quality: 80,
            output_dir: None,
            resize_enabled: false,
            resize_mode: ResizeMode::LongestSide,
            resize_value: DEFAULT_RESIZE_PIXELS.to_owned(),
            allow_upscale: false,
        }
    }
}

impl Settings {
    fn is_lossless(&self) -> bool {
        self.format == FormatChoice::Fixed(OutputFormat::Png)
    }

    /// Redimensionnement demandé, ou le message à afficher si la saisie est invalide.
    fn resize(&self) -> Result<Option<Resize>, String> {
        if !self.resize_enabled {
            return Ok(None);
        }
        let resize = Resize {
            mode: self.resize_mode,
            // Une saisie vide ou trop longue pour un entier donne 0, refusé par la validation
            value: self.resize_value.parse().unwrap_or(0),
            allow_upscale: self.allow_upscale,
        };
        resize.validate().map_err(|error| error.to_string())?;
        Ok(Some(resize))
    }

    fn options(&self) -> Option<ConvertOptions> {
        Some(ConvertOptions {
            output_dir: self.output_dir.clone(),
            quality: self.quality,
            format: self.format,
            resize: self.resize().ok()?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    DarkModeToggled,
    FileHovered,
    HoverLeft,
    FileDropped(PathBuf),
    SelectFiles,
    FilesSelected(Vec<PathBuf>),
    ImageLoaded(u64, Option<ImageEntry>),
    RemoveImage(ImageId),
    RemoveAllImages,
    FormatSelected(FormatChoice),
    QualityChanged(u8),
    ResizeToggled(bool),
    ResizeModeSelected(ResizeMode),
    ResizeValueChanged(String),
    AllowUpscaleToggled(bool),
    SelectOutputDir,
    OutputDirSelected(Option<PathBuf>),
    OutputDirReset,
    Convert,
    Converted(ImageId, PathBuf, Result<ConversionResult, String>),
    OpenFile(PathBuf),
    ClearHistory,
    ClearHistoryConfirmed(bool),
}

pub struct App {
    tab: Tab,
    images: Vec<ImageEntry>,
    next_id: ImageId,
    settings: Settings,
    is_converting: bool,
    /// Images traitées et images à traiter du lot en cours.
    batch: (usize, usize),
    /// Conversions en cours d'exécution.
    active_conversions: usize,
    /// Images en cours de lecture (dimensions et miniature).
    loading: usize,
    /// Augmente quand la liste est vidée : les lectures lancées avant sont alors ignorées.
    load_generation: u64,
    /// Fichiers refusés depuis le dernier ajout.
    ignored: usize,
    is_hovered: bool,
    /// Un sélecteur de fichiers ou de dossier est ouvert : ne pas en ouvrir un second.
    is_dialog_open: bool,
    /// L'écran Historique demande de confirmer le vidage.
    is_confirming_clear: bool,
    /// Message de la barre de statut (fichiers ignorés, ouverture impossible…).
    notice: Option<String>,
    history: History,
    preferences: Preferences,
    preferences_path: PathBuf,
    tokens: Tokens,
    theme: Theme,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self::with_data_dir(&storage::data_dir());
        // Fichiers passés en arguments : « Ouvrir avec » ou dépôt sur l'icône de l'application
        // Un chemin relatif (lancement depuis un terminal) est résolu depuis le dossier courant
        let paths = std::env::args_os()
            .skip(1)
            .filter_map(|argument| std::path::absolute(argument).ok())
            .collect();
        let task = app.load_images(paths);
        (app, task)
    }

    fn with_data_dir(data_dir: &Path) -> Self {
        let preferences_path = data_dir.join("preferences.json");
        let preferences = Preferences::load(&preferences_path);
        Self {
            tab: Tab::Convert,
            images: Vec::new(),
            next_id: 0,
            settings: Settings::default(),
            is_converting: false,
            batch: (0, 0),
            active_conversions: 0,
            loading: 0,
            load_generation: 0,
            ignored: 0,
            is_hovered: false,
            is_dialog_open: false,
            is_confirming_clear: false,
            notice: None,
            history: History::load(data_dir.join("history.json")),
            preferences,
            preferences_path,
            tokens: theme::tokens(preferences.dark_mode),
            theme: theme::iced_theme(preferences.dark_mode),
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn pending_count(&self) -> usize {
        self.images
            .iter()
            .filter(|image| matches!(image.status, Status::Pending))
            .count()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.tab = tab;
                self.is_confirming_clear = false;
            }
            Message::DarkModeToggled => {
                self.preferences.dark_mode = !self.preferences.dark_mode;
                self.tokens = theme::tokens(self.preferences.dark_mode);
                self.theme = theme::iced_theme(self.preferences.dark_mode);
                if self.preferences.save(&self.preferences_path).is_err() {
                    self.notice = Some("Impossible d'enregistrer les préférences".to_owned());
                }
            }

            Message::FileHovered => self.is_hovered = !self.is_converting,
            Message::HoverLeft => self.is_hovered = false,
            Message::FileDropped(path) => {
                self.is_hovered = false;
                if !self.is_converting {
                    self.tab = Tab::Convert;
                    return self.load_images(vec![path]);
                }
            }
            Message::SelectFiles => {
                if !self.is_dialog_open && !self.is_converting {
                    self.is_dialog_open = true;
                    return Task::perform(pick_files(), Message::FilesSelected);
                }
            }
            Message::FilesSelected(paths) => {
                self.is_dialog_open = false;
                // Le sélecteur a pu rester ouvert pendant qu'une conversion démarrait
                if !self.is_converting {
                    return self.load_images(paths);
                }
            }
            Message::ImageLoaded(generation, loaded) => {
                self.loading = self.loading.saturating_sub(1);
                match loaded {
                    // La liste a été vidée pendant la lecture : l'image ne doit pas y revenir
                    _ if generation != self.load_generation => {}
                    Some(image) => {
                        // Les lectures se terminent dans le désordre : on garde l'ordre d'ajout
                        let position = self.images.partition_point(|other| other.id < image.id);
                        self.images.insert(position, image);
                    }
                    None => {
                        self.ignored += 1;
                        self.notice = Some(format!(
                            "{} : format non pris en charge ou fichier illisible",
                            plural(self.ignored, "fichier ignoré", "fichiers ignorés")
                        ));
                    }
                }
            }
            Message::RemoveImage(id) => self.images.retain(|image| image.id != id),
            Message::RemoveAllImages => {
                self.images.clear();
                self.load_generation += 1;
            }

            Message::FormatSelected(format) => {
                self.change_settings(|settings| settings.format = format);
            }
            Message::QualityChanged(quality) => {
                if !self.settings.is_lossless() {
                    self.change_settings(|settings| settings.quality = quality);
                }
            }
            Message::ResizeToggled(enabled) => {
                self.change_settings(|settings| settings.resize_enabled = enabled);
            }
            Message::ResizeModeSelected(mode) => self.change_settings(|settings| {
                // Une valeur en pixels n'a pas de sens en pourcentage, et inversement
                let was_percent = settings.resize_mode == ResizeMode::Percent;
                let is_percent = mode == ResizeMode::Percent;
                if was_percent != is_percent {
                    settings.resize_value = if is_percent {
                        DEFAULT_RESIZE_PERCENT
                    } else {
                        DEFAULT_RESIZE_PIXELS
                    }
                    .to_owned();
                }
                settings.resize_mode = mode;
            }),
            Message::ResizeValueChanged(value) => self.change_settings(|settings| {
                settings.resize_value =
                    value.chars().filter(char::is_ascii_digit).take(5).collect();
            }),
            Message::AllowUpscaleToggled(allow) => {
                self.change_settings(|settings| settings.allow_upscale = allow);
            }
            Message::SelectOutputDir => {
                if !self.is_dialog_open && !self.is_converting {
                    self.is_dialog_open = true;
                    return Task::perform(pick_folder(), Message::OutputDirSelected);
                }
            }
            Message::OutputDirSelected(dir) => {
                self.is_dialog_open = false;
                if dir.is_some() {
                    self.change_settings(|settings| settings.output_dir = dir);
                }
            }
            Message::OutputDirReset => self.change_settings(|settings| settings.output_dir = None),

            Message::Convert => {
                let pending = self.pending_count();
                // Pas de conversion tant que des images se chargent : le lot doit être complet
                let is_ready = !self.is_converting && self.loading == 0 && pending > 0;
                if is_ready && self.settings.options().is_some() {
                    self.is_converting = true;
                    self.batch = (0, pending);
                    self.notice = None;
                    return self.start_conversions();
                }
            }
            Message::Converted(id, original_path, result) => {
                self.batch.0 += 1;
                self.active_conversions = self.active_conversions.saturating_sub(1);
                let status = match result {
                    Ok(result) => {
                        let item = HistoryItem::new(original_path, &result, Utc::now());
                        if self.history.add(item).is_err() {
                            self.notice = Some("Impossible d'enregistrer l'historique".to_owned());
                        }
                        Status::Converted(result)
                    }
                    Err(error) => Status::Failed(error),
                };
                // L'image a pu être retirée de la liste pendant sa conversion
                if let Some(image) = self.images.iter_mut().find(|image| image.id == id) {
                    image.status = status;
                }
                return self.start_conversions();
            }

            Message::OpenFile(path) => self.open_file(&path),
            Message::ClearHistory => self.is_confirming_clear = true,
            Message::ClearHistoryConfirmed(confirmed) => {
                self.is_confirming_clear = false;
                if confirmed && self.history.clear().is_err() {
                    self.notice = Some("Impossible d'enregistrer l'historique".to_owned());
                }
            }
        }
        Task::none()
    }

    fn load_images(&mut self, paths: Vec<PathBuf>) -> Task<Message> {
        if self.loading == 0 {
            self.ignored = 0;
            self.notice = None;
        }
        let generation = self.load_generation;
        Task::batch(paths.into_iter().map(|path| {
            let id = self.next_id;
            self.next_id += 1;
            self.loading += 1;
            Task::perform(load_image(id, path), move |loaded| {
                Message::ImageLoaded(generation, loaded)
            })
        }))
    }

    /// Après un changement de réglages, les images déjà traitées redeviennent à convertir.
    fn change_settings(&mut self, change: impl FnOnce(&mut Settings)) {
        if self.is_converting {
            return;
        }
        change(&mut self.settings);
        for image in &mut self.images {
            if matches!(image.status, Status::Converted(_) | Status::Failed(_)) {
                image.status = Status::Pending;
            }
        }
    }

    /// Nombre de conversions menées de front. Seul l'encodeur AVIF utilise déjà tous les
    /// cœurs ; les autres n'en occupent qu'un, mais chaque image décodée pèse lourd en mémoire.
    fn conversion_slots(&self) -> usize {
        if self.settings.format == FormatChoice::Fixed(OutputFormat::Avif) {
            1
        } else {
            std::thread::available_parallelism()
                .map_or(1, |count| count.get())
                .min(MAX_PARALLEL_CONVERSIONS)
        }
    }

    /// Lance les conversions des images en attente, dans la limite de `conversion_slots`.
    /// Le lot est terminé quand plus rien n'est en attente ni en cours.
    fn start_conversions(&mut self) -> Task<Message> {
        let Some(options) = self.settings.options() else {
            self.is_converting = false;
            return Task::none();
        };
        let slots = self.conversion_slots();

        let mut tasks = Vec::new();
        for image in &mut self.images {
            if self.active_conversions >= slots {
                break;
            }
            if !matches!(image.status, Status::Pending) {
                continue;
            }
            image.status = Status::Converting;
            self.active_conversions += 1;

            let id = image.id;
            let path = image.path.clone();
            let original_path = path.clone();
            let options = options.clone();
            tasks.push(Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        conversion::convert_image(&path, &options)
                            .map_err(|error| error.to_string())
                    })
                    .await
                    .unwrap_or_else(|_| Err("Erreur inattendue pendant la conversion".to_owned()))
                },
                move |result| Message::Converted(id, original_path.clone(), result),
            ));
        }

        if self.active_conversions == 0 {
            self.is_converting = false;
        }
        Task::batch(tasks)
    }

    fn open_file(&mut self, path: &Path) {
        // Seulement des images produites par l'application : jamais d'exécutable ni de script
        let is_output = OutputFormat::ALL
            .iter()
            .any(|format| format.extension() == conversion::extension(path));
        if !path.is_absolute() || !is_output {
            return;
        }
        let name = file_name(path);
        self.notice = if !path.exists() {
            Some(format!("Le fichier n'existe plus : {name}"))
        } else if open::that_detached(path).is_err() {
            Some(format!("Impossible d'ouvrir {name}"))
        } else {
            None
        };
    }

    fn view(&self) -> Element<'_, Message> {
        let body = match self.tab {
            Tab::Convert => convert::view(self),
            Tab::History => history::view(self),
        };
        container(column![
            widgets::header(self),
            container(body).height(Length::Fill),
            widgets::status_bar(self),
        ])
        .style(move |_| theme::page(self.tokens))
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _window| match event {
            Event::Window(window::Event::FileHovered(_)) => Some(Message::FileHovered),
            Event::Window(window::Event::FilesHoveredLeft) => Some(Message::HoverLeft),
            Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
            _ => None,
        })
    }
}

async fn load_image(id: ImageId, path: PathBuf) -> Option<ImageEntry> {
    let _permit = LOAD_LIMIT.acquire().await.ok()?;
    tokio::task::spawn_blocking(move || {
        let info = conversion::image_info(&path).ok()?;
        Some(ImageEntry {
            id,
            name: file_name(&path),
            size: info.size,
            dimensions: (info.width, info.height),
            thumbnail: image::Handle::from_rgba(
                THUMBNAIL_SIZE,
                THUMBNAIL_SIZE,
                info.thumbnail.into_raw(),
            ),
            status: Status::Pending,
            path,
        })
    })
    .await
    .ok()
    .flatten()
}

async fn pick_files() -> Vec<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Choisir des images")
        .add_filter("Images", &INPUT_EXTENSIONS)
        .pick_files()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|file| file.path().to_owned())
        .collect()
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Choisir le dossier de sortie")
        .pick_folder()
        .await
        .map(|folder| folder.path().to_owned())
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || "inconnu".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// « 1 image », « 3 images » : le nombre suivi du nom accordé.
fn plural(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count > 1 { plural } else { singular })
}

const FORMAT_CHOICES: [FormatChoice; 5] = [
    FormatChoice::Fixed(OutputFormat::Webp),
    FormatChoice::Fixed(OutputFormat::Jpg),
    FormatChoice::Fixed(OutputFormat::Png),
    FormatChoice::Fixed(OutputFormat::Avif),
    FormatChoice::Original,
];

const RESIZE_MODES: [ResizeMode; 4] = [
    ResizeMode::LongestSide,
    ResizeMode::Width,
    ResizeMode::Height,
    ResizeMode::Percent,
];

// Libellés des listes déroulantes
impl fmt::Display for FormatChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Fixed(OutputFormat::Webp) => "WebP - Meilleur rapport qualité/taille",
            Self::Fixed(OutputFormat::Jpg) => "JPEG - Compatible partout",
            Self::Fixed(OutputFormat::Png) => "PNG - Sans perte avec transparence",
            Self::Fixed(OutputFormat::Avif) => "AVIF - Format moderne haute performance",
            Self::Original => "Format d'origine - Pour seulement redimensionner",
        })
    }
}

impl fmt::Display for ResizeMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LongestSide => "Côté le plus long",
            Self::Width => "Largeur",
            Self::Height => "Hauteur",
            Self::Percent => "Pourcentage",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> (App, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        (App::with_data_dir(dir.path()), dir)
    }

    fn image(id: ImageId, status: Status) -> ImageEntry {
        ImageEntry {
            id,
            name: format!("photo-{id}.png"),
            path: PathBuf::from(format!("/images/photo-{id}.png")),
            size: 1000,
            dimensions: (4000, 3000),
            thumbnail: image::Handle::from_rgba(1, 1, vec![0; 4]),
            status,
        }
    }

    fn result() -> ConversionResult {
        ConversionResult {
            output_path: PathBuf::from("/images/photo-1.webp"),
            original_size: 1000,
            new_size: 250,
            compression_ratio: 75.0,
            original_dimensions: (4000, 3000),
            new_dimensions: (4000, 3000),
        }
    }

    #[test]
    fn changing_a_setting_resets_processed_images_to_pending() {
        let (mut app, _dir) = app();
        app.images = vec![
            image(0, Status::Converted(result())),
            image(1, Status::Failed("erreur".to_owned())),
        ];

        let _ = app.update(Message::QualityChanged(60));

        assert_eq!(app.settings.quality, 60);
        assert_eq!(app.pending_count(), 2);
    }

    #[test]
    fn quality_does_not_change_in_png() {
        let (mut app, _dir) = app();
        let _ = app.update(Message::FormatSelected(FormatChoice::Fixed(
            OutputFormat::Png,
        )));
        app.images = vec![image(0, Status::Converted(result()))];

        let _ = app.update(Message::QualityChanged(10));

        assert_eq!(app.settings.quality, 80);
        assert_eq!(app.pending_count(), 0);
    }

    #[test]
    fn settings_are_locked_while_converting() {
        let (mut app, _dir) = app();
        app.is_converting = true;

        let _ = app.update(Message::QualityChanged(10));
        let _ = app.update(Message::ResizeToggled(true));

        assert_eq!(app.settings.quality, 80);
        assert!(!app.settings.resize_enabled);
    }

    #[test]
    fn resize_input_keeps_only_digits() {
        let (mut app, _dir) = app();
        let _ = app.update(Message::ResizeToggled(true));

        let _ = app.update(Message::ResizeValueChanged("12a 80px".to_owned()));

        assert_eq!(app.settings.resize_value, "1280");
        assert_eq!(
            app.settings.resize(),
            Ok(Some(Resize {
                mode: ResizeMode::LongestSide,
                value: 1280,
                allow_upscale: false,
            }))
        );
    }

    #[test]
    fn invalid_size_blocks_conversion() {
        let (mut app, _dir) = app();
        app.images = vec![image(0, Status::Pending)];
        let _ = app.update(Message::ResizeToggled(true));
        let _ = app.update(Message::ResizeValueChanged(String::new()));

        let _ = app.update(Message::Convert);

        assert!(app.settings.resize().is_err());
        assert!(!app.is_converting);
    }

    #[test]
    fn switching_unit_restores_a_default_value() {
        let (mut app, _dir) = app();
        let _ = app.update(Message::ResizeValueChanged("800".to_owned()));

        let _ = app.update(Message::ResizeModeSelected(ResizeMode::Width));
        assert_eq!(app.settings.resize_value, "800");

        let _ = app.update(Message::ResizeModeSelected(ResizeMode::Percent));
        assert_eq!(app.settings.resize_value, DEFAULT_RESIZE_PERCENT);

        let _ = app.update(Message::ResizeModeSelected(ResizeMode::Height));
        assert_eq!(app.settings.resize_value, DEFAULT_RESIZE_PIXELS);
    }

    #[test]
    fn successful_conversion_is_added_to_history() {
        let (mut app, dir) = app();
        app.images = vec![image(0, Status::Converting)];
        app.is_converting = true;
        app.batch = (0, 1);

        let _ = app.update(Message::Converted(
            0,
            PathBuf::from("/images/photo-0.png"),
            Ok(result()),
        ));

        assert!(matches!(app.images[0].status, Status::Converted(_)));
        assert!(!app.is_converting);
        assert_eq!(app.batch, (1, 1));
        let saved = History::load(dir.path().join("history.json"));
        assert_eq!(saved.items().len(), 1);
        assert_eq!(saved.items()[0].output_path, result().output_path);
    }

    #[test]
    fn failure_does_not_stop_the_batch_or_reach_history() {
        let (mut app, _dir) = app();
        app.images = vec![image(0, Status::Converting)];
        app.is_converting = true;

        let _ = app.update(Message::Converted(
            0,
            PathBuf::from("/images/photo-0.png"),
            Err("Image illisible".to_owned()),
        ));

        assert!(
            matches!(&app.images[0].status, Status::Failed(error) if error == "Image illisible")
        );
        assert!(app.history.items().is_empty());
    }

    #[test]
    fn images_keep_insertion_order() {
        let (mut app, _dir) = app();
        app.loading = 3;

        for id in [2, 0, 1] {
            let _ = app.update(Message::ImageLoaded(0, Some(image(id, Status::Pending))));
        }

        let ids: Vec<ImageId> = app.images.iter().map(|image| image.id).collect();
        assert_eq!(ids, [0, 1, 2]);
        assert_eq!(app.loading, 0);
    }

    #[test]
    fn images_loaded_after_the_list_was_cleared_are_dropped() {
        let (mut app, _dir) = app();
        app.loading = 2;
        let _ = app.update(Message::ImageLoaded(0, Some(image(0, Status::Pending))));

        let _ = app.update(Message::RemoveAllImages);
        let _ = app.update(Message::ImageLoaded(0, Some(image(1, Status::Pending))));

        assert!(app.images.is_empty());
        assert_eq!(app.loading, 0);
    }

    #[test]
    fn conversion_waits_for_images_to_finish_loading() {
        let (mut app, _dir) = app();
        app.images = vec![image(0, Status::Pending)];
        app.loading = 1;

        let _ = app.update(Message::Convert);

        assert!(!app.is_converting);
    }

    #[test]
    fn files_picked_during_a_conversion_are_not_added() {
        let (mut app, _dir) = app();
        app.is_converting = true;
        app.is_dialog_open = true;

        let _ = app.update(Message::FilesSelected(vec![PathBuf::from("/images/a.png")]));

        assert_eq!(app.loading, 0);
        assert!(!app.is_dialog_open);
    }

    #[test]
    fn converts_several_images_at_once_but_one_at_a_time_in_avif() {
        let (mut app, _dir) = app();
        app.images = (0..6).map(|id| image(id, Status::Pending)).collect();

        let _ = app.update(Message::Convert);

        let slots = app.conversion_slots();
        assert!((1..=MAX_PARALLEL_CONVERSIONS).contains(&slots));
        assert_eq!(app.active_conversions, slots);
        assert_eq!(app.batch, (0, 6));
        assert_eq!(app.pending_count(), 6 - slots);

        let (mut app, _dir) = self::app();
        app.images = (0..6).map(|id| image(id, Status::Pending)).collect();
        let _ = app.update(Message::FormatSelected(FormatChoice::Fixed(
            OutputFormat::Avif,
        )));

        let _ = app.update(Message::Convert);

        assert_eq!(app.active_conversions, 1);
    }

    #[test]
    fn batch_ends_when_the_last_conversion_returns() {
        let (mut app, _dir) = app();
        app.images = vec![image(0, Status::Converting), image(1, Status::Converting)];
        app.is_converting = true;
        app.active_conversions = 2;
        app.batch = (0, 2);

        let failure = || Err("Image illisible".to_owned());
        let _ = app.update(Message::Converted(
            0,
            PathBuf::from("/images/a.png"),
            failure(),
        ));
        assert!(app.is_converting);

        let _ = app.update(Message::Converted(
            1,
            PathBuf::from("/images/b.png"),
            failure(),
        ));
        assert!(!app.is_converting);
        assert_eq!(app.batch, (2, 2));
    }

    #[test]
    fn clearing_history_needs_a_confirmation() {
        let (mut app, _dir) = app();
        let item = HistoryItem::new(PathBuf::from("/images/a.png"), &result(), Utc::now());
        app.history.add(item).unwrap();

        let _ = app.update(Message::ClearHistory);
        assert!(app.is_confirming_clear);
        assert_eq!(app.history.items().len(), 1);

        let _ = app.update(Message::ClearHistoryConfirmed(false));
        assert!(!app.is_confirming_clear);
        assert_eq!(app.history.items().len(), 1);

        let _ = app.update(Message::ClearHistory);
        let _ = app.update(Message::ClearHistoryConfirmed(true));
        assert!(app.history.items().is_empty());
    }

    #[test]
    fn reports_ignored_files() {
        let (mut app, _dir) = app();
        app.loading = 2;

        let _ = app.update(Message::ImageLoaded(0, None));
        let _ = app.update(Message::ImageLoaded(0, None));

        assert_eq!(
            app.notice.as_deref(),
            Some("2 fichiers ignorés : format non pris en charge ou fichier illisible")
        );
    }

    #[test]
    fn dark_mode_is_persisted() {
        let (mut app, dir) = app();

        let _ = app.update(Message::DarkModeToggled);

        assert!(App::with_data_dir(dir.path()).preferences.dark_mode);
    }

    #[test]
    fn opens_only_converted_images() {
        let (mut app, _dir) = app();

        app.open_file(&std::env::temp_dir().join("script.exe"));
        assert_eq!(app.notice, None);

        let missing = std::env::temp_dir().join("image-converter-absent.webp");
        app.open_file(&missing);
        assert_eq!(
            app.notice.as_deref(),
            Some("Le fichier n'existe plus : image-converter-absent.webp")
        );
    }
}
