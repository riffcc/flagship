import {app, BrowserWindow} from 'electron';
import {dirname, join} from 'node:path';
import {fileURLToPath, URL as NodeURL} from 'node:url';
import {connectFileSystem} from './file-system';
import { LensService } from '@riffcc/lens-sdk';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export let lensService: LensService | undefined = undefined;

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
  const siteAddress = import.meta.env.VITE_SITE_ADDRESS as string | undefined;
  if (siteAddress) {

    lensService = new LensService();
    await lensService.init(join('.lens-node'));
    await lensService.openSite(siteAddress);

  } else {
    throw new Error('VITE_SITE_ADDRESS env var missing. Please review your .env file');
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
      connectFileSystem();
    } catch (e) {
      console.error('[MainWindow] CRITICAL: Failed to initialize main process components:', e);
      app.quit();
      throw e;
    }
  }

  if (window.isMinimized()) {
    window.restore();
  }

  window.focus();
  return window;
}

if (app && typeof app.on === 'function') {
  app.on('will-quit', async () => {

    await lensService?.stop();
  });
}
