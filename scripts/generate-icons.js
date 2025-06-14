import sharp from 'sharp';
import { existsSync, mkdirSync } from 'fs';
import { join } from 'path';

async function generateIcons() {
  // Tailles d'icônes requises pour Windows
  const sizes = [16, 24, 32, 48, 64, 128, 256];
  
  try {
    // Créer le dossier build s'il n'existe pas
    const buildDir = join('assets', 'icons');
    if (!existsSync(buildDir)) {
      mkdirSync(buildDir, { recursive: true });
    }

    // Générer PNG pour chaque taille
    for (const size of sizes) {
      await sharp('public/logo.svg')
        .resize(size, size)
        .png()
        .toFile(join(buildDir, `${size}x${size}.png`));
      console.log(`✓ Généré ${size}x${size}.png`);
    }

    console.log('✓ Génération des icônes terminée avec succès');
  } catch (error) {
    console.error('❌ Erreur lors de la génération des icônes:', error);
    process.exit(1);
  }
}

generateIcons();
