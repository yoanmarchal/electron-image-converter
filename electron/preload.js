const { contextBridge, ipcRenderer, webUtils } = require('electron');

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('electron', {
  ipcRenderer: {
    invoke: (channel, ...args) => {
      const validChannels = [
        'select-files',
        'select-output-dir',
        'get-image-info',
        'convert-image',
        'save-conversion-history',
        'get-conversion-history',
        'clear-conversion-history',
        'open-file',
      ];
      if (validChannels.includes(channel)) {
        return ipcRenderer.invoke(channel, ...args);
      }
      return Promise.reject(new Error(`Channel "${channel}" is not allowed`));
    },
  },
  // Chemin réel sur le disque d'un fichier glissé-déposé (File.path n'existe plus depuis Electron 32)
  getPathForFile: (file) => webUtils.getPathForFile(file),
});
