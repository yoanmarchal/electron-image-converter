/**
 * Formats a file size in bytes to a human-readable string
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 octet';

  const k = 1024;
  const sizes = ['octets', 'Ko', 'Mo', 'Go'];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)).toLocaleString('fr-FR') + ' ' + sizes[i];
}

/**
 * Formats a compression ratio such as "91.38" as "91,38 %"
 */
export function formatPercent(ratio: string): string {
  return parseFloat(ratio).toLocaleString('fr-FR') + ' %';
}

/**
 * Extracts the filename from a path (Windows or POSIX separators)
 */
export function getFilenameFromPath(path: string): string {
  return path.split(/[\\/]/).pop() || 'inconnu';
}
