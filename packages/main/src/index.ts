import {app, ipcMain} from 'electron';
import '/@/security-restrictions';
import {lensService, restoreOrCreateWindow } from '/@/main-window';
import type { ReleaseData } from '@riffcc/lens-sdk';


const isSingleInstance = app.requestSingleInstanceLock();
if (!isSingleInstance) {
  app.quit();
  process.exit(0);
}
app.on('second-instance', restoreOrCreateWindow);

app.disableHardwareAcceleration();

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', restoreOrCreateWindow);

app
  .whenReady()
  .then(async () => {
    const mainWindow = await restoreOrCreateWindow();

      ipcMain.handle('peerbit:get-public-key', async () => lensService?.getPublicKey());
      ipcMain.handle('peerbit:get-peer-id', async () => lensService?.getPeerId());
      ipcMain.handle('peerbit:dial', async (_event, address: string) => lensService?.dial(address));
      ipcMain.handle('peerbit:add-release', async (_event, releaseData: ReleaseData) =>
        lensService?.addRelease(releaseData),
      );
      ipcMain.handle('peerbit:get-release', async (_event, id: string) =>
        lensService?.getRelease({ id }),
      );
      ipcMain.handle('peerbit:get-latest-releases', async (_event, size?: number) =>
        lensService?.getReleases(size ? { fetch: size } : undefined),
      );
    // Notify renderer that main is ready
    if (mainWindow && mainWindow.webContents && !mainWindow.isDestroyed()) {
      const sendReadySignal = () => {
        if (!mainWindow.isDestroyed()) {
          mainWindow.webContents.send('main-process-ready');
          console.log('[Main IPC] Sent "main-process-ready" to renderer.');
        } else {
          console.warn('[Main IPC] Main window was destroyed before ready signal could be sent.');
        }
      };

      if (mainWindow.webContents.isLoading()) {
        console.log('[Main IPC] Main window is loading. Waiting for did-finish-load to send ready signal.');
        mainWindow.webContents.once('did-finish-load', () => {
          console.log('[Main IPC] Main window did-finish-load event fired.');
          sendReadySignal();
        });
      } else {
        // If not loading, content is presumably already loaded (e.g., hot reload, or already loaded)
        console.log('[Main IPC] Main window is not loading. Sending ready signal immediately.');
        sendReadySignal();
      }
    } else {
      console.warn('[Main IPC] Main window not available or destroyed; cannot send "main-process-ready".');
    }
  })
  .catch(e => console.error('Error during app initialization or IPC setup:', e));
