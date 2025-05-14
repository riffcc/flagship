/**
 * @module preload
 */
import {contextBridge, ipcRenderer} from 'electron';

// Define the type for the release data to be sent to the main process
// This should match the structure expected by `addRelease` in `peerbitNode.ts`
// and the `ReleaseType` definition there (effectively, an object with these properties).
type ReleaseDataType = {
  name: string;
  file: string; // Corresponds to contentCID from the form
  author: string;
  category: string;
  thumbnail?: string;
  cover?: string;
  metadata?: Record<string, unknown> | string; // Peerbit's Release class constructor handles object stringification
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

export {plateforme, surLinux, surMac, surWindows} from './so.js';

export {requêteHttp} from './http.js';

export {choisirDossier} from './systèmeFichiers.js';
