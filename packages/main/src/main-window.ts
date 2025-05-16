import {app, BrowserWindow} from 'electron';
import {dirname, join} from 'node:path';
import {fileURLToPath, URL as NodeURL} from 'node:url';
import {connectHttp} from './http';
import {connectFileSystem} from './file-system';
import {startPeerbitNode} from './peerbit-node';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function createWindow() {
  const preloadScriptPath = join(__dirname, '../../preload/dist/index.cjs');
  const browserWindow = new BrowserWindow({
    show: false,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: false,
      webviewTag: false,
      preload: preloadScriptPath,
    },
  });

  if (browserWindow.webContents && typeof browserWindow.webContents.on === 'function') {
    browserWindow.webContents.on('did-fail-load',
      (_event, errorCode, errorDescription, validatedURL, isMainFrame) => {
      console.error(`[MainWindow] WebContents did-fail-load: ${errorDescription} (URL: ${validatedURL}, Code: ${errorCode}, isMainFrame: ${isMainFrame})`);
    });

    browserWindow.webContents.on('preload-error', (_event, path, error) => {
      console.error(`[MainWindow] WebContents preload-error: Path: ${path}, Error: ${error.name} - ${error.message}`);
    });
  }

  browserWindow.on('ready-to-show', () => {
    browserWindow?.show();
    if (import.meta.env.DEV) {
      browserWindow?.webContents.openDevTools();
    }
  });

  if (import.meta.env.DEV && import.meta.env.VITE_DEV_SERVER_URL !== undefined) {
    await browserWindow.loadURL(import.meta.env.VITE_DEV_SERVER_URL);
  } else {
    await browserWindow.loadFile(
      fileURLToPath(new NodeURL('../../../renderer/dist/index.html', import.meta.url)),
    );
  }

  return browserWindow;
}

export async function restoreOrCreateWindow(): Promise<BrowserWindow> {
  let window = BrowserWindow.getAllWindows().find(
    w => !w.isDestroyed(),
  );

  if (window === undefined) {
    try {
      window = await createWindow();
      connectHttp();
      connectFileSystem();
      console.log('[MainWindow] Starting Peerbit node...');
      await startPeerbitNode();
      console.log('[MainWindow] Peerbit node started or already running.');
    } catch (e) {
      console.error('[MainWindow] CRITICAL: Failed to initialize main process components:', e);
      app.quit(); // Ensure app quits on critical failure
      throw e; // Re-throw to reject the promise from restoreOrCreateWindow
    }
  }

  if (window.isMinimized()) {
    window.restore();
  }

  window.focus();
  return window;
}
