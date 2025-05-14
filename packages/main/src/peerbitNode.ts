import {field, variant} from '@dao-xyz/borsh';
// import {Documents} from '@peerbit/document'; // COMMENTED OUT
// import {Program} from '@peerbit/program'; // COMMENTED OUT
import {Peerbit} from 'peerbit'; // Keep this for now, Peerbit itself might be CJS-friendly or its main export is.

// Define constants locally, assuming their values were their names or similar common patterns.
// Adjust these string values if they were different in the original consts file.
const RELEASES_NAME_COLUMN = 'name';
const RELEASES_FILE_COLUMN = 'file'; // Example, adjust if original was different (e.g. 'contentCID')
const RELEASES_AUTHOR_COLUMN = 'author';
const RELEASES_CATEGORY_COLUMN = 'category';
const RELEASES_THUMBNAIL_COLUMN = 'thumbnail';
const RELEASES_COVER_COLUMN = 'cover';
const RELEASES_METADATA_COLUMN = 'metadata';
const RELEASES_TIMESTAMP_COLUMN = 'timestamp';
const RELEASES_TARGET_GROUP_COLUMN = 'targetGroup';
const RELEASES_CATEGORY_BOOKS = 'livres';
const RELEASES_CATEGORY_GAMES = 'jeux';
const RELEASES_CATEGORY_MOVIES = 'films';
const RELEASES_CATEGORY_MUSIC = 'musiques';
const RELEASES_CATEGORY_OTHER = 'autres';
const RELEASES_TARGET_GROUP_VISUAL_NOVEL_FRENCH = 'visual-novel-fr';
const RELEASES_TARGET_GROUP_VISUAL_NOVEL_ENGLISH = 'visual-novel-en';
const RELEASES_TARGET_GROUP_ADULTS_ONLY = 'adultes-seulement';

// Define ReleaseType locally, matching the structure from packages/peerbit-adapter/src/types.ts
export type ReleaseType<T = Record<string, unknown> | string> = {
  [RELEASES_NAME_COLUMN]: string;
  [RELEASES_FILE_COLUMN]: string;
  [RELEASES_AUTHOR_COLUMN]: string;
  [RELEASES_CATEGORY_COLUMN]: string;
  [RELEASES_THUMBNAIL_COLUMN]?: string;
  [RELEASES_COVER_COLUMN]?: string;
  [RELEASES_METADATA_COLUMN]?: T;
  [RELEASES_TIMESTAMP_COLUMN]?: number;
  [RELEASES_TARGET_GROUP_COLUMN]?: string[];
};

let peerbitNode: Peerbit | undefined = undefined;
// let releasesDB: ReleasesDB | undefined = undefined; // COMMENTED OUT as ReleasesDB is commented

// @variant(0) // COMMENTED OUT
// class Release implements ReleaseType {
//   @field({type: 'string'})
//   [RELEASES_NAME_COLUMN]: string;
// 
//   @field({type: 'string'})
//   [RELEASES_FILE_COLUMN]: string;
// 
//   @field({type: 'string'})
//   [RELEASES_AUTHOR_COLUMN]: string;
// 
//   @field({type: 'string'})
//   [RELEASES_CATEGORY_COLUMN]: string;
// 
//   @field({type: 'string', option: true})
//   [RELEASES_THUMBNAIL_COLUMN]?: string;
// 
//   @field({type: 'string', option: true})
//   [RELEASES_COVER_COLUMN]?: string;
// 
//   @field({type: 'string', option: true})
//   [RELEASES_METADATA_COLUMN]?: string;
// 
//   constructor(data: ReleaseType) {
//     this[RELEASES_NAME_COLUMN] = data[RELEASES_NAME_COLUMN];
//     this[RELEASES_FILE_COLUMN] = data[RELEASES_FILE_COLUMN];
//     this[RELEASES_AUTHOR_COLUMN] = data[RELEASES_AUTHOR_COLUMN];
//     this[RELEASES_CATEGORY_COLUMN] = data[RELEASES_CATEGORY_COLUMN];
//     this[RELEASES_THUMBNAIL_COLUMN] = data[RELEASES_THUMBNAIL_COLUMN];
//     this[RELEASES_COVER_COLUMN] = data[RELEASES_COVER_COLUMN];
//     if (typeof data[RELEASES_METADATA_COLUMN] === 'object' && data[RELEASES_METADATA_COLUMN] !== null) {
//       this[RELEASES_METADATA_COLUMN] = JSON.stringify(data[RELEASES_METADATA_COLUMN]);
//     } else {
//       this[RELEASES_METADATA_COLUMN] = data[RELEASES_METADATA_COLUMN] as string | undefined;
//     }
//   }
// }
// 
// 
// @variant('releases-db') // COMMENTED OUT
// class ReleasesDB extends Program { // Program itself is an ESM import problem
//   @field({type: Documents}) // Documents is an ESM import problem
//   releases: Documents<Release>;
// 
//   constructor() {
//     super();
//     this.releases = new Documents();
//   }
// 
//   async open(): Promise<void> {
//     await this.releases.open({
//       type: Release,
//     });
//   }
// }

export async function startPeerbitNode(): Promise<void> {
  console.log('[PeerbitNode] startPeerbitNode called (dummy implementation).');
  // if (peerbitNode) {
  //   console.log('[PeerbitNode] Peerbit node already started.');
  //   return;
  // }
  // try {
  //   console.log('[PeerbitNode] Starting Peerbit node...');
  //   peerbitNode = await Peerbit.create();
  //   console.log(`[PeerbitNode] Peerbit node started. Peer ID: ${peerbitNode.id.toString()}`);
  //   console.log('[PeerbitNode] Opening ReleasesDB...');
  //   releasesDB = await peerbitNode.open(new ReleasesDB());
  //   console.log(`[PeerbitNode] ReleasesDB opened at address: ${releasesDB.address.toString()}`);
  // } catch (error) {
  //   console.error('[PeerbitNode] Error starting Peerbit node or opening DB:', error);
  //   throw error;
  // }
}

export async function addRelease(releaseData: ReleaseType) {
  console.log('[PeerbitNode] addRelease called with (dummy implementation):', releaseData);
  // if (!releasesDB) {
  //   console.error('[PeerbitNode] ReleasesDB not initialized. Call startPeerbitNode first.');
  //   return {success: false, error: 'ReleasesDB not initialized.'};
  // }
  // try {
  //   const newRelease = new Release(); // This would fail as Release is commented
  //   Object.assign(newRelease, releaseData);
  //   newRelease.id = releaseData.file; // Use CID as ID
  //   newRelease.timestamp = releaseData.timestamp || Date.now();
  //   newRelease.targetGroup = releaseData.targetGroup || [];

  //   // If metadata is an object, stringify it for storage, or handle as string if already
  //   if (typeof releaseData.metadata === 'object' && releaseData.metadata !== null) {
  //     newRelease.metadata = JSON.stringify(releaseData.metadata);
  //   } else {
  //     newRelease.metadata = releaseData.metadata as string | undefined;
  //   }

  //   console.log('[PeerbitNode] Adding release to DB:', newRelease);
  //   await releasesDB.releases.put(newRelease);
  //   console.log(`[PeerbitNode] Release "${releaseData.name}" added successfully with ID ${newRelease.id}.`);
  //   return {success: true, message: `Release "${releaseData.name}" added successfully.`};
  // } catch (error: any) {
  //   console.error('[PeerbitNode] Error adding release:', error);
  //   return {success: false, error: error.message || 'Failed to add release.'};
  // }
  return {success: true, message: `Release "${releaseData.name}" processed by dummy addRelease.`}; // Dummy success
}

// Ensure Peerbit node is stopped gracefully on app exit
import {app} from 'electron';

// Conditionally attach the 'will-quit' listener
// This prevents errors in test environments where `app.on` might not be defined on a mocked `app` object
if (typeof app.on === 'function') {
  app.on('will-quit', async () => {
    await stopPeerbitNode();
  });
}

export async function getRelease(id: string) {
  console.log(`[PeerbitNode] getRelease(${id}) called (dummy implementation).`);
  return undefined;
}

export async function getAllReleases() {
  console.log('[PeerbitNode] getAllReleases called (dummy implementation).');
  return [];
}

export async function stopPeerbitNode(): Promise<void> {
  console.log('[PeerbitNode] stopPeerbitNode called (dummy implementation).');
  // if (peerbitNode) {
  //   console.log('[PeerbitNode] Stopping Peerbit node...');
  //   await peerbitNode.stop();
  //   peerbitNode = undefined;
  //   releasesDB = undefined;
  //   console.log('[PeerbitNode] Peerbit node stopped.');
  // }
}
