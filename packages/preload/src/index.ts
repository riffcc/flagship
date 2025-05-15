/**
 * @module preload
 */

import { contextBridge, ipcRenderer } from 'electron';
import * as so from './so';
import type { Release } from '/@/lib/schema';

contextBridge.exposeInMainWorld('osInfo', {
  isMac: so.isMac,
  isLinux: so.isLinux,
  isWindows: so.isWindows,
  platform: so.platform,
});

contextBridge.exposeInMainWorld('electronPeerbit', {
  getPublicKey: (): Promise<string> => ipcRenderer.invoke('peerbit:get-public-key'),
  getPeerId: (): Promise<string> => ipcRenderer.invoke('peerbit:get-peer-id'),
  dial: (address: string): Promise<boolean> => ipcRenderer.invoke('peerbit:dial', address),
  addRelease: (release: Release): Promise<boolean> => ipcRenderer.invoke('peerbit:add-release', release),
  getRelease: (id: string): Promise<Release> => ipcRenderer.invoke('peerbit:get-release', id),
  getLatestReleases: (size?: number): Promise<Release[]> => ipcRenderer.invoke('peerbit:get-latest-releases', size),
});
