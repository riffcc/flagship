import { Documents, SearchRequest, Sort, SortDirection } from '@peerbit/document';
import { field, option, variant } from '@dao-xyz/borsh';
import { type PublicSignKey, sha256Sync } from '@peerbit/crypto';
import { Program } from '@peerbit/program';
import type { ReplicationOptions } from '@peerbit/shared-log';
import {
  IdentityAccessController,
  Access,
  AccessType,
  PublicKeyAccessCondition,
} from '@peerbit/identity-access-controller';
import { v4 as uuid } from 'uuid';
import { concat } from 'uint8arrays';
import {
  ID_PROPERTY,
  RELEASE_NAME_PROPERTY,
  RELEASE_CATEGORY_ID_PROPERTY,
  RELEASE_CONTENT_CID_PROPERTY,
  RELEASE_THUMBNAIL_CID_PROPERTY,
  RELEASE_METADATA_PROPERTY,
  FEATURED_RELEASE_ID_PROPERTY,
  FEATURED_START_TIME_PROPERTY,
  FEATURED_END_TIME_PROPERTY,
  FEATURED_PROMOTED_PROPERTY,
  CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY,
  CONTENT_CATEGORY_DESCRIPTION_PROPERTY,
  CONTENT_CATEGORY_FEATURED_PROPERTY,
  CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY,
  SUBSCRIPTION_SITE_ID_PROPERTY,
  SUBSCRIPTION_NAME_PROPERTY,
  BLOCKED_CONTENT_CID_PROPERTY,
} from './constants';

import type {
  IdData,
  ReleaseData,
  FeaturedReleaseData,
  ContentCategoryData,
  SubcriptionData,
  BlockedContentData,
} from './types';

export enum AccountType {
  ADMIN = 'ADMIN',
  MODERATOR = 'MODERATOR',
  USER = 'USER',
  GUEST = 'GUEST'
}


@variant(0)
export class Release {
  @field({ type: 'string' })
  [ID_PROPERTY]: string;

  @field({ type: 'string' })
  [RELEASE_NAME_PROPERTY]: string;

  @field({ type: 'string' })
  [RELEASE_CATEGORY_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [RELEASE_CONTENT_CID_PROPERTY]: string;

  @field({ type: option('string') })
  [RELEASE_THUMBNAIL_CID_PROPERTY]?: string;

  @field({ type: option('string') })
  [RELEASE_METADATA_PROPERTY]?: string;

  constructor(props: ReleaseData) {
    this[ID_PROPERTY] = uuid();
    this[RELEASE_NAME_PROPERTY] = props[RELEASE_NAME_PROPERTY];
    this[RELEASE_CATEGORY_ID_PROPERTY] = props[RELEASE_CATEGORY_ID_PROPERTY];
    this[RELEASE_CONTENT_CID_PROPERTY] = props[RELEASE_CONTENT_CID_PROPERTY];
    if (props[RELEASE_THUMBNAIL_CID_PROPERTY]) {
      this[RELEASE_THUMBNAIL_CID_PROPERTY] = props[RELEASE_THUMBNAIL_CID_PROPERTY];
    }
    if (props[RELEASE_METADATA_PROPERTY]) {
      this[RELEASE_METADATA_PROPERTY] = props[RELEASE_METADATA_PROPERTY];
    }
  }
}

export class IndexableRelease {
  @field({ type: 'string' })
  [ID_PROPERTY]: string;

  @field({ type: 'string' })
  [RELEASE_NAME_PROPERTY]: string;

  @field({ type: 'string' })
  [RELEASE_CATEGORY_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [RELEASE_CONTENT_CID_PROPERTY]: string;

  @field({ type: option('string') })
  [RELEASE_THUMBNAIL_CID_PROPERTY]?: string;

  @field({ type: option('string') })
  [RELEASE_METADATA_PROPERTY]?: string;

  @field({ type: 'u64' })
  created: bigint;

  @field({ type: 'u64' })
  modified: bigint;

  @field({ type: Uint8Array })
  author: Uint8Array;

  constructor(
    release: IdData & ReleaseData,
    createdAt: bigint,
    modified: bigint,
    author: PublicSignKey,
  ) {
    this[ID_PROPERTY] = release[ID_PROPERTY];
    this[RELEASE_NAME_PROPERTY] = release[RELEASE_NAME_PROPERTY];
    this[RELEASE_CATEGORY_ID_PROPERTY] = release[RELEASE_CATEGORY_ID_PROPERTY];
    this[RELEASE_CONTENT_CID_PROPERTY] = release[RELEASE_CONTENT_CID_PROPERTY];
    if (release[RELEASE_THUMBNAIL_CID_PROPERTY]) {
      this[RELEASE_THUMBNAIL_CID_PROPERTY] = release[RELEASE_THUMBNAIL_CID_PROPERTY];
    }
    if (release[RELEASE_METADATA_PROPERTY]) {
      this[RELEASE_METADATA_PROPERTY] = release[RELEASE_METADATA_PROPERTY];
    }
    this.created = createdAt;
    this.modified = modified;
    this.author = author.bytes;
  }
}

@variant(0)
export class FeaturedRelease {
  @field({ type: 'string' })
  [ID_PROPERTY]: string;

