/**
 * Credits and licensing information for third-party assets used in Flagship.
 */

export interface CreditEntry {
  name: string;
  author: string;
  authorUrl?: string;
  license: string;
  licenseUrl: string;
  sourceUrl: string;
  description: string;
}

export interface SoftwareCredit {
  name: string;
  description: string;
  license: string;
  licenseUrl?: string;
  projectUrl: string;
}

export const assetCredits: CreditEntry[] = [
  {
    name: 'CD Case Placeholder',
    author: 'kenny.r',
    authorUrl: 'https://commons.wikimedia.org/wiki/User:Kenny.r',
    license: 'CC BY-SA 3.0',
    licenseUrl: 'https://creativecommons.org/licenses/by-sa/3.0/',
    sourceUrl: 'https://commons.wikimedia.org/wiki/File:Cd_case.svg',
    description: 'Used as placeholder artwork for releases without cover art.',
  },
];

export const softwareCredits: SoftwareCredit[] = [
  {
    name: 'Vue.js',
    description: 'Progressive JavaScript framework for building user interfaces.',
    license: 'MIT',
    projectUrl: 'https://vuejs.org/',
  },
  {
    name: 'Vuetify',
    description: 'Material Design component framework for Vue.',
    license: 'MIT',
    projectUrl: 'https://vuetifyjs.com/',
  },
  {
    name: 'IPFS',
    description: 'Peer-to-peer hypermedia protocol for content-addressed storage.',
    license: 'MIT/Apache-2.0',
    projectUrl: 'https://ipfs.tech/',
  },
  {
    name: 'Archivist',
    description: 'Distributed storage network for preserving digital content.',
    license: 'MIT/Apache-2.0',
    projectUrl: 'https://github.com/durability-labs/archivist-node',
  },
];

export const contentLicenses = {
  description: 'Riff.CC hosts content under various open licenses. Each release displays its specific license. Common licenses include:',
  licenses: [
    {
      name: 'Creative Commons',
      description: 'A family of licenses that allow creators to share their work while retaining some rights.',
      url: 'https://creativecommons.org/licenses/',
    },
    {
      name: 'Public Domain',
      description: 'Works with no copyright restrictions, free for any use.',
      url: 'https://creativecommons.org/publicdomain/',
    },
  ],
};
