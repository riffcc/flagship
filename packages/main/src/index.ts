import {app, ipcMain} from 'electron';
import '/@/security-restrictions';
import {restoreOrCreateWindow} from '/@/main-window';
import { getPeerbitNode, getSiteProgram } from './peerbit-node';
import type { Release } from '/@/lib/schema';

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
  .then(restoreOrCreateWindow)
  .then(() => {
    ipcMain.handle('peerbit:get-public-key', async () => {
      const node = getPeerbitNode();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      return new Promise(r => r(node.identity.publicKey.toString()));
    });

    ipcMain.handle('peerbit:get-peer-id', async () => {
      const node = getPeerbitNode();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      return new Promise(r => r(node.peerId.toString()));
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
        return false;
      }
    });
    ipcMain.handle('peerbit:add-release', async (_event, release: Release) => {
      const node = getPeerbitNode();
      const siteProgram = getSiteProgram();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      if (!siteProgram) throw new Error('Site program not initialized in main process');
      try {
        console.log(`[Main IPC] Adding release ${release.id}`);
        return await siteProgram.addRelease(release);
      } catch (error) {
        console.error(`[Main IPC] Error adding release ${release.id}:`, error);
      }
    });
    ipcMain.handle('peerbit:get-release', async (_event, id: string) => {
      const node = getPeerbitNode();
      const siteProgram = getSiteProgram();
      if (!node) throw new Error('Peerbit node not initialized in main process');
      if (!siteProgram) throw new Error('Site program not initialized in main process');
      try {
        console.log(`[Main IPC] Getting release: ${id}`);
        const release = await siteProgram.getRelease(id);
        console.log('[Main IPC] Release retrieved');
        return release;
      } catch (error) {
        console.error('[Main IPC] Error getting releases:', error);
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
        console.error('[Main IPC] Error getting releases:', error);
      }
    });
  })
  .catch(e => console.error('Failed to create window:', e));
