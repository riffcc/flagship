// packages/lib/tests/site.test.ts

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { TestSession } from '@peerbit/test-utils';
import type { ProgramClient } from '@peerbit/program';
import { Site, Release } from '../src/schema';
import type { ReleaseData } from '../src/types';
import {
  DEFAULT_SITE_ID,
  RELEASE_NAME_PROPERTY,
  RELEASE_CATEGORY_ID_PROPERTY,
  RELEASE_CONTENT_CID_PROPERTY,
  ID_PROPERTY,
  RELEASE_THUMBNAIL_CID_PROPERTY,
  RELEASE_METADATA_PROPERTY,
} from '../src/constants';
import { delay } from '@peerbit/time';

describe('Site Program', () => {
  let session: TestSession;
  let client: ProgramClient;
  let site: Site;

  beforeEach(async () => {
    session = await TestSession.connected(1);
    client = session.peers[0];
    site = new Site(DEFAULT_SITE_ID, client.identity.publicKey);
    await client.open(site);
  });

  afterEach(async () => {
    if (site && !site.closed) {
      await site.close();
    }
    if (session) {
      await session.stop();
    }
  });
  it('can create a site, add a release, and get the release', async () => {
    // 1. Prepare Release Data
    const releaseData: ReleaseData = {
      [RELEASE_NAME_PROPERTY]: 'RiP!: A Remix Manifesto',
      [RELEASE_CATEGORY_ID_PROPERTY]: 'movie',
      [RELEASE_CONTENT_CID_PROPERTY]: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      [RELEASE_THUMBNAIL_CID_PROPERTY]: 'Qmb3eeESRoX5L6NhTYLEtFFUS1FZgqe1e7hdBk2f57DUGh',
    };

    // 2. Create a Release instance (this will auto-generate an ID)
    const newReleaseInstance = new Release(releaseData);
    const expectedReleaseId = newReleaseInstance[ID_PROPERTY];


    // 3. Add the Release to the Site's releases store
    const entryHash = await site.addRelease(newReleaseInstance);
    expect(entryHash).toBeTypeOf('string');
    expect(entryHash.length).toBeGreaterThan(0);

    await delay(200);

    // 4. Get the Release using its ID
    const retrievedRelease = await site.getRelease(expectedReleaseId);
    // 5. Assertions
    expect(retrievedRelease).toBeDefined();
    expect(retrievedRelease).not.toBeNull();

    if (retrievedRelease) {
      expect(retrievedRelease[ID_PROPERTY]).toEqual(expectedReleaseId);
      expect(retrievedRelease[RELEASE_NAME_PROPERTY]).toEqual(releaseData[RELEASE_NAME_PROPERTY]);
      expect(retrievedRelease[RELEASE_CATEGORY_ID_PROPERTY]).toEqual(releaseData[RELEASE_CATEGORY_ID_PROPERTY]);
      expect(retrievedRelease[RELEASE_CONTENT_CID_PROPERTY]).toEqual(releaseData[RELEASE_CONTENT_CID_PROPERTY]);

      // Check optional properties if they were set
      if (releaseData[RELEASE_THUMBNAIL_CID_PROPERTY]) {
        expect(retrievedRelease[RELEASE_THUMBNAIL_CID_PROPERTY]).toEqual(releaseData[RELEASE_THUMBNAIL_CID_PROPERTY]);
      }
      if (releaseData[RELEASE_METADATA_PROPERTY]) {
        expect(retrievedRelease[RELEASE_METADATA_PROPERTY]).toEqual(releaseData[RELEASE_METADATA_PROPERTY]);
      }
    }

  });

  it('getRelease returns undefined for a non-existent ID', async () => {
    const nonExistentId = 'non-existent-id-12345';
    const retrievedRelease = await site.getRelease(nonExistentId);
    expect(retrievedRelease).toBeUndefined();
  });
});
