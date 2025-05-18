import { app, BrowserWindow } from 'electron';

// ES modules don't have __dirname by default. We need to derive it.
// const __filename = fileURLToPath(import.meta.url);
// const __dirname = path.dirname(__filename);

app.whenReady().then(() => {
  console.log('[MinimalMain E2E] App ready, creating window...');
  const win = new BrowserWindow({
    show: false,
    webPreferences: {
      nodeIntegration: false, // Default and good practice
      contextIsolation: true, // Default and good practice
      sandbox: false, // Matching your app's setting for now
      // preload: path.join(__dirname, '../packages/preload/dist/index.cjs'), // Path using __dirname
      preload: '/Users/wings/projects/flagship/packages/preload/dist/index.cjs', // Hardcoded absolute path
    },
  });

  win.webContents.on('did-finish-load', () => {
    console.log('[MinimalMain E2E] Window finished loading content (about:blank).');
  });

  win.webContents.on('console-message', (event, level, message, line, sourceId) => {
    console.log(`[MinimalMain E2E - Renderer Console] ${message} (Source: ${sourceId}:${line})`);
  });

  win.on('ready-to-show', () => {
    console.log('[MinimalMain E2E] Window ready-to-show, showing window.');
    win.show();
  });

  win.loadURL('about:blank');
  console.log('[MinimalMain E2E] Window created, loadURL(\'about:blank\') called.');
});

app.on('window-all-closed', () => {
  console.log('[MinimalMain E2E] All windows closed.');
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('quit', () => {
  console.log('[MinimalMain E2E] App quitting.');
});
