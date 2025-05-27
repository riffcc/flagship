/**
 * @module preload
 */

import { contextBridge, ipcRenderer } from 'electron';
import { isLinux, isMac, isWindows, platform} from './so';
import type { ReleaseData, HashResponse, Release } from '@riffcc/lens-sdk';

contextBridge.exposeInMainWorld('osInfo', {
  isMac,
  isLinux,
  isWindows,
  platform,
});

contextBridge.exposeInMainWorld('electronIPC', {
  onceMainReady: (callback: () => void) => {
    ipcRenderer.once('main-process-ready', callback);
  },
});


contextBridge.exposeInMainWorld('electronLensService', {
  getPublicKey: (): Promise<string> => ipcRenderer.invoke('peerbit:get-public-key'),
  getPeerId: (): Promise<string> => ipcRenderer.invoke('peerbit:get-peer-id'),
  dial: (address: string): Promise<boolean> => ipcRenderer.invoke('peerbit:dial', address),
  addRelease: (releaseData: ReleaseData): Promise<HashResponse> => ipcRenderer.invoke('peerbit:add-release', releaseData),
  getRelease: (id: string): Promise<Release | undefined> => ipcRenderer.invoke('peerbit:get-release', id),
  getLatestReleases: (size?: number): Promise<Release[]> => ipcRenderer.invoke('peerbit:get-latest-releases', size),
});
