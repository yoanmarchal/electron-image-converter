// Préférences d'affichage conservées d'un lancement à l'autre
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::storage;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub dark_mode: bool,
}

impl Preferences {
    pub fn load(path: &Path) -> Self {
        storage::load(path)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        storage::save(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reloads_saved_preferences() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences.json");

        assert_eq!(Preferences::load(&path), Preferences { dark_mode: false });

        Preferences { dark_mode: true }.save(&path).unwrap();

        assert_eq!(Preferences::load(&path), Preferences { dark_mode: true });
    }
}
