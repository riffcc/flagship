// tests/acl.test.ts
import { describe, test, expect, beforeAll, afterAll, afterEach } from 'vitest';
import type { ProgramClient } from '@peerbit/program';
import { TestSession } from '@peerbit/test-utils';
import { Site, Release } from '../src/schema'; // Assuming Release is exported from schema
import { DEFAULT_SITE_ID, RELEASE_NAME_PROPERTY, RELEASE_CATEGORY_ID_PROPERTY, RELEASE_CONTENT_CID_PROPERTY, RELEASE_THUMBNAIL_CID_PROPERTY } from '../src/constants';
import type { ReleaseData } from '../src/types';
import { AccessError } from '@peerbit/crypto';
import { waitForResolved } from '@peerbit/time';
import { SearchRequest } from '@peerbit/document';
import {
  PublicKeyAccessCondition,
  AccessType,
  type Access,
} from '@peerbit/identity-access-controller';

describe('Site ACL', () => {
  let session: TestSession;
  let peer1: ProgramClient, peer2: ProgramClient;
  let site1: Site | undefined;

  beforeAll(async () => {
    session = await TestSession.connected(3);
    peer1 = session.peers[0]; // Admin/Root trust
    peer2 = session.peers[1]; // Regular user
  }, 30000);

  afterEach(async () => {
    if (site1 && !site1.closed) {
      await site1.close();
    }
    site1 = undefined;
  });

  afterAll(async () => {
    await session.stop();
  });

  const releaseData: ReleaseData = {
    [RELEASE_NAME_PROPERTY]: 'TPB AFK: The Pirate Bay Away from Keyboard',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'movie',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmPSGARS6emPSEf8umwmjdG8AS7z7o8Nd36258B3BMi291',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: 'bafkreiemqveqhpksefhup46d77iybtatf2vb2bgyak4hfydxaz5hxser34',
  };

  test('root trust (creator) can add releases', async () => {
    site1 = await peer1.open(new Site(DEFAULT_SITE_ID, peer1.identity.publicKey));
    const release = new Release(releaseData);
    await site1.addRelease(release);
    const fetched = await site1.getRelease(release.id);
    expect(fetched).toBeDefined();
    expect(fetched?.name).toEqual(releaseData.name);
  }, 15000);

  test('untrusted peer cannot add releases', async () => {
    site1 = await peer1.open(new Site(DEFAULT_SITE_ID, peer1.identity.publicKey));
    const site1Address = site1.address!;
    const site2 = await peer2.open<Site>(site1Address, { args: { replicate: true } });

    await site1.waitFor(peer2.identity.publicKey);
    await site2.waitFor(peer1.identity.publicKey);

    const release = new Release(releaseData);
    await expect(site2.addRelease(release)).rejects.toThrow(AccessError);
    await site2.close();
  }, 20000);

  test('peer granted write access can add releases', async () => {
    site1 = await peer1.open(new Site(DEFAULT_SITE_ID, peer1.identity.publicKey));
    // Use Site's method to grant access
    await site1.grantWriteAccess(peer2.identity.publicKey);

    const site1Address = site1.address!;
    const site2 = await peer2.open<Site>(site1Address, { args: { replicate: true } });

    await site1.waitFor(peer2.identity.publicKey);
    await site2.waitFor(peer1.identity.publicKey);

    // Verify ACL state by querying the IdentityAccessController's 'access' store directly
    await waitForResolved(async () => {
      const accesses = await site2.acl.access.index.search(new SearchRequest({ query: [] }));
      const pkCondition = accesses.find(a =>
        a.accessCondition instanceof PublicKeyAccessCondition &&
        (a.accessCondition as PublicKeyAccessCondition<Access>).key.equals(peer2.identity.publicKey) &&
        (a.accessTypes.includes(AccessType.Write) || a.accessTypes.includes(AccessType.Any)),
      );
      expect(pkCondition).toBeDefined();
    }, { timeout: 15000, delayInterval: 1000 });

    const releaseByPeer2Data: ReleaseData = { ...releaseData, name: 'Release by Peer2' };
    const release = new Release(releaseByPeer2Data);
    await site2.addRelease(release);

    await waitForResolved(async () => {
      const fetchedOnSite1 = await site1!.getRelease(release.id);
      expect(fetchedOnSite1).toBeDefined();
      expect(fetchedOnSite1?.name).toEqual(releaseByPeer2Data.name);
    }, { timeout: 10000, delayInterval: 1000 });
    await site2.close();
  }, 30000);


});
