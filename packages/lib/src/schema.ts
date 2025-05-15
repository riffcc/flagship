import { SearchRequest, Sort, SortDirection } from '@peerbit/document';
import { deserialize, field, option, variant } from '@dao-xyz/borsh';
import { PublicSignKey } from '@peerbit/crypto';
import { sha256Sync } from '@peerbit/crypto';
import {
  Documents,

} from '@peerbit/document';
import { Program } from '@peerbit/program';
import { type ReplicationOptions } from '@peerbit/shared-log';
import { v4 as uuid } from 'uuid';
import { concat } from 'uint8arrays';

const RELEASE_ID_PROPERTY = 'id';
const RELEASE_NAME_PROPERTY = 'name';
const RELEASE_CATEGORY_ID_PROPERTY = 'categoryId';
const RELEASE_CONTENT_CID_PROPERTY = 'contentCID';
const RELEASE_THUMBNAIL_CID_PROPERTY = 'thumbnailCID';
const RELEASE_METADATA_PROPERTY = 'metadata';

const FEATURED_ID_PROPERTY = 'id';
const FEATURED_RELEASE_ID_PROPERTY = 'releaseId';
const FEATURED_START_TIME_PROPERTY = 'startTime';
const FEATURED_END_TIME_PROPERTY = 'endTime';
const FEATURED_PROMOTED_PROPERTY = 'promoted';

const CONTENT_CATEGORY_ID_PROPERTY = 'id';
const CONTENT_CATEGORY_NAME_PROPERTY = 'name';
const CONTENT_CATEGORY_DESCRIPTION_PROPERTY = 'description';
const CONTENT_CATEGORY_FEATURED_PROPERTY = 'featured';
const CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY = 'metadataSchema';

const SUBSCRIPTION_ID_PROPERTY = 'id';
const SUBSCRIPTION_SITE_ID_PROPERTY = 'siteId';
const SUBSCRIPTION_NAME_PROPERTY = 'name';

const BLOCKED_CONTENT_ID_PROPERTY = 'id';
const BLOCKED_CONTENT_CID_PROPERTY = 'cid';

@variant(0)
export class Release {
  @field({ type: 'string' })
  [RELEASE_ID_PROPERTY]: string;

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

