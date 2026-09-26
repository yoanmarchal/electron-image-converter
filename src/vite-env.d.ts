/// <reference types="vite/client" />

declare const __APP_VERSION__: string;

interface Window {
  electron: {
    ipcRenderer: {
      invoke<T = unknown>(channel: string, ...args: unknown[]): Promise<T>;
    };
    getPathForFile(file: File): string;
  }
}
