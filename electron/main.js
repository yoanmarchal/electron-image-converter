import { app, BrowserWindow, ipcMain, dialog, protocol, shell } from 'electron';
import { join, basename, extname, dirname } from 'path';
import { fileURLToPath } from 'url';
import isDev from 'electron-is-dev';
import { statSync } from 'fs';
import { promisify } from 'util';
import Store from 'electron-store';
import { exec } from 'child_process';

const execAsync = promisify(exec);
const __dirname = dirname(fileURLToPath(import.meta.url));

// Initialize store for app settings
const store = new Store();

let mainWindow;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1000,
    height: 700,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: join(__dirname, 'preload.js'),
      webSecurity: true,
    },
    show: false,
    backgroundColor: '#f8fafc',
  });

  const startUrl = isDev
    ? 'http://localhost:5173'
    : `file://${join(__dirname, '../dist/index.html')}`;

  mainWindow.loadURL(startUrl);

  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });

  // Open DevTools if in dev mode
  if (isDev) {
    mainWindow.webContents.openDevTools({ mode: 'detach' });
  }
}

// Configurer le protocole personnalisé
const registerProtocols = () => {
  // Protocole pour les fichiers locaux
  protocol.registerFileProtocol('local-file', (request, callback) => {
    const url = decodeURIComponent(request.url.slice('local-file://'.length));
    try {
      callback({ path: url });
    } catch (error) {
      console.error('Protocol error:', error);
      callback({ error: -2 });
    }
  });
};

// Initialisation de l'application
app.whenReady().then(() => {
  registerProtocols();
  createWindow();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});

// IPC handlers for image conversion
ipcMain.handle('select-files', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openFile', 'multiSelections'],
    filters: [
      { name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'tiff'] },
    ],
  });

  if (!result.canceled) {
    // Convertir les chemins en URLs avec notre protocole personnalisé
    return result.filePaths.map(path => ({
      path,
      previewUrl: `local-file://${path}`,
    }));
  }
  return [];
});

ipcMain.handle('select-output-dir', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openDirectory'],
  });

  if (!result.canceled) {
    store.set('lastOutputDir', result.filePaths[0]);
    return result.filePaths[0];
  }
  return null;
});

ipcMain.handle('get-last-output-dir', () => {
  return store.get('lastOutputDir', '');
});

ipcMain.handle('get-image-info', async (_, filePath) => {
  try {
    const { stdout } = await execAsync(`magick identify -format "%w %h %m" "${filePath}"`);
    const [width, height, format] = stdout.trim().split(' ');
    return {
      format,
      width: parseInt(width),
      height: parseInt(height),
      size: statSync(filePath).size,
    };
  } catch (error) {
    console.error('Error getting image info:', error);
    return null;
  }
});

ipcMain.handle('convert-image', async (_, { filePath, outputDir, quality }) => {
  try {
    const filename = basename(filePath, extname(filePath));
    const outputPath = join(outputDir, `${filename}.webp`);
    
    await execAsync(`magick "${filePath}" -quality ${quality} "${outputPath}"`);
    
    const originalSize = statSync(filePath).size;
    const newSize = statSync(outputPath).size;
    
    return {
      success: true,
      originalPath: filePath,
      outputPath,
      originalSize,
      newSize,
      compressionRatio: ((1 - (newSize / originalSize)) * 100).toFixed(2),
    };
  } catch (error) {
    console.error('Error converting image:', error);
    return {
      success: false,
      error: error.message,
      originalPath: filePath,
    };
  }
});

ipcMain.handle('save-conversion-history', (_, historyItem) => {
  const history = store.get('conversionHistory', []);
  history.unshift(historyItem);
  // Keep only the last 100 records
  if (history.length > 100) {
    history.length = 100;
  }
  store.set('conversionHistory', history);
  return true;
});

ipcMain.handle('get-conversion-history', () => {
  return store.get('conversionHistory', []);
});

ipcMain.handle('clear-conversion-history', () => {
  store.set('conversionHistory', []);
  return true;
});

// Handler pour ouvrir les fichiers dans l'explorateur
ipcMain.handle('open-file', async (_, filePath) => {
  try {
    await shell.openPath(filePath);
    return true;
  } catch (error) {
    console.error('Error opening file:', error);
    return false;
  }
});