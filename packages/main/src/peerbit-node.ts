import { DEFAULT_SITE_ID } from '/@/lib/constants';
import { app } from 'electron';
import { Peerbit } from 'peerbit';
import { join as joinPath } from 'node:path';
import { Site } from '/@/lib/schema';

let peerbitNodeInstance: Peerbit | undefined = undefined;
let mainSiteProgram: Site | undefined;

export async function startPeerbitNode(): Promise<Peerbit> {
  let siteId = import.meta.env.VITE_SITE_ID as string | undefined;
  if (!siteId) {
    console.warn('VITE_SITE_ID is missing in main process environment variables, using the default siteId');
  }
  siteId = DEFAULT_SITE_ID;
  if (peerbitNodeInstance) {
    console.log('[PeerbitNode Main] Returning existing Peerbit node instance.');
    return peerbitNodeInstance;
  }

  try {
    const userDataPath = app.getPath('userData');
    const peerbitDir = joinPath(userDataPath, '.peerbit', siteId);
    console.log(`[PeerbitNode Main] Creating new Peerbit node instance in directory: ${peerbitDir}`);

    peerbitNodeInstance = await Peerbit.create({
      directory: peerbitDir,
    });

    console.log(`[PeerbitNode Main] New Peerbit node instance created with ID: ${peerbitNodeInstance.peerId.toString()}`);

    mainSiteProgram = await peerbitNodeInstance.open(new Site(siteId));
    console.log(`[PeerbitNode Main] Site program opened for siteId: ${siteId}`);

    return peerbitNodeInstance;
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(`[PeerbitNode Main] Error during Peerbit.create() or Site program opening: ${errorMessage}`, error);
    throw new Error(`[PeerbitNode Main] Error during Peerbit.create() or Site program opening: ${errorMessage}`);
  }
}

export async function stopPeerbitNode(): Promise<void> {
  if (mainSiteProgram) {
    await mainSiteProgram.close();
    mainSiteProgram = undefined;
  }
  if (peerbitNodeInstance) {
    console.log('[PeerbitNode Main] Stopping Peerbit node instance...');
    await peerbitNodeInstance.stop();
    console.log('[PeerbitNode Main] Peerbit node instance stopped.');
  }
  peerbitNodeInstance = undefined;
}

export function getPeerbitNode(): Peerbit | undefined {
  return peerbitNodeInstance;
}

export function getSiteProgram(): Site | undefined {
  return mainSiteProgram;
}

// Graceful shutdown
if (app && typeof app.on === 'function') {
  app.on('will-quit', async () => {
    console.log('[PeerbitNode Main] Electron app is about to quit. Stopping Peerbit node and Site program.');
    await stopPeerbitNode();
  });
}
