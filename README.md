# Image Converter

Une application de bureau moderne et efficace pour la conversion d'images, construite avec Electron et React.

[![Build and Release](https://github.com/yoanmarchal/electron-image-converter/actions/workflows/release.yml/badge.svg)](https://github.com/yoanmarchal/electron-image-converter/actions/workflows/release.yml)

## Fonctionnalités

- 🖼️ **Conversion d'images multi-formats**
  - Formats de sortie : WebP (meilleur rapport qualité/taille), JPEG (compatible partout), PNG (sans perte, avec transparence), AVIF (format moderne haute performance)
  - Formats d'entrée : JPEG, PNG, GIF, TIFF, WebP, AVIF
  - Orientation EXIF appliquée : les photos prises en portrait restent en portrait
  - GIF animés conservés en WebP animé
  - Transparence remplacée par un fond blanc en JPEG

- 🎯 **Interface utilisateur intuitive**
  - Glisser-déposer des images
  - Prévisualisation des images
  - Suivi en temps réel de la conversion
  - Mode sombre/clair
  - Interface responsive

- ⚙️ **Options de conversion avancées**
  - Contrôle de la qualité (1 à 100 %, sauf PNG qui est sans perte)
  - Fichiers convertis à côté des originaux, ou dans un dossier de sortie au choix
  - Aucun fichier écrasé : si `photo.webp` existe déjà, la sortie devient `photo-1.webp`

- 📊 **Historique des conversions**
  - Suivi des conversions effectuées
  - Statistiques de compression
  - Accès rapide aux fichiers convertis

- 🔄 **Mises à jour automatiques** (Windows)
  - Système de mise à jour intégré
  - Notifications de nouvelles versions
  - Sous Linux, installez la nouvelle version depuis les [releases GitHub](https://github.com/yoanmarchal/electron-image-converter/releases)

## Technologies utilisées

- Electron
- React
- TypeScript
- TailwindCSS
- Sharp (pour le traitement d'images)
- Vite

## Installation

Prérequis : Node.js 22.12 ou plus récent (requis par Electron 44).

```bash
# Cloner le repository
git clone https://github.com/yoanmarchal/electron-image-converter.git

# Installer les dépendances (versions exactes du package-lock.json)
npm ci
```

Le binaire d'Electron est téléchargé au premier lancement de `npm run electron:dev`.

## Commandes disponibles

- `npm run dev` - Lance l'application en mode développement
- `npm run electron:dev` - Lance l'application Electron en mode développement
- `npm run build` - Compile l'application
- `npm run electron:build` - Compile l'application Electron pour la distribution
- `npm run release` - Crée une nouvelle release de l'application
- `npm run icons` - Génère les icônes de l'application
- `npm run lint` - Vérifie le code avec ESLint (renderer et processus principal)
- `npm test` - Lance les tests (Vitest), dont les tests de conversion avec sharp

## Configuration système requise

### Windows
- Windows 10 ou plus récent
- Architecture x64

### Linux
- Ubuntu 18.04 ou plus récent, ou distribution équivalente
- Architecture x64 ou arm64

## License

[MIT](LICENSE) © Yoan Marchal
