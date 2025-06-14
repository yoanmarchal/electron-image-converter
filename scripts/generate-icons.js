import { read } from 'jimp';
import { existsSync, mkdirSync } from 'fs';
import { join } from 'path';

async function generateIcons() {
  // Tailles d'icônes requises pour Windows
  const sizes = [16, 24, 32, 48, 64, 128, 256];
  
  try {
    const image = await read('public/logo.svg');
    
    // Créer le dossier build s'il n'existe pas
    const buildDir = join('assets', 'icons');
    if (!existsSync(buildDir)) {
      mkdirSync(buildDir, { recursive: true });
    }

    // Générer PNG pour chaque taille
    for (const size of sizes) {
      await image
        .clone()
        .resize(size, size)
        .writeAsync(join(buildDir, `${size}x${size}.png`));
    }

    console.log('Icons generated successfully!');
  } catch (err) {
    console.error('Error generating icons:', err);
    process.exit(1);
  }
}

generateIcons();