  @field({ type: 'string' })
  [FEATURED_RELEASE_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [FEATURED_START_TIME_PROPERTY]: string;

  @field({ type: 'string' })
  [FEATURED_END_TIME_PROPERTY]: string;

  @field({ type: 'bool' })
  [FEATURED_PROMOTED_PROPERTY]: boolean;

  constructor(props: FeaturedReleaseData) {
    this[ID_PROPERTY] = uuid();
    this[FEATURED_RELEASE_ID_PROPERTY] = props[FEATURED_RELEASE_ID_PROPERTY];
    this[FEATURED_START_TIME_PROPERTY] = props[FEATURED_START_TIME_PROPERTY];
    this[FEATURED_END_TIME_PROPERTY] = props[FEATURED_END_TIME_PROPERTY];
    this[FEATURED_PROMOTED_PROPERTY] = props[FEATURED_PROMOTED_PROPERTY];
  }
}

@variant(0)
export class ContentCategory {
  @field({ type: 'string' })
  [ID_PROPERTY]: string;

  @field({ type: 'string' })
  [CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY]: string;

  @field({ type: 'bool' })
  [CONTENT_CATEGORY_FEATURED_PROPERTY]: boolean;

  @field({ type: option('string') })
  [CONTENT_CATEGORY_DESCRIPTION_PROPERTY]?: string;

  @field({ type: option('string') })
  [CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY]?: string;

  constructor(props: ContentCategoryData) {
    this[ID_PROPERTY] = props[ID_PROPERTY];
    this[CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY] = props[CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY];
    this[CONTENT_CATEGORY_FEATURED_PROPERTY] = props[CONTENT_CATEGORY_FEATURED_PROPERTY];
    if (props[CONTENT_CATEGORY_DESCRIPTION_PROPERTY]) {
      this[CONTENT_CATEGORY_DESCRIPTION_PROPERTY] = props[CONTENT_CATEGORY_DESCRIPTION_PROPERTY];
    }
    if (props[CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY]) {
      this[CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY] = props[CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY];
    }
  }
}

@variant(0)
export class Subscription {
  @field({ type: 'string' })
  [ID_PROPERTY]: string;

  @field({ type: 'string' })
  [SUBSCRIPTION_SITE_ID_PROPERTY]: string;

  @field({ type: option('string') })
  [SUBSCRIPTION_NAME_PROPERTY]?: string;

  constructor(props: SubcriptionData) {
    this[ID_PROPERTY] = uuid();
    this[SUBSCRIPTION_SITE_ID_PROPERTY] = props[SUBSCRIPTION_SITE_ID_PROPERTY];
    if (props[SUBSCRIPTION_NAME_PROPERTY]) {
      this[SUBSCRIPTION_NAME_PROPERTY] = props[SUBSCRIPTION_NAME_PROPERTY];
    }
  }
}

@variant(0)
export class BlockedContent {
  @field({ type: 'string' })
  [ID_PROPERTY]: string;

  @field({ type: 'string' })
  [BLOCKED_CONTENT_CID_PROPERTY]: string;

  constructor(props: BlockedContentData) {
    this[ID_PROPERTY] = uuid();
    this[BLOCKED_CONTENT_CID_PROPERTY] = props[BLOCKED_CONTENT_CID_PROPERTY];
  }
}

type Args = {
  replicate?: ReplicationOptions
};

@variant('site')
export class Site extends Program<Args> {

  @field({ type: Documents })
  releases: Documents<Release, IndexableRelease>;

  @field({ type: Documents })
  featuredReleases: Documents<FeaturedRelease>;

  @field({ type: Documents })
  contentCategories: Documents<ContentCategory>;

  @field({ type: Documents })
  subscriptions: Documents<Subscription>;

  @field({ type: Documents })
  blockedContent: Documents<BlockedContent>;

  @field({ type: IdentityAccessController })
  acl: IdentityAccessController;

