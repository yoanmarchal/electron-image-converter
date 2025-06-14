import { app, BrowserWindow, ipcMain, dialog, protocol, shell } from 'electron';
import { join, basename, extname, dirname } from 'path';
import { fileURLToPath } from 'url';
import isDev from 'electron-is-dev';
import { statSync } from 'fs';
import Store from 'electron-store';
import { promises } from 'fs';
import sharp from 'sharp';

// Configurer la mise à jour automatique et la journalisation
import log from 'electron-log';

// Configuration des journaux
log.transports.file.level = 'info';
log.info('Application starting...');

const { updateElectronApp } = await import('update-electron-app');
updateElectronApp({
  logger: log,
  repo: 'yoanmarchal/electron-image-converter',
  updateInterval: '1 hour',
  notifyUser: true
});

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
      additionalArguments: ['--enable-features=NativeFileSystemAPI'],
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
    return result.filePaths[0];
  }
  return null;
});

ipcMain.handle('get-image-info', async (_, filePath) => {
  try {
    const metadata = await sharp(filePath).metadata();
    return {
      format: metadata.format,
      width: metadata.width,
      height: metadata.height,
      size: statSync(filePath).size,
    };
  } catch (error) {
    console.error('Error getting image info:', error);
    return null;
  }
});

ipcMain.handle('convert-image', async (_, { filePath, outputDir, quality, format = 'webp' }) => {
  try {
    // Validation des paramètres d'entrée
    if (!filePath || typeof filePath !== 'string' || filePath.trim() === '') {
      throw new Error('Le chemin du fichier source est invalide ou manquant');
    }
    if (!quality || typeof quality !== 'number' || quality < 1 || quality > 100) {
      throw new Error('La qualité doit être un nombre entre 1 et 100');
    }

    console.log(`Début de la conversion : ${filePath}`);
    
    // Toujours utiliser le dossier d'origine si aucun dossier de sortie n'est spécifié
    const effectiveOutputDir = outputDir?.trim() || dirname(filePath);
    console.log(`Paramètres : qualité=${quality}, format=${format}, dossier de sortie=${effectiveOutputDir}`);
    
    const filename = basename(filePath, extname(filePath));
    const outputPath = join(effectiveOutputDir, `${filename}.${format}`);
    
    console.log(`Chemin de sortie : ${outputPath}`);
    
    // Utiliser sharp pour la conversion selon le format choisi
    let sharpInstance = sharp(filePath);
    
    switch (format) {
      case 'webp':
        sharpInstance = sharpInstance.webp({ quality });
        break;
      case 'jpg':
        sharpInstance = sharpInstance.jpeg({ quality });
        break;
      case 'png':
        sharpInstance = sharpInstance.png({ quality });
        break;
      case 'avif':
        sharpInstance = sharpInstance.avif({ quality });
        break;
      default:
        throw new Error(`Format non supporté : ${format}`);
    }
    
    await sharpInstance.toFile(outputPath);
    
    // Vérifier si le fichier de sortie existe et a une taille
    const outputExists = statSync(outputPath);
    if (!outputExists || outputExists.size === 0) {
      throw new Error('Le fichier de sortie est vide ou n\'existe pas');
    }
    
    const originalSize = statSync(filePath).size;
    const newSize = outputExists.size;
    
    console.log(`Conversion réussie : ${outputPath}`);
    console.log(`Taille originale : ${originalSize}, Nouvelle taille : ${newSize}`);
    
    return {
      success: true,
      originalPath: filePath,
      outputPath,
      originalSize,
      newSize,
      compressionRatio: ((1 - (newSize / originalSize)) * 100).toFixed(2),
    };
  } catch (error) {
    console.error('Erreur détaillée de conversion :', error);
    console.error('Stack trace :', error.stack);
    let errorMessage = error.message;
    
    // Vérifier si le fichier source existe
    try {
      statSync(filePath);
    } catch (e) {
      errorMessage = `Le fichier source n'existe pas : ${filePath}`;
    }
    
    return {
      success: false,
      error: `Erreur de conversion : ${errorMessage}`,
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

ipcMain.handle('handle-dropped-file', async (_, { buffer, name }) => {
  try {
    const tempPath = join(app.getPath('temp'), `webp-converter-${Date.now()}-${name}`);
    const fileBuffer = Buffer.from(buffer);
    await promises.writeFile(tempPath, fileBuffer);
    return tempPath;
  } catch (error) {
    console.error('Error handling dropped file:', error);
    throw error;
  }
});