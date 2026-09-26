import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { mkdir, mkdtemp, readdir, rm, writeFile } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';
import sharp from 'sharp';
import { assertImageFile, convertImage, getAvailableOutputPath } from './imageConversion.js';

let dir;

beforeEach(async () => {
  dir = await mkdtemp(join(tmpdir(), 'image-converter-test-'));
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

async function createImage(name, { width = 20, height = 20, background = '#3366cc', channels = 3 } = {}) {
  const filePath = join(dir, name);
  await sharp({ create: { width, height, channels, background } }).toFile(filePath);
  return filePath;
}

describe('convertImage', () => {
  it.each(['webp', 'jpg', 'png', 'avif'])('convertit en %s à côté de l\'original', async (format) => {
    const input = await createImage('photo.tiff');

    const result = await convertImage({ filePath: input, outputDir: '', quality: 80, format });

    expect(result.outputPath).toBe(join(dir, `photo.${format}`));
    expect(result.newSize).toBeGreaterThan(0);
    const { format: actual } = await sharp(result.outputPath).metadata();
    expect(actual).toBe({ jpg: 'jpeg', avif: 'heif' }[format] ?? format);
  });

  it('écrit dans le dossier de sortie demandé', async () => {
    const input = await createImage('photo.png');
    const outputDir = join(dir, 'sortie');
    await mkdir(outputDir);

    const result = await convertImage({ filePath: input, outputDir, quality: 80, format: 'webp' });

    expect(result.outputPath).toBe(join(outputDir, 'photo.webp'));
  });

  it('applique l\'orientation EXIF aux pixels', async () => {
    const input = join(dir, 'portrait.jpg');
    await sharp({ create: { width: 60, height: 40, channels: 3, background: '#3366cc' } })
      .jpeg()
      .withMetadata({ orientation: 6 })
      .toFile(input);

    const { outputPath } = await convertImage({ filePath: input, outputDir: '', quality: 80, format: 'webp' });

    const { width, height } = await sharp(outputPath).metadata();
    expect({ width, height }).toEqual({ width: 40, height: 60 });
  });

  it('remplace la transparence par du blanc en JPEG', async () => {
    const input = await createImage('transparent.png', { channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } });

    const { outputPath } = await convertImage({ filePath: input, outputDir: '', quality: 90, format: 'jpg' });

    const { data } = await sharp(outputPath).raw().toBuffer({ resolveWithObject: true });
    expect([...data.subarray(0, 3)].every(value => value > 245)).toBe(true);
  });

  it('conserve la transparence en PNG', async () => {
    const input = await createImage('transparent.png', { channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } });

    const { outputPath } = await convertImage({ filePath: input, outputDir: '', quality: 80, format: 'png' });

    expect(outputPath).toBe(join(dir, 'transparent-1.png'));
    expect((await sharp(outputPath).metadata()).hasAlpha).toBe(true);
  });

  describe('GIF animé', () => {
    let input;

    beforeEach(async () => {
      const frames = await Promise.all(['#ff0000', '#00ff00', '#0000ff'].map(color =>
        sharp({ create: { width: 16, height: 16, channels: 3, background: color } }).png().toBuffer()
      ));
      input = join(dir, 'anim.gif');
      await sharp(frames, { join: { animated: true } }).gif().toFile(input);
    });

    it('garde l\'animation en WebP', async () => {
      const { outputPath } = await convertImage({ filePath: input, outputDir: '', quality: 80, format: 'webp' });

      expect((await sharp(outputPath, { animated: true }).metadata()).pages).toBe(3);
    });

    it('garde la première image pour les autres formats', async () => {
      const { outputPath } = await convertImage({ filePath: input, outputDir: '', quality: 80, format: 'png' });

      const { width, height, pages } = await sharp(outputPath).metadata();
      expect({ width, height, pages: pages ?? 1 }).toEqual({ width: 16, height: 16, pages: 1 });
    });
  });

  it('n\'écrase jamais l\'original ni une conversion précédente', async () => {
    const input = await createImage('photo.png');
    const originalBytes = (await sharp(input).raw().toBuffer()).length;

    const first = await convertImage({ filePath: input, outputDir: '', quality: 80, format: 'png' });
    const second = await convertImage({ filePath: input, outputDir: '', quality: 80, format: 'png' });

    expect(first.outputPath).toBe(join(dir, 'photo-1.png'));
    expect(second.outputPath).toBe(join(dir, 'photo-2.png'));
    expect((await sharp(input).raw().toBuffer()).length).toBe(originalBytes);
    expect((await readdir(dir)).sort()).toEqual(['photo-1.png', 'photo-2.png', 'photo.png']);
  });

  it('refuse un format de sortie inconnu', async () => {
    const input = await createImage('photo.png');

    await expect(convertImage({ filePath: input, outputDir: '', quality: 80, format: 'exe' }))
      .rejects.toThrow('Format non supporté');
  });
});

describe('assertImageFile', () => {
  it('refuse un chemin relatif', async () => {
    await expect(assertImageFile('photo.png')).rejects.toThrow('invalide');
  });

  it('refuse une extension non lisible par sharp', async () => {
    const bmp = join(dir, 'image.bmp');
    await writeFile(bmp, 'BM');

    await expect(assertImageFile(bmp)).rejects.toThrow('Extension non supportée : .bmp');
  });

  it('signale un fichier absent', async () => {
    await expect(assertImageFile(join(dir, 'absent.png'))).rejects.toMatchObject({ code: 'ENOENT' });
  });
});

describe('getAvailableOutputPath', () => {
  it('ajoute un suffixe tant que le nom est pris', async () => {
    await writeFile(join(dir, 'photo.webp'), '');
    await writeFile(join(dir, 'photo-1.webp'), '');

    expect(await getAvailableOutputPath(dir, 'photo', 'webp')).toBe(join(dir, 'photo-2.webp'));
  });
});