  constructor(siteIdString: string, rootTrust: PublicSignKey) {
    super();
    const textEncoder = new TextEncoder();
    const siteIdBytes = textEncoder.encode(siteIdString);

    const aclIdSuffix = textEncoder.encode('acl');
    const releasesSuffix = textEncoder.encode('releases');
    const featuredReleasesSuffix = textEncoder.encode('featuredReleases');
    const contentCategoriesSuffix = textEncoder.encode('contentCategories');
    const subscriptionsSuffix = textEncoder.encode('subscriptions');
    const blockedContentSuffix = textEncoder.encode('blockedContent');

    this.acl = new IdentityAccessController({
      id: sha256Sync(concat([siteIdBytes, aclIdSuffix])),
      rootTrust: rootTrust,
    });

    this.releases = new Documents({
      id: sha256Sync(concat([siteIdBytes, releasesSuffix])),
    });

    this.featuredReleases = new Documents({
      id: sha256Sync(concat([siteIdBytes, featuredReleasesSuffix])),
    });

    this.contentCategories = new Documents({
      id: sha256Sync(concat([siteIdBytes, contentCategoriesSuffix])),
    });

    this.subscriptions = new Documents({
      id: sha256Sync(concat([siteIdBytes, subscriptionsSuffix])),
    });

    this.blockedContent = new Documents({
      id: sha256Sync(concat([siteIdBytes, blockedContentSuffix])),
    });
  }

  async open(args: Args): Promise<void> {

    const defaultReplicationOptions = args?.replicate || { factor: 1 };
    const defaultReplicaSettings = { min: 2, max: undefined };

    await this.acl.open({ replicate: args?.replicate || { factor: 1 } });

    const commonCanPerform = this.acl.canPerform.bind(this.acl);

    await this.releases.open({
      type: Release,
      replicate: defaultReplicationOptions,
      replicas: defaultReplicaSettings,
      canPerform: commonCanPerform,
      index: {
        canRead: async () => {
          return true;
        },
        type: IndexableRelease,
        transform: async (release, ctx) => {
          return new IndexableRelease(
            release,
            ctx.created,
            ctx.modified,
            (await this.releases.log.log.get(
              ctx.head,
            ))!.signatures[0].publicKey,
          );
        },
      },
    });

    await this.featuredReleases.open({
      type: FeaturedRelease,
      replicate: defaultReplicationOptions,
      replicas: defaultReplicaSettings,
      canPerform: commonCanPerform,
      index: {
        canRead: async () => {
          return true;
        },
      },
    });

    await this.contentCategories.open({
      type: ContentCategory,
      replicate: defaultReplicationOptions,
      replicas: defaultReplicaSettings,
      canPerform: commonCanPerform,
      index: {
        canRead: async () => {
          return true;
        },
      },
    });

    await this.subscriptions.open({
      type: Subscription,
      replicate: defaultReplicationOptions,
      replicas: defaultReplicaSettings,
      canPerform: commonCanPerform,
    });

    await this.blockedContent.open({
      type: BlockedContent,
      replicate: defaultReplicationOptions,
      replicas: defaultReplicaSettings,
      canPerform: commonCanPerform,
    });
  }

  async addRelease(release: Release): Promise<string> {
    const result = await this.releases.put(release);
    return result.entry.hash;
  }

  async getRelease(id: string): Promise<Release | undefined> {
    return this.releases.index.get(id);
  }

  async getLatestReleases(size = 30): Promise<Release[]> {
    return this.releases.index.search(
      new SearchRequest({
        sort: [
          new Sort({ key: 'created', direction: SortDirection.DESC }),
        ],
        fetch: size,
      }),
    );
  }
  // --- ACL Management Methods ---
  async grantWriteAccess(identity: PublicSignKey): Promise<void> {
    const access = new Access({
      accessCondition: new PublicKeyAccessCondition({ key: identity }),
      accessTypes: [AccessType.Write, AccessType.Read],
    });
    await this.acl.access.put(access.initialize());
  }

  async grantAdminAccess(identity: PublicSignKey): Promise<void> {
    const access = new Access({
      accessCondition: new PublicKeyAccessCondition({ key: identity }),
      accessTypes: [AccessType.Any],
    });
    await this.acl.access.put(access.initialize());
  }

  async grantReadAccess(identity: PublicSignKey): Promise<void> {
    const access = new Access({
      accessCondition: new PublicKeyAccessCondition({ key: identity }),
      accessTypes: [AccessType.Read],
    });
    await this.acl.access.put(access.initialize());
  }

  async addTrustedIdentity(identity: PublicSignKey): Promise<void> {
    await this.acl.trustedNetwork.add(identity);
  }

  async addIdentityRelation(from: PublicSignKey, to: PublicSignKey): Promise<void> {
    if (!this.node.identity.publicKey.equals(from)) {
        throw new Error("addIdentityRelation must be called by the 'from' identity's peer, or the ACL for IdentityGraph needs to allow this peer to act on behalf of 'from'.");
    }
    await this.acl.identityGraphController.addRelation(to);
  }
}
