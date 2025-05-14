console.log('[Main Index] Minimal script execution started.');

import {app, ipcMain} from 'electron'; // MODIFIED: ipcMain added
// import './security-restrictions';
import {restoreOrCreateWindow} from './mainWindow';
import {addRelease, ReleaseType as PeerbitReleaseType} from './peerbitNode'; // UNCOMMENTED

console.log('[Main Index] Imports (mostly) bypassed.');

// app.on('ready', () => { // Keep this commented if whenReady is used below
//   console.log('[Main Index] Minimal app ready event fired.');
// });

/**
 * Prevent electron from running multiple instances.
 */
const isSingleInstance = app.requestSingleInstanceLock();
if (!isSingleInstance) {
  app.quit();
  process.exit(0);
}
app.on('second-instance', restoreOrCreateWindow);

/**
 * Disable Hardware Acceleration to save more system resources.
 */
app.disableHardwareAcceleration();

/**
 * Shout down background process if all windows was closed
 */
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

/**
 * @see https://www.electronjs.org/docs/latest/api/app#event-activate-macos Event: 'activate'.
 */
app.on('activate', restoreOrCreateWindow);

/**
 * Create the application window when the background process is ready.
 */
app
  .whenReady()
  .then(restoreOrCreateWindow) // UNCOMMENTED
  .then(() => {
    // Assuming ipcMain and addRelease are needed for your app functionality
    // You might need to uncomment their imports and related code if they are used by the window
    // For now, focusing on getting the window to appear.
    ipcMain.handle('peerbit:addRelease', async (_event, releaseData: PeerbitReleaseType) => { // UNCOMMENTED
      try { // UNCOMMENTED
        const result = await addRelease(releaseData); // UNCOMMENTED
        return result; // UNCOMMENTED
      } catch (error: any) { // UNCOMMENTED
        console.error('IPC peerbit:addRelease error:', error); // UNCOMMENTED
        return {success: false, error: error.message || 'Failed to process release.'}; // UNCOMMENTED
      } // UNCOMMENTED
    }); // UNCOMMENTED
  })
  .catch(e => console.error('Failed create window:', e));

// /**
//  * Install Vue.js or any other extension in development mode only.
//  * Note: You must install `electron-devtools-installer` manually
//  */
// // if (import.meta.env.DEV) { ... }

// /**
//  * Check for new version of the application - production mode only.
//  */
// if (import.meta.env.PROD) { ... }
