import type { Release } from './schema';

export interface IPeerbitService {
  getPublicKey: () => string | Promise<string>;
  getPeerId: () => string | Promise<string>;
  dial: (address: string) => Promise<boolean>;
  addRelease: (release: Release) => Promise<string>;
  getRelease: (id: string) => Promise<Release>;
  getLatestReleases: (size?: number) => Promise<Release[]>;
}
