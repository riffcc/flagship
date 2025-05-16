/**
 * @module preload
 */

import { contextBridge, ipcRenderer } from 'electron';
import type { Release } from '../../lib/src/schema';
import type { AddReleaseResponse } from '../../lib/src/types';

contextBridge.exposeInMainWorld('osInfo', {
  isMac: process.platform === 'darwin',
  isLinux: process.platform === 'linux',
  isWindows: process.platform === 'win32',
  platform: process.platform,
});

contextBridge.exposeInMainWorld('electronIPC', {
  onceMainReady: (callback: () => void) => {
    ipcRenderer.once('main-process-ready', callback);
  }
});

contextBridge.exposeInMainWorld('electronIPC', {
  onceMainReady: (callback: () => void) => {
    ipcRenderer.once('main-process-ready', callback);
  },
});

contextBridge.exposeInMainWorld('electronPeerbit', {
  getPublicKey: (): Promise<string> => ipcRenderer.invoke('peerbit:get-public-key'),
  getPeerId: (): Promise<string> => ipcRenderer.invoke('peerbit:get-peer-id'),
  dial: (address: string): Promise<boolean> => ipcRenderer.invoke('peerbit:dial', address),
  addRelease: (releaseData: any): Promise<AddReleaseResponse> => ipcRenderer.invoke('peerbit:add-release', releaseData),
  getRelease: (id: string): Promise<Release | undefined> => ipcRenderer.invoke('peerbit:get-release', id),
  getLatestReleases: (size?: number): Promise<Release[]> => ipcRenderer.invoke('peerbit:get-latest-releases', size),
});
