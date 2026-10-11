// Historique des conversions, enregistré en JSON dans le dossier de données de l'application
use std::io;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::conversion::ConversionResult;
use crate::storage;

// Seules les conversions les plus récentes sont gardées
pub const MAX_HISTORY: usize = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub original_size: u64,
    pub new_size: u64,
    pub compression_ratio: f64,
    pub original_dimensions: (u32, u32),
    pub new_dimensions: (u32, u32),
    pub timestamp: DateTime<Utc>,
}

impl HistoryItem {
    pub fn new(
        original_path: PathBuf,
        result: &ConversionResult,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            original_path,
            output_path: result.output_path.clone(),
            original_size: result.original_size,
            new_size: result.new_size,
            compression_ratio: result.compression_ratio,
            original_dimensions: result.original_dimensions,
            new_dimensions: result.new_dimensions,
            timestamp,
        }
    }
}

#[derive(Debug)]
pub struct History {
    path: PathBuf,
    items: Vec<HistoryItem>,
}

impl History {
    /// Un fichier absent ou illisible donne un historique vide : il ne doit jamais empêcher
    /// l'application de démarrer.
    pub fn load(path: PathBuf) -> Self {
        let items = storage::load(&path);
        Self { path, items }
    }

    /// Conversions de la plus récente à la plus ancienne.
    pub fn items(&self) -> &[HistoryItem] {
        &self.items
    }

    pub fn add(&mut self, item: HistoryItem) -> io::Result<()> {
        // Une autre fenêtre de l'application a pu enregistrer des conversions entre-temps :
        // on repart du fichier pour ne pas effacer ses entrées
        if let Some(items) = storage::read(&self.path) {
            self.items = items;
        }
        self.items.insert(0, item);
        self.items.truncate(MAX_HISTORY);
        self.save()
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.items.clear();
        self.save()
    }

    fn save(&self) -> io::Result<()> {
        storage::save(&self.path, &self.items)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn item(name: &str) -> HistoryItem {
        HistoryItem {
            original_path: PathBuf::from(format!("/images/{name}.png")),
            output_path: PathBuf::from(format!("/images/{name}.webp")),
            original_size: 2000,
            new_size: 500,
            compression_ratio: 75.0,
            original_dimensions: (4000, 3000),
            new_dimensions: (1920, 1440),
            timestamp: DateTime::from_timestamp(1_791_700_000, 0).unwrap(),
        }
    }

    fn history_path(dir: &tempfile::TempDir) -> PathBuf {
        dir.path().join("donnees").join("history.json")
    }

    #[test]
    fn starts_empty_without_a_file() {
        let dir = tempfile::tempdir().unwrap();

        assert!(History::load(history_path(&dir)).items().is_empty());
    }

    #[test]
    fn starts_empty_with_an_unreadable_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        fs::write(&path, "{ pas du json").unwrap();

        assert!(History::load(path).items().is_empty());
    }

    #[test]
    fn saves_most_recent_first_and_reloads() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = History::load(history_path(&dir));

        history.add(item("premiere")).unwrap();
        history.add(item("seconde")).unwrap();

        let reloaded = History::load(history_path(&dir));
        assert_eq!(reloaded.items(), [item("seconde"), item("premiere")]);
    }

    #[test]
    fn keeps_only_the_last_hundred() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = History::load(history_path(&dir));

        for index in 0..MAX_HISTORY + 5 {
            history.add(item(&index.to_string())).unwrap();
        }

        let reloaded = History::load(history_path(&dir));
        assert_eq!(reloaded.items().len(), MAX_HISTORY);
        assert_eq!(reloaded.items()[0], item("104"));
        assert_eq!(reloaded.items()[MAX_HISTORY - 1], item("5"));
    }

    #[test]
    fn keeps_entries_saved_by_another_window() {
        let dir = tempfile::tempdir().unwrap();
        let mut first = History::load(history_path(&dir));
        let mut second = History::load(history_path(&dir));

        first.add(item("premiere")).unwrap();
        second.add(item("seconde")).unwrap();

        assert_eq!(second.items(), [item("seconde"), item("premiere")]);
        let reloaded = History::load(history_path(&dir));
        assert_eq!(reloaded.items(), [item("seconde"), item("premiere")]);
    }

    #[test]
    fn clears_the_history() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = History::load(history_path(&dir));
        history.add(item("photo")).unwrap();

        history.clear().unwrap();

        assert!(history.items().is_empty());
        assert!(History::load(history_path(&dir)).items().is_empty());
    }
}
