import {app, ipcMain, BrowserWindow} from 'electron';
import '/@/security-restrictions';
import {restoreOrCreateWindow} from '/@/main-window';
import { getPeerbitNode, getSiteProgram } from './peerbit-node';
import { Release } from '/@/lib/schema';

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

    ipcMain.handle('peerbit:get-public-key', async () => {
      const node = getPeerbitNode();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      return node.identity.publicKey.toString();
    });

    ipcMain.handle('peerbit:get-peer-id', async () => {
      const node = getPeerbitNode();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      return node.peerId.toString();
    });

    ipcMain.handle('peerbit:dial', async (_event, address: string) => {
      const node = getPeerbitNode();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      try {
        console.log(`[Main IPC] Dialing ${address}`);
        await node.dial(address);
        return true;
      } catch (error) {
        console.error(`[Main IPC] Error dialing ${address}:`, error);
        throw error;
      }
    });

    ipcMain.handle('peerbit:add-release', async (_event, releaseDataFromRenderer: any) => {
      const node = getPeerbitNode();
      const siteProgram = getSiteProgram();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      if (!siteProgram) throw new Error('Site program not initialized in main process');

      try {
        // Map data from renderer to Release constructor properties
        const releaseConstructorProps = {
          name: releaseDataFromRenderer.name,
          categoryId: releaseDataFromRenderer.category, // Map 'category' to 'categoryId'
          contentCID: releaseDataFromRenderer.file,     // Map 'file' to 'contentCID'
          thumbnailCID: releaseDataFromRenderer.thumbnail, // Map 'thumbnail' to 'thumbnailCID'
          // 'author' and 'cover' from renderer are not in Release constructor, so they are omitted here.
          // If they need to be stored, the Release class definition needs to include them.
          metadata: releaseDataFromRenderer.metadata ? JSON.stringify(releaseDataFromRenderer.metadata) : undefined,
        };

        // Validate required fields before constructing
        if (!releaseConstructorProps.name || !releaseConstructorProps.categoryId || !releaseConstructorProps.contentCID) {
            throw new Error('Missing required fields (name, categoryId, or contentCID) for Release constructor.');
        }

        const releaseInstance = new Release(releaseConstructorProps);

        if (!releaseInstance.id) {
          console.error('[Main IPC] Critical: Release instance created without an ID. Data:', releaseConstructorProps);
          throw new Error('Release instance created without an ID. Check Release class constructor.');
        }

        console.log(`[Main IPC] Adding release - ID: ${releaseInstance.id}, Name: ${releaseInstance.name}`);
        const resultHash = await siteProgram.addRelease(releaseInstance);

        return { success: true, id: releaseInstance.id, hash: resultHash, message: "Release added successfully" };
      } catch (error) {
        console.error(`[Main IPC] Error adding release. Initial data from renderer (ID: ${releaseDataFromRenderer.id}):`, releaseDataFromRenderer, "Mapped props:", /*releaseConstructorProps cannot be logged here if error was before its definition*/ error);
        // Ensure a structured error is returned if the component expects it, or rethrow for a generic error.
        // For now, rethrowing the original error as the component might not be robustly handling {success: false}
        throw error;
      }
    });

    ipcMain.handle('peerbit:get-release', async (_event, id: string) => {
      const node = getPeerbitNode();
      const siteProgram = getSiteProgram();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      if (!siteProgram) throw new Error('Site program not initialized in main process');
      try {
        console.log(`[Main IPC] Getting release by ID: ${id}`);
        const release = await siteProgram.getRelease(id);
        console.log('[Main IPC] Release retrieved (or not found)');
        return release;
      } catch (error) {
        console.error(`[Main IPC] Error getting release by ID ${id}:`, error);
        throw error;
      }
    });

    ipcMain.handle('peerbit:get-latest-releases', async (_event, size?: number) => {
      const node = getPeerbitNode();
      const siteProgram = getSiteProgram();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      if (!siteProgram) throw new Error('Site program not initialized in main process');
      try {
        console.log('[Main IPC] Getting latest releases');
        const releases = await siteProgram.getLatestReleases(size);
        console.log('[Main IPC] Releases retrieved');
        return releases;
      } catch (error) {
        console.error('[Main IPC] Error getting latest releases:', error);
        throw error;
      }
    });

    console.log('[Main IPC] All Peerbit IPC handlers registered.');

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
