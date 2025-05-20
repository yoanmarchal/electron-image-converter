import { v4 as uuidv4 } from 'uuid';

/**
 * Formats a file size in bytes to a human-readable string
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Generates a unique ID
 */
export function generateId(): string {
  return uuidv4();
}

/**
 * Extracts the filename from a path
 */
export function getFilenameFromPath(path: string): string {
  return path.split('/').pop() || path.split('\\').pop() || 'unknown';
}