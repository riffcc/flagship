console.log('[Main Index] Minimal script execution started.');

import {app} from 'electron'; // Keep one essential Electron import
// import './security-restrictions';
// import {restoreOrCreateWindow} from '/@/mainWindow';
// import {addRelease, ReleaseType as PeerbitReleaseType} from './peerbitNode';

console.log('[Main Index] Imports (mostly) bypassed.');

app.on('ready', () => {
  console.log('[Main Index] Minimal app ready event fired.');
  // For testing, you might want to quit immediately or after a short delay
  // setTimeout(() => app.quit(), 1000);
});

// /**
//  * Prevent electron from running multiple instances.
//  */
// const isSingleInstance = app.requestSingleInstanceLock();
// if (!isSingleInstance) {
//   app.quit();
//   process.exit(0);
// }
// app.on('second-instance', restoreOrCreateWindow);

// /**
//  * Disable Hardware Acceleration to save more system resources.
//  */
// app.disableHardwareAcceleration();

// /**
//  * Shout down background process if all windows was closed
//  */
// app.on('window-all-closed', () => {
//   if (process.platform !== 'darwin') {
//     app.quit();
//   }
// });

// /**
//  * @see https://www.electronjs.org/docs/latest/api/app#event-activate-macos Event: 'activate'.
//  */
// app.on('activate', restoreOrCreateWindow);

// /**
//  * Create the application window when the background process is ready.
//  */
// app
//   .whenReady() // This is effectively replaced by the app.on('ready', ...) above for this minimal test
//   .then(restoreOrCreateWindow)
//   .then(() => {
//     ipcMain.handle('peerbit:addRelease', async (_event, releaseData: PeerbitReleaseType) => {
//       try {
//         const result = await addRelease(releaseData);
//         return result;
//       } catch (error: any) {
//         console.error('IPC peerbit:addRelease error:', error);
//         return {success: false, error: error.message || 'Failed to process release.'};
//       }
//     });
//   })
//   .catch(e => console.error('Failed create window:', e));

// /**
//  * Install Vue.js or any other extension in development mode only.
//  * Note: You must install `electron-devtools-installer` manually
//  */
// // if (import.meta.env.DEV) { ... }

// /**
//  * Check for new version of the application - production mode only.
//  */
// if (import.meta.env.PROD) { ... }
