// Conversion d'images avec sharp, sans dépendance à Electron (testable avec Vitest)
import { join, basename, extname, dirname, isAbsolute } from 'path';
import { access, stat } from 'fs/promises';
import sharp from 'sharp';

// Le cache de libvips garde les fichiers ouverts : sous Windows, les images converties
// restaient verrouillées (impossible de les supprimer ou renommer tant que l'app tourne)
sharp.cache(false);

// Formats lisibles par sharp (le BMP ne l'est pas sans ImageMagick)
export const INPUT_EXTENSIONS = ['jpg', 'jpeg', 'png', 'gif', 'tif', 'tiff', 'webp', 'avif'];
export const OUTPUT_FORMATS = ['webp', 'jpg', 'png', 'avif'];

export function getExtension(filePath) {
  return extname(filePath).slice(1).toLowerCase();
}

export async function assertImageFile(filePath) {
  if (typeof filePath !== 'string' || !isAbsolute(filePath)) {
    throw new Error('Le chemin du fichier source est invalide ou manquant');
  }
  if (!INPUT_EXTENSIONS.includes(getExtension(filePath))) {
    throw new Error(`Extension non supportée : ${extname(filePath) || '(aucune)'}`);
  }
  const fileStat = await stat(filePath);
  if (!fileStat.isFile()) {
    throw new Error(`Le chemin ne désigne pas un fichier : ${filePath}`);
  }
  return fileStat;
}

async function fileExists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

// Ne jamais écraser un fichier existant (ni l'original) : photo.webp, puis photo-1.webp, photo-2.webp…
export async function getAvailableOutputPath(dir, name, extension) {
  for (let index = 0; ; index++) {
    const suffix = index === 0 ? '' : `-${index}`;
    const candidate = join(dir, `${name}${suffix}.${extension}`);
    if (!(await fileExists(candidate))) {
      return candidate;
    }
  }
}

/**
 * Convertit une image. Les paramètres doivent avoir été validés par l'appelant
 * (format dans OUTPUT_FORMATS, qualité entre 1 et 100, dossier de sortie autorisé).
 * Sans dossier de sortie, le fichier est écrit à côté de l'original.
 */
export async function convertImage({ filePath, outputDir, quality, format }) {
  const { size: originalSize } = await assertImageFile(filePath);

  const effectiveOutputDir = outputDir || dirname(filePath);
  const filename = basename(filePath, extname(filePath));
  const outputPath = await getAvailableOutputPath(effectiveOutputDir, filename, format);

  // Seul le WebP conserve l'animation d'un GIF ; les autres formats gardent la première image
  const keepAnimation = format === 'webp' && getExtension(filePath) === 'gif';

  // Les métadonnées EXIF ne sont pas copiées : appliquer l'orientation aux pixels
  // pour que les photos prises en portrait ne ressortent pas couchées
  let sharpInstance = sharp(filePath, { animated: keepAnimation }).autoOrient();

  switch (format) {
    case 'webp':
      sharpInstance = sharpInstance.webp({ quality });
      break;
    case 'jpg':
      // Le JPEG n'a pas de transparence : fond blanc plutôt que noir
      sharpInstance = sharpInstance.flatten({ background: '#ffffff' }).jpeg({ quality });
      break;
    case 'png':
      // PNG sans perte : la qualité ne s'applique pas, on compresse au maximum
      sharpInstance = sharpInstance.png({ compressionLevel: 9 });
      break;
    case 'avif':
      sharpInstance = sharpInstance.avif({ quality });
      break;
    default:
      throw new Error(`Format non supporté : ${format}`);
  }

  await sharpInstance.toFile(outputPath);

  const { size: newSize } = await stat(outputPath);
  if (newSize === 0) {
    throw new Error('Le fichier de sortie est vide');
  }

  return {
    outputPath,
    originalSize,
    newSize,
    compressionRatio: ((1 - (newSize / originalSize)) * 100).toFixed(2),
  };
}
