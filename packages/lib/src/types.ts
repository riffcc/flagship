import type { Release } from './schema';

export interface AddReleaseResponse {
  success: boolean;
  id: string;
  hash: string;
  message?: string;
  error?: string;
}

export interface IPeerbitService {
  getPublicKey: () => Promise<string>;
  getPeerId: () => Promise<string>;
  dial: (address: string) => Promise<boolean>;
  addRelease: (releaseData: any) => Promise<AddReleaseResponse>;
  getRelease: (id: string) => Promise<Release | undefined>;
  getLatestReleases: (size?: number) => Promise<Release[]>;
  // updateRelease will also need to be defined here eventually
  // updateRelease?: (id: string, releaseData: any) => Promise<any>; 
}
