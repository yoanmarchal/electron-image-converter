import { app, BrowserWindow, ipcMain, dialog, protocol, shell, net } from 'electron';
import { join, dirname, resolve, isAbsolute } from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import isDev from 'electron-is-dev';
import Store from 'electron-store';
import sharp from 'sharp';
import {
  INPUT_EXTENSIONS,
  OUTPUT_FORMATS,
  getExtension,
  assertImageFile,
  convertImage,
} from './imageConversion.js';
import squirrelStartup from 'electron-squirrel-startup';


// Configurer la mise à jour automatique et la journalisation
import log from 'electron-log';

// Configuration des journaux
log.transports.file.level = 'info';
log.info('Application starting...');

// Sous Windows, l'installateur Squirrel lance l'application avec --squirrel-* :
// electron-squirrel-startup crée ou supprime les raccourcis, puis on quitte sans ouvrir de fenêtre
if (squirrelStartup) {
  app.quit();
}

const { updateElectronApp, UpdateSourceType } = await import('update-electron-app');

const __dirname = dirname(fileURLToPath(import.meta.url));

const DEV_SERVER_URL = 'http://localhost:5173';
const INDEX_HTML = join(__dirname, '../dist/index.html');

// Le protocole doit être déclaré avant l'événement ready
protocol.registerSchemesAsPrivileged([
  { scheme: 'local-file', privileges: { standard: true, secure: true } },
]);

// Initialize store for app settings
const store = new Store();

// Chemins choisis par l'utilisateur via les boîtes de dialogue natives
const previewablePaths = new Set();
const allowedOutputDirs = new Set();

let mainWindow;

function samePath(a, b) {
  const left = resolve(a);
  const right = resolve(b);
  return process.platform === 'win32'
    ? left.toLowerCase() === right.toLowerCase()
    : left === right;
}

function isAppUrl(url) {
  try {
    const parsed = new URL(url);
    if (isDev) {
      return parsed.origin === DEV_SERVER_URL;
    }
    return parsed.protocol === 'file:' && samePath(fileURLToPath(parsed), INDEX_HTML);
  } catch {
    return false;
  }
}

// N'accepte que les appels venant de la page de l'application, dans la fenêtre principale
function handle(channel, handler) {
  ipcMain.handle(channel, (event, ...args) => {
    const frame = event.senderFrame;
    const isTrusted = mainWindow
      && event.sender === mainWindow.webContents
      && frame
      && frame.parent === null
      && isAppUrl(frame.url);

    if (!isTrusted) {
      log.warn(`IPC "${channel}" refusé pour l'émetteur ${frame?.url ?? 'inconnu'}`);
      throw new Error(`Unauthorized IPC sender for "${channel}"`);
    }
    return handler(...args);
  });
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1000,
    height: 700,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true,
      preload: join(__dirname, 'preload.js'),
      webSecurity: true,
    },
    show: false,
    backgroundColor: '#f8fafc',
  });

  const startUrl = isDev ? DEV_SERVER_URL : pathToFileURL(INDEX_HTML).href;

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
  // Aperçus des images sélectionnées : local-file://preview/<chemin encodé>
  protocol.handle('local-file', (request) => {
    const filePath = decodeURIComponent(new URL(request.url).pathname.slice(1));
    if (!previewablePaths.has(filePath)) {
      log.warn(`Aperçu refusé : ${filePath}`);
      return new Response('Forbidden', { status: 403 });
    }
    return net.fetch(pathToFileURL(filePath).href);
  });
};

// Empêcher la page de quitter l'application ou d'ouvrir d'autres fenêtres
// (par exemple en déposant un fichier .html en dehors de la zone de dépôt)
app.on('web-contents-created', (_, contents) => {
  contents.on('will-navigate', (event, url) => {
    if (!isAppUrl(url)) {
      log.warn(`Navigation bloquée vers ${url}`);
      event.preventDefault();
    }
  });
  contents.setWindowOpenHandler(({ url }) => {
    log.warn(`Ouverture de fenêtre bloquée vers ${url}`);
    return { action: 'deny' };
  });
  contents.on('will-attach-webview', (event) => {
    event.preventDefault();
  });
});

