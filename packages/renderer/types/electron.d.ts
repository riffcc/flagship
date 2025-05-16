import type { Release } from '../../../lib/src/schema';
import type { AddReleaseResponse } from '../../../lib/src/types';

declare global {
  interface Window {
    osInfo: {
      isMac: boolean;
      isLinux: boolean;
      isWindows: boolean;
      platform: string;
    };
    electronIPC: {
      onceMainReady: (callback: () => void) => void;
    };
    electronPeerbit: {
      getPublicKey: () => Promise<string>;
      getPeerId: () => Promise<string>;
      dial: (address: string) => Promise<boolean>;
      addRelease: (releaseData: any) => Promise<AddReleaseResponse>;
      getRelease: (id: string) => Promise<Release | undefined>;
      getLatestReleases: (size?: number) => Promise<Release[]>;
      // updateRelease would also be defined here eventually
    };
  }
}

export {}; // Ensure this file is treated as a module. 