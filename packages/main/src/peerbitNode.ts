import {field, variant} from '@dao-xyz/borsh';
import {Documents} from '@peerbit/document';
import {Program} from '@peerbit/program';
import {Peerbit} from 'peerbit';

// Define constants locally, assuming their values were their names or similar common patterns.
// Adjust these string values if they were different in the original consts file.
const RELEASES_NAME_COLUMN = 'name';
const RELEASES_FILE_COLUMN = 'file'; // Example, adjust if original was different (e.g. 'contentCID')
const RELEASES_AUTHOR_COLUMN = 'author';
const RELEASES_CATEGORY_COLUMN = 'category';
const RELEASES_THUMBNAIL_COLUMN = 'thumbnail';
const RELEASES_COVER_COLUMN = 'cover';
const RELEASES_METADATA_COLUMN = 'metadata';

// Define ReleaseType locally, matching the structure from packages/peerbit-adapter/src/types.ts
export type ReleaseType<T = Record<string, unknown> | string> = {
  [RELEASES_NAME_COLUMN]: string;
  [RELEASES_FILE_COLUMN]: string;
  [RELEASES_AUTHOR_COLUMN]: string;
  [RELEASES_CATEGORY_COLUMN]: string;
  [RELEASES_THUMBNAIL_COLUMN]?: string;
  [RELEASES_COVER_COLUMN]?: string;
  [RELEASES_METADATA_COLUMN]?: T;
};

let peerbitClient: Peerbit | undefined;

// Define the Release class for borsh serialization, mapping to existing type fields
@variant(0) // Example version, adjust if needed
class Release implements ReleaseType {
  @field({type: 'string'})
  [RELEASES_NAME_COLUMN]: string;

  @field({type: 'string'})
  [RELEASES_FILE_COLUMN]: string;

  @field({type: 'string'})
  [RELEASES_AUTHOR_COLUMN]: string;

  @field({type: 'string'})
  [RELEASES_CATEGORY_COLUMN]: string;

  @field({type: 'string', option: true})
  [RELEASES_THUMBNAIL_COLUMN]?: string;

  @field({type: 'string', option: true})
  [RELEASES_COVER_COLUMN]?: string;

  // Assuming metadata is a JSON string for simplicity with borsh.
  // If it's a complex object, further schema definition or a custom serializer might be needed.
  @field({type: 'string', option: true})
  [RELEASES_METADATA_COLUMN]?: string; // Or use 'json-object' if borsh supports it directly, or a nested @variant class

  constructor(data: ReleaseType) {
    this[RELEASES_NAME_COLUMN] = data[RELEASES_NAME_COLUMN];
    this[RELEASES_FILE_COLUMN] = data[RELEASES_FILE_COLUMN];
    this[RELEASES_AUTHOR_COLUMN] = data[RELEASES_AUTHOR_COLUMN];
    this[RELEASES_CATEGORY_COLUMN] = data[RELEASES_CATEGORY_COLUMN];
    this[RELEASES_THUMBNAIL_COLUMN] = data[RELEASES_THUMBNAIL_COLUMN];
    this[RELEASES_COVER_COLUMN] = data[RELEASES_COVER_COLUMN];
    // Ensure metadata is stringified if it's an object and the field expects a string
    if (typeof data[RELEASES_METADATA_COLUMN] === 'object' && data[RELEASES_METADATA_COLUMN] !== null) {
      this[RELEASES_METADATA_COLUMN] = JSON.stringify(data[RELEASES_METADATA_COLUMN]);
    } else {
      this[RELEASES_METADATA_COLUMN] = data[RELEASES_METADATA_COLUMN] as string | undefined;
    }
  }
}


@variant('releases-db') // Unique name for the program
class ReleasesDB extends Program {
  @field({type: Documents})
  releases: Documents<Release>;

  constructor() {
    super();
    this.releases = new Documents();
  }

  async open(): Promise<void> {
    await this.releases.open({
      type: Release,
      // index: {
      //   key: "name" // Example: if you want to index by name
      // }
    });
  }
}

export async function startPeerbitNode() {
  if (peerbitClient) {
    console.log('Peerbit client already started.');
    return peerbitClient;
  }

  try {
    console.log('Starting Peerbit client...');
    // Peerbit.create() handles Helia initialization internally
    peerbitClient = await Peerbit.create();
    console.log('Peerbit client started. Peer ID:', peerbitClient.peerId.toString());

    // Open the ReleasesDB program
    const releasesProgram = await peerbitClient.open(new ReleasesDB());
    console.log('ReleasesDB program opened at address:', releasesProgram.address?.toString());

    // Example: Listen for updates on the releases store (optional, for debugging)
    releasesProgram.releases.events.addEventListener('change', event => {
      console.log('Releases store changed:', event.detail);
    });

    return peerbitClient;
  } catch (error) {
    console.error('Failed to start Peerbit client:', error);
    throw error;
  }
}

export async function stopPeerbitNode() {
  if (peerbitClient) {
    console.log('Stopping Peerbit client...');
    await peerbitClient.stop();
    peerbitClient = undefined;
    console.log('Peerbit client stopped.');
  }
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
