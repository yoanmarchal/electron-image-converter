const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('electron', {
  ipcRenderer: {
    invoke: (channel, ...args) => {
      const validChannels = [
        'select-files',
        'select-output-dir',
        'get-last-output-dir',
        'get-image-info',
        'convert-image',
        'save-conversion-history',
        'get-conversion-history',
        'clear-conversion-history',
        'open-file',
        'handle-dropped-file'
      ];
      if (validChannels.includes(channel)) {
        return ipcRenderer.invoke(channel, ...args);
      }
      return Promise.reject(new Error(`Channel "${channel}" is not allowed`));
    },
  },
});