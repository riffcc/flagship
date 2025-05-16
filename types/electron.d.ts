import type { IPeerbitService } from '../packages/lib/src/types';

declare global {
  interface Window {
    electronIPC: {
      onceMainReady: (callback: () => void) => void;
    };
    // Exposed by preload/src/index.ts
    electronPeerbit: IPeerbitService;
    // Exposed by preload/src/index.ts for OS info
    osInfo: {
      isMac: boolean;
      isLinux: boolean;
      isWindows: boolean;
      platform: string;
    };
    // Add any other APIs exposed by your preload script here
  }
}
export {}; // Ensures this is treated as a module.
