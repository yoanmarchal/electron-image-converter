// Fichiers JSON du dossier de données de l'application (historique, préférences)
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Dossier de données de l'utilisateur, par exemple `%APPDATA%\yoanmarchal\Image Converter\data`.
pub fn data_dir() -> PathBuf {
    ProjectDirs::from("com", "yoanmarchal", "Image Converter")
        .map(|dirs| dirs.data_dir().to_owned())
        .unwrap_or_else(|| std::env::temp_dir().join("image-converter"))
}

/// Contenu du fichier, ou `None` s'il est absent ou illisible.
pub fn read<T: DeserializeOwned>(path: &Path) -> Option<T> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

/// Un fichier absent ou illisible donne la valeur par défaut : il ne doit jamais empêcher
/// l'application de démarrer.
pub fn load<T: DeserializeOwned + Default>(path: &Path) -> T {
    read(path).unwrap_or_default()
}

pub fn save<T: Serialize + ?Sized>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    // Écriture dans un fichier voisin puis renommage : une coupure en cours d'écriture
    // ne laisse pas un fichier tronqué. Le nom porte l'identifiant du processus pour que
    // deux fenêtres de l'application n'écrivent pas dans le même fichier temporaire
    let mut temporary = path.file_name().unwrap_or_default().to_owned();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = path.with_file_name(temporary);
    fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&temporary, path)
}
