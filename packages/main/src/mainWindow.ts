import {app, BrowserWindow} from 'electron';
import {join, resolve as pathResolve} from 'path';
import {fileURLToPath, URL} from 'url';
import {connecterHttp} from './http';
import {connecterSystèmeFichiers} from './systèmeFichiers';
import {startPeerbitNode} from './peerbitNode';

async function createWindow() {
  console.log('[MainWindow] createWindow called.');

  // Determine the project root relative to app.getAppPath()
  // If app.getAppPath() is /Users/wings/projects/flagship/packages/main/dist,
  // then going up THREE levels should be the project root.
  const projectRootGuess = pathResolve(app.getAppPath(), '..', '..', '..');
  const preloadScriptPath = join(projectRootGuess, 'packages/preload/dist/index.cjs');
  
  console.log(`[MainWindow] app.getAppPath() resolved to: ${app.getAppPath()}`);
  console.log(`[MainWindow] Guessed project root: ${projectRootGuess}`);
  console.log(`[MainWindow] Attempting to use preload script path: ${preloadScriptPath}`);

  const browserWindow = new BrowserWindow({
    show: false, // Use the 'ready-to-show' event to show the instantiated BrowserWindow.
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: false, // Sandbox disabled because the demo of preload script depend on the Node.js api
      webviewTag: false, // The webview tag is not recommended. Consider alternatives like an iframe or Electron's BrowserView. @see https://www.electronjs.org/docs/latest/api/webview-tag#warning
      preload: preloadScriptPath, // Use the resolved dynamic path
    },
  });
  console.log('[MainWindow] new BrowserWindow() returned.');

  // Safely access getWebPreferences for logging
  const webPrefs =
    browserWindow.webContents && typeof browserWindow.webContents.getWebPreferences === 'function'
      ? browserWindow.webContents.getWebPreferences()
      : {preload: '[WebPreferences not available]'}; // Fallback for mock or early access
  const actualPreloadPath = webPrefs ? webPrefs.preload : '[Could not determine]';

  console.log(`[MainWindow] app.getAppPath() resolved to: ${app.getAppPath()}`);
  console.log(`[MainWindow] Preload script path configured in webPreferences: ${actualPreloadPath}`);
  if (actualPreloadPath !== preloadScriptPath && actualPreloadPath !== '[WebPreferences not available]') {
    console.warn('[MainWindow] MISMATCH: Preload path used does not match path read from webPreferences!');
  }

  // Only attach these listeners if webContents and its .on method exist (i.e., not in a heavily mocked unit test env)
  if (browserWindow.webContents && typeof browserWindow.webContents.on === 'function') {
    browserWindow.webContents.on('did-fail-load', 
      (event, errorCode, errorDescription, validatedURL, isMainFrame, frameProcessId, frameRoutingId) => {
      console.error(`[MainWindow] WebContents did-fail-load: ${errorDescription} (URL: ${validatedURL}, Code: ${errorCode}, isMainFrame: ${isMainFrame})`);
    });

    browserWindow.webContents.on('preload-error', (event, path, error) => {
      console.error(`[MainWindow] WebContents preload-error: Path: ${path}, Error: ${error.name} - ${error.message}`);
    });
  } else {
    console.warn('[MainWindow] browserWindow.webContents.on is not a function, skipping attachment of did-fail-load and preload-error listeners. (Likely unit test environment)');
  }

  /**
   * If the 'show' property of the BrowserWindow's constructor is omitted from the initialization options,
   * it then defaults to 'true'. This can cause flickering as the window loads the html content,
   * and it also has show problematic behaviour with the closing of the window.
   * Use `show: false` and listen to the  `ready-to-show` event to show the window.
   *
   * @see https://github.com/electron/electron/issues/25012 for the afford mentioned issue.
   */
  browserWindow.on('ready-to-show', () => {
    browserWindow?.show();

    if (import.meta.env.DEV) {
      browserWindow?.webContents.openDevTools();
    }
  });

  /**
   * Load the main page of the main window.
   */
  if (import.meta.env.DEV && import.meta.env.VITE_DEV_SERVER_URL !== undefined) {
    /**
     * Load from the Vite dev server for development.
     */
    await browserWindow.loadURL(import.meta.env.VITE_DEV_SERVER_URL);
  } else {
    /**
     * Load from the local file system for production and test.
     *
     * Use BrowserWindow.loadFile() instead of BrowserWindow.loadURL() for WhatWG URL API limitations
     * when path contains special characters like `#`.
     * Let electron handle the path quirks.
     * @see https://github.com/nodejs/node/issues/12682
     * @see https://github.com/electron/electron/issues/6869
     */
    await browserWindow.loadFile(
      fileURLToPath(new URL('./../../renderer/dist/index.html', import.meta.url)),
    );
  }

  return browserWindow;
}

/**
 * Restore an existing BrowserWindow or Create a new BrowserWindow.
 */
export async function restoreOrCreateWindow() {
  let window = BrowserWindow.getAllWindows().find(
    w => !w.isDestroyed(), // && w.title !== 'WRTC Relay', Vérification 'WRTC Relay' probablement plus nécessaire
  );

  if (window === undefined) {
    window = await createWindow();
    connecterHttp();
    connecterSystèmeFichiers();
    // Start the local Peerbit node for testing
    await startPeerbitNode();
  }

  if (window.isMinimized()) {
    window.restore();
  }

  window.focus();
}
