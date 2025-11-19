/// <reference types="vite/client" />

interface Window {
  electron: {
    ipcRenderer: {
      invoke<T = unknown>(channel: string, ...args: unknown[]): Promise<T>;
    }
  }
}
