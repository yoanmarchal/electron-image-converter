const Jimp = require('jimp');
const fs = require('fs');
const path = require('path');

async function generateIcons() {
  // Tailles d'icônes requises pour Windows
  const sizes = [16, 24, 32, 48, 64, 128, 256];
  
  try {
    const image = await Jimp.read('public/logo.svg');
    
    // Créer le dossier build s'il n'existe pas
    const buildDir = path.join('assets', 'icons');
    if (!fs.existsSync(buildDir)) {
      fs.mkdirSync(buildDir, { recursive: true });
    }

    // Générer PNG pour chaque taille
    for (const size of sizes) {
      await image
        .clone()
        .resize(size, size)
        .writeAsync(path.join(buildDir, `${size}x${size}.png`));
    }

    console.log('Icons generated successfully!');
  } catch (err) {
    console.error('Error generating icons:', err);
    process.exit(1);
  }
}

generateIcons();
