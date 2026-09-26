import { describe, expect, it } from 'vitest';
import { formatFileSize, formatPercent, getFilenameFromPath } from './fileUtils';

describe('formatFileSize', () => {
  it('formate les tailles avec des unités françaises', () => {
    expect(formatFileSize(0)).toBe('0 octet');
    expect(formatFileSize(512)).toBe('512 octets');
    expect(formatFileSize(1536)).toBe('1,5 Ko');
    expect(formatFileSize(5 * 1024 * 1024)).toBe('5 Mo');
  });

  it('plafonne à l\'unité la plus grande', () => {
    expect(formatFileSize(2 * 1024 ** 4)).toBe('2\u202f048 Go');
  });
});

describe('formatPercent', () => {
  it('formate le gain à la française', () => {
    expect(formatPercent('91.38')).toBe('91,38 %');
    expect(formatPercent('-12.50')).toBe('-12,5 %');
  });
});

describe('getFilenameFromPath', () => {
  it('extrait le nom d\'un chemin Windows', () => {
    expect(getFilenameFromPath('C:\\Users\\Yoan\\Images\\photo.png')).toBe('photo.png');
  });

  it('extrait le nom d\'un chemin POSIX', () => {
    expect(getFilenameFromPath('/home/yoan/photo.jpg')).toBe('photo.jpg');
  });

  it('renvoie une valeur par défaut pour un chemin vide', () => {
    expect(getFilenameFromPath('')).toBe('inconnu');
  });
});
