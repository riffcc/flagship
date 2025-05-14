/**
 * @module preload
 */
console.log('[Preload] Script starting...');

import {contextBridge, ipcRenderer} from 'electron';

// Define the type for the release data to be sent to the main process
type ReleaseDataType = {
  name: string;
  file: string; // Corresponds to contentCID from the form
  author: string;
  category: string;
  thumbnail?: string;
  cover?: string;
  metadata?: Record<string, unknown> | string;
};

// Define the expected response structure from the IPC call
type AddReleaseResponseType = {
  success: boolean;
  message?: string;
  error?: string;
};

contextBridge.exposeInMainWorld('peerbitAPI', {
  addRelease: (releaseData: ReleaseDataType): Promise<AddReleaseResponseType> =>
    ipcRenderer.invoke('peerbit:addRelease', releaseData),
});
console.log('[Preload] peerbitAPI exposed via contextBridge.');

// Restore original exports
export {plateforme, surLinux, surMac, surWindows} from './so.js';
export {requêteHttp} from './http.js';
export {choisirDossier} from './systèmeFichiers.js';

console.log('[Preload] Original exports restored.');
