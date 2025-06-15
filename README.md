# Image Converter

Une application de bureau moderne et efficace pour la conversion d'images, construite avec Electron et React.

[![Build and Release](https://github.com/yoanmarchal/electron-image-converter/actions/workflows/release.yml/badge.svg)](https://github.com/yoanmarchal/electron-image-converter/actions/workflows/release.yml)

## Fonctionnalités

- 🖼️ **Conversion d'images multi-formats**
  - WebP (Meilleur rapport qualité/taille)
  - JPEG (Compatible partout)
  - PNG (Sans perte avec transparence)
  - AVIF (Format moderne haute performance)

- 🎯 **Interface utilisateur intuitive**
  - Glisser-déposer des images
  - Prévisualisation des images
  - Suivi en temps réel de la conversion
  - Mode sombre/clair
  - Interface responsive

- ⚙️ **Options de conversion avancées**
  - Contrôle de la qualité (0-100%)
  - Choix du dossier de sortie
  - Conservation de la structure des dossiers

- 📊 **Historique des conversions**
  - Suivi des conversions effectuées
  - Statistiques de compression
  - Accès rapide aux fichiers convertis

- 🔄 **Mises à jour automatiques**
  - Système de mise à jour intégré
  - Notifications de nouvelles versions

## Technologies utilisées

- Electron
- React
- TypeScript
- TailwindCSS
- Sharp (pour le traitement d'images)
- Vite

## Installation

```bash
# Cloner le repository
git clone https://github.com/yoanmarchal/electron-image-converter.git

# Installer les dépendances
npm install
```

## Commandes disponibles

- `npm run dev` - Lance l'application en mode développement
- `npm run electron:dev` - Lance l'application Electron en mode développement
- `npm run build` - Compile l'application
- `npm run electron:build` - Compile l'application Electron pour la distribution
- `npm run release` - Crée une nouvelle release de l'application
- `npm run icons` - Génère les icônes de l'application
- `npm run lint` - Vérifie le code avec ESLint

## Configuration système requise

### Windows
- Windows 10 ou plus récent
- Architecture x64

### Linux
- Ubuntu 18.04 ou plus récent, ou distribution équivalente
- Architecture x64 ou arm64

## License

[MIT](LICENSE) © Yoan Marchal