  constructor(props: {
    [RELEASE_NAME_PROPERTY]: string;
    [RELEASE_CATEGORY_ID_PROPERTY]: string;
    [RELEASE_CONTENT_CID_PROPERTY]: string;
    [RELEASE_THUMBNAIL_CID_PROPERTY]?: string;
    [RELEASE_METADATA_PROPERTY]?: string;
  }) {
    this[RELEASE_ID_PROPERTY] = uuid();
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
  [RELEASE_ID_PROPERTY]: string;

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
    release: Release,
    createdAt: bigint,
    modified: bigint,
    author: PublicSignKey,
  ) {
    this[RELEASE_ID_PROPERTY] = release[RELEASE_ID_PROPERTY];
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
  [FEATURED_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [FEATURED_RELEASE_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [FEATURED_START_TIME_PROPERTY]: string;

  @field({ type: 'string' })
  [FEATURED_END_TIME_PROPERTY]: string;

  @field({ type: 'bool' })
  [FEATURED_PROMOTED_PROPERTY]: boolean;

  constructor(props: {
    releaseId: string;
    startTime: string;
    endTime: string;
    promoted: boolean;
  }) {
    this.id = uuid();
    this.releaseId = props[FEATURED_RELEASE_ID_PROPERTY];
    this.startTime = props[FEATURED_START_TIME_PROPERTY];
    this.endTime = props[FEATURED_END_TIME_PROPERTY];
    this.promoted = props[FEATURED_PROMOTED_PROPERTY];
  }
}

@variant(0)
export class ContentCategory {
  @field({ type: 'string' })
  [CONTENT_CATEGORY_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [CONTENT_CATEGORY_NAME_PROPERTY]: string;

  @field({ type: 'bool' })
  [CONTENT_CATEGORY_FEATURED_PROPERTY]: boolean;

  @field({ type: option('string') })
  [CONTENT_CATEGORY_DESCRIPTION_PROPERTY]?: string;

  @field({ type: option('string') })
  [CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY]?: string;

  constructor(props: {
    name: string;
    featured: boolean;
    description?: string;
    metadataSchema?: string;
  }) {
    this[CONTENT_CATEGORY_ID_PROPERTY] = uuid();
    this[CONTENT_CATEGORY_NAME_PROPERTY] = props[CONTENT_CATEGORY_NAME_PROPERTY];
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
  [SUBSCRIPTION_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [SUBSCRIPTION_SITE_ID_PROPERTY]: string;

  @field({ type: option('string') })
  [SUBSCRIPTION_NAME_PROPERTY]?: string;

  constructor(props: { siteId: string, name?: string }) {
    this[SUBSCRIPTION_ID_PROPERTY] = uuid();
    this[SUBSCRIPTION_SITE_ID_PROPERTY] = props[SUBSCRIPTION_SITE_ID_PROPERTY];
    if (props[SUBSCRIPTION_NAME_PROPERTY]) {
      this[SUBSCRIPTION_NAME_PROPERTY] = props[SUBSCRIPTION_NAME_PROPERTY];
    }
  }
}

@variant(0)
export class BlockedContent {
  @field({ type: 'string' })
  [BLOCKED_CONTENT_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [BLOCKED_CONTENT_CID_PROPERTY]: string;

  constructor(cid: string) {
    this[BLOCKED_CONTENT_ID_PROPERTY] = uuid();
    this[BLOCKED_CONTENT_CID_PROPERTY] = cid;
  }
}


export enum AccountType {
  GUEST = 0,
  USER = 1,
  MODERATOR = 2,
  ADMIN = 3,
}

@variant(0)
export class Account {
  @field({ type: Uint8Array })
  id: Uint8Array;

  @field({ type: 'string' })
  name: string;

  @field({ type: 'u8' })
  type: AccountType;


  constructor(publicKey: PublicSignKey, name: string, type: AccountType) {
    this.id = publicKey.bytes;
    this.name = name;
    this.type = type;
  }

  get publicKey() {
    return deserialize(this.id, PublicSignKey);
  }
}

type SiteArgs = { replicate?: ReplicationOptions };

@variant('site')
export class Site extends Program<SiteArgs> {

  @field({ type: Documents })
  releases: Documents<Release, IndexableRelease>;

  @field({ type: Documents })
  featuredReleases: Documents<FeaturedRelease>;

  @field({ type: Documents })
  contentCategories: Documents<ContentCategory>;

  @field({ type: Documents })
  users: Documents<Account>;

  @field({ type: Documents })
  subscriptions: Documents<Subscription>;

  @field({ type: Documents })
  blockedContent: Documents<BlockedContent>;

  constructor(siteIdString: string) {
    super();
    const textEncoder = new TextEncoder();
    const siteIdBytes = textEncoder.encode(siteIdString);

    const releasesSuffix = textEncoder.encode('releases');
    const featuredReleasesSuffix = textEncoder.encode('featuredReleases');
    const contentCategoriesSuffix = textEncoder.encode('contentCategories');
    const usersSuffix = textEncoder.encode('users');
    const subscriptionsSuffix = textEncoder.encode('subscriptions');
    const blockedContentSuffix = textEncoder.encode('blockedContent');

    this.releases = new Documents({
      id: sha256Sync(concat([siteIdBytes, releasesSuffix])),
    });

    this.featuredReleases = new Documents({
      id: sha256Sync(concat([siteIdBytes, featuredReleasesSuffix])),
    });

    this.contentCategories = new Documents({
      id: sha256Sync(concat([siteIdBytes, contentCategoriesSuffix])),
    });

    this.users = new Documents({
      id: sha256Sync(concat([siteIdBytes, usersSuffix])),
    });

    this.subscriptions = new Documents({
      id: sha256Sync(concat([siteIdBytes, subscriptionsSuffix])),
    });

    this.blockedContent = new Documents({
      id: sha256Sync(concat([siteIdBytes, blockedContentSuffix])),
    });
  }

  async open(): Promise<void> {
    await this.releases.open({
      type: Release,
      replicate: {
        factor: 1,
      },
      canPerform: async () => {
        //TODO: implement access control
        return true;
      },
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
      replicas: {
        min: 2,
        max: undefined,
      },
    });
    await this.featuredReleases.open({
      type: FeaturedRelease,
      canPerform: async () => {
        //TODO: implement access control
        return true;
      },
      replicate: {
        factor: 1,
      },
      replicas: {
        min: 2,
        max: undefined,
      },
    });
    await this.contentCategories.open({
      type: ContentCategory,
      canPerform: async () => {
        //TODO: implement access control
        return true;
      },
      replicate: {
        factor: 1,
      },
      replicas: {
        min: 2,
        max: undefined,
      },
    });
    await this.users.open({
      type: Account,
      canPerform: async () => {
        //TODO: implement access control
        return true;
      },
      replicate: {
        factor: 1,
      },
      replicas: {
        min: 2,
        max: undefined,
      },
    });
    await this.subscriptions.open({
      type: Subscription,
      canPerform: async () => {
        //TODO: implement access control
        return true;
      },
      replicate: {
        factor: 1,
      },
      replicas: {
        min: 2,
        max: undefined,
      },
    });
    await this.blockedContent.open({
      type: BlockedContent,
      canPerform: async () => {
        //TODO: implement access control
        return true;
      },
      replicate: {
        factor: 1,
      },
      replicas: {
        min: 2,
        max: undefined,
      },
    });
  }

  async addRelease(release: Release): Promise<string> {
    const result = await this.releases.put(release);
    return result.entry.hash;
  }

  async getRelease(id: string): Promise<Release> {
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
}
