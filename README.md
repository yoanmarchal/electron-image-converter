# Image Converter

Une application de bureau pour convertir, compresser et redimensionner des images par lot, écrite en Rust avec [Iced](https://iced.rs).

[![Build and Release](https://github.com/yoanmarchal/electron-image-converter/actions/workflows/release.yml/badge.svg)](https://github.com/yoanmarchal/electron-image-converter/actions/workflows/release.yml)

> **Vous venez de la version 1 (Electron) ?** La version 2 est une réécriture complète : elle ne se met pas à jour depuis la 1. Désinstallez l'ancienne version, puis installez la nouvelle depuis les [releases GitHub](https://github.com/yoanmarchal/electron-image-converter/releases). L'historique des conversions n'est pas repris.

## Fonctionnalités

- 🖼️ **Conversion d'images multi-formats**
  - Formats de sortie : WebP (meilleur rapport qualité/taille), JPEG (compatible partout), PNG (sans perte, avec transparence), AVIF (format moderne haute performance)
  - « Format d'origine » pour seulement redimensionner ou recompresser (les GIF et TIFF sont alors enregistrés en PNG)
  - Formats d'entrée : JPEG, PNG, GIF, TIFF, WebP
  - Orientation EXIF appliquée : les photos prises en portrait restent en portrait
  - GIF animés conservés en WebP animé
  - Transparence remplacée par un fond blanc en JPEG

- 📐 **Redimensionnement par lot** (optionnel)
  - Par largeur, hauteur, côté le plus long ou pourcentage, toujours à proportions conservées
  - Option « Ne pas agrandir les images plus petites », cochée par défaut
  - Dimensions finales de chaque image affichées avant la conversion

- ⚙️ **Options de conversion**
  - Contrôle de la qualité (1 à 100 %, sauf PNG qui est sans perte)
  - Fichiers convertis à côté des originaux, ou dans un dossier de sortie au choix
  - Aucun fichier écrasé : si `photo.webp` existe déjà, la sortie devient `photo-1.webp`

- 🎯 **Interface**
  - Glisser-déposer des images, ou ouverture depuis l'Explorateur (« Ouvrir avec »)
  - Miniatures, suivi de chaque conversion, gain de poids
  - Mode sombre/clair

- 📊 **Historique des conversions**
  - Les 100 dernières conversions, avec poids et dimensions avant et après
  - Accès rapide aux fichiers convertis

Il n'y a pas de mise à jour automatique pour l'instant : installez les nouvelles versions depuis les [releases GitHub](https://github.com/yoanmarchal/electron-image-converter/releases).

## Technologies utilisées

- Rust
- [Iced](https://iced.rs) pour l'interface, sans navigateur embarqué
- [`image`](https://crates.io/crates/image), [`webp`](https://crates.io/crates/webp), [`jpeg-encoder`](https://crates.io/crates/jpeg-encoder) et [`ravif`](https://crates.io/crates/ravif) pour les codecs
- [`fast_image_resize`](https://crates.io/crates/fast_image_resize) pour le redimensionnement
- [`cargo-packager`](https://crates.io/crates/cargo-packager) pour les installeurs

## Développement

Prérequis : [Rust](https://rustup.rs) stable (édition 2024, donc Rust 1.85 ou plus récent). Sous Windows, les outils de build C++ de Visual Studio sont nécessaires pour compiler libwebp.

```bash
git clone https://github.com/yoanmarchal/electron-image-converter.git
cd electron-image-converter
cargo run --release
```

## Commandes disponibles

- `cargo run --release` - Lance l'application
- `cargo test` - Lance les tests ; le rendu de chaque écran est aussi écrit dans `target/captures/`
- `cargo fmt` et `cargo clippy --all-targets` - Formate et vérifie le code
- `cargo packager --release --formats nsis` - Construit l'installeur Windows dans `release/`
- `cargo packager --release --formats deb,appimage` - Construit les paquets Linux dans `release/`
- `cargo run --release --example compare -- <dossier de sortie> <images…>` - Mesure le temps et le poids de sortie de chaque format

Les deux commandes de packaging demandent `cargo install cargo-packager --locked`.

## Organisation du code

- `src/conversion.rs` - Conversion et redimensionnement, sans dépendance à l'interface
- `src/history.rs`, `src/preferences.rs`, `src/storage.rs` - Données enregistrées dans le dossier de l'utilisateur
- `src/format.rs` - Mise en forme française des tailles, pourcentages et dates
- `src/ui/` - Interface Iced : état et messages (`mod.rs`), thème (`theme.rs`), écrans (`convert.rs`, `history.rs`)
- `tests/conversion.rs` - Tests de conversion sur de vraies images

## Publier une version

Modifiez `version` dans `Cargo.toml` et fusionnez sur `main` : le workflow crée le tag `vX.Y.Z`, la release GitHub et y dépose les installeurs Windows et Linux.

## Configuration système requise

### Windows
- Windows 10 ou plus récent
- Architecture x64

### Linux
- Ubuntu 22.04 ou plus récent, ou distribution équivalente
- Architecture x64 ou arm64

## License

[MIT](LICENSE) © Yoan Marchal. La police [Inter](https://rsms.me/inter/) est distribuée sous licence SIL Open Font License (`assets/fonts/Inter-LICENSE.txt`).