// Initialisation de l'application
app.whenReady().then(() => {
  if (squirrelStartup) {
    return;
  }

  registerProtocols();
  createWindow();

  updateElectronApp({
    logger: log,
    updateSource: {
      type: UpdateSourceType.ElectronPublicUpdateService,
      repo: 'yoanmarchal/electron-image-converter'
    },
    notifyUser: true
  });

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
handle('select-files', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openFile', 'multiSelections'],
    filters: [
      { name: 'Images', extensions: INPUT_EXTENSIONS },
    ],
  });

  if (!result.canceled) {
    // Convertir les chemins en URLs avec notre protocole personnalisé
    return result.filePaths.map(path => {
      previewablePaths.add(path);
      return {
        path,
        previewUrl: `local-file://preview/${encodeURIComponent(path)}`,
      };
    });
  }
  return [];
});

handle('select-output-dir', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openDirectory'],
  });

  if (!result.canceled) {
    allowedOutputDirs.add(result.filePaths[0]);
    return result.filePaths[0];
  }
  return null;
});

handle('get-image-info', async (filePath) => {
  try {
    const fileStat = await assertImageFile(filePath);
    const metadata = await sharp(filePath).metadata();
    return {
      format: metadata.format,
      width: metadata.width,
      height: metadata.height,
      size: fileStat.size,
    };
  } catch (error) {
    console.error('Error getting image info:', error);
    return null;
  }
});

handle('convert-image', async ({ filePath, outputDir, quality, format = 'webp' } = {}) => {
  try {
    // Validation des paramètres d'entrée
    if (!quality || typeof quality !== 'number' || quality < 1 || quality > 100) {
      throw new Error('La qualité doit être un nombre entre 1 et 100');
    }
    if (!OUTPUT_FORMATS.includes(format)) {
      throw new Error(`Format non supporté : ${format}`);
    }
    if (outputDir && (typeof outputDir !== 'string' || !allowedOutputDirs.has(outputDir))) {
      throw new Error('Le dossier de sortie doit être choisi via le sélecteur de dossier');
    }

    console.log(`Début de la conversion : ${filePath} (qualité=${quality}, format=${format}, dossier=${outputDir || "dossier d'origine"})`);

    const result = await convertImage({ filePath, outputDir, quality, format });

    console.log(`Conversion réussie : ${result.outputPath} (${result.originalSize} → ${result.newSize} octets)`);

    return {
      success: true,
      originalPath: filePath,
      ...result,
    };
  } catch (error) {
    console.error('Erreur détaillée de conversion :', error);
    const errorMessage = error.code === 'ENOENT' && error.path === filePath
      ? `Le fichier source n'existe pas : ${filePath}`
      : error.message;

    return {
      success: false,
      error: `Erreur de conversion : ${errorMessage}`,
      originalPath: filePath,
    };
  }
});

handle('save-conversion-history', (historyItem) => {
  const { id, originalPath, outputPath, originalSize, newSize, compressionRatio, timestamp } = historyItem ?? {};
  const hasValidShape = [id, originalPath, outputPath, compressionRatio, timestamp].every(value => typeof value === 'string')
    && [originalSize, newSize].every(value => typeof value === 'number');
  if (!hasValidShape) {
    throw new Error('Entrée d\'historique invalide');
  }

  const history = store.get('conversionHistory', []);
  history.unshift({ id, originalPath, outputPath, originalSize, newSize, compressionRatio, timestamp });
  // Keep only the last 100 records
  if (history.length > 100) {
    history.length = 100;
  }
  store.set('conversionHistory', history);
  return true;
});

handle('get-conversion-history', () => {
  return store.get('conversionHistory', []);
});

handle('clear-conversion-history', () => {
  store.set('conversionHistory', []);
  return true;
});

// Handler pour ouvrir les fichiers convertis avec l'application par défaut
handle('open-file', async (filePath) => {
  // Seulement des images produites par l'application : jamais d'exécutable ni de script
  if (typeof filePath !== 'string' || !isAbsolute(filePath) || !OUTPUT_FORMATS.includes(getExtension(filePath))) {
    log.warn(`Ouverture refusée : ${filePath}`);
    return false;
  }

  // shell.openPath ne lève pas d'exception : il renvoie un message d'erreur, vide en cas de succès
  const error = await shell.openPath(filePath);
  if (error) {
    console.error('Error opening file:', error);
    return false;
  }
  return true;
});
