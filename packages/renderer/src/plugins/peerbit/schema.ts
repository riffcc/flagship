import { deserialize, field, option, serialize, variant } from '@dao-xyz/borsh';
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

  constructor(
    name: string,
    categoryId: string,
    contentCID: string,
    thumbnailCID?: string,
    metadata?: string
  ) {
    this[RELEASE_ID_PROPERTY] = uuid();
    this[RELEASE_NAME_PROPERTY] = name;
    this[RELEASE_CATEGORY_ID_PROPERTY] = categoryId;
    this[RELEASE_CONTENT_CID_PROPERTY] = contentCID;
    if (thumbnailCID) {
      this[RELEASE_THUMBNAIL_CID_PROPERTY] = thumbnailCID;
    }
    if (metadata) {
      this[RELEASE_METADATA_PROPERTY] = metadata;
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
    this.id = release.id;
    this.name = release.name;
    this.categoryId = release.categoryId;
    this.contentCID = release.contentCID;
    this.thumbnailCID = release.thumbnailCID;
    this.metadata = release.metadata;
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

  constructor(
    releaseId: string,
    startTime: string,
    endTime: string,
    promoted: boolean,
  ) {
    this.id = uuid();
    this.releaseId = releaseId;
    this.startTime = startTime;
    this.endTime = endTime;
    this.promoted = promoted;
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

  constructor(
    name: string,
    featured: boolean,
    description?: string,
    metadataSchema?: string,
  ) {
    this.id = uuid();
    this.name = name;
    this.featured = featured;
    this.description = description;
    this.metadataSchema = metadataSchema;
  }
}

enum AccounType {
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
  type: AccounType;


  constructor(publicKey: PublicSignKey, name: string, type: AccounType) {
    this.id = serialize(publicKey);
    this.name = name;
    this.type = type;
  }

  get publicKey() {
    return deserialize(this.id, PublicSignKey);
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

  constructor(siteId: string, name: string) {
    this.id = uuid();
    this.siteId = siteId;
    this.name = name;
  }
}

@variant(0)
export class BlockedContent {
  @field({ type: 'string' })
  [BLOCKED_CONTENT_ID_PROPERTY]: string;

  @field({ type: 'string' })
  [BLOCKED_CONTENT_CID_PROPERTY]: string;

  constructor(cid: string) {
    this.id = uuid();
    this.cid = cid;
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

  constructor(siteId: string) {
    super();
    this.releases = new Documents({
      id: sha256Sync(concat(siteId, new TextEncoder().encode('releases'))),
    });

    this.featuredReleases = new Documents({
      id: sha256Sync(concat(siteId, new TextEncoder().encode('featuredReleases'))),
    });

    this.contentCategories = new Documents({
      id: sha256Sync(concat(siteId, new TextEncoder().encode('contentCategories'))),
    });

    this.users = new Documents({
      id: sha256Sync(concat(siteId, new TextEncoder().encode('users'))),
    });

    this.subscriptions = new Documents({
      id: sha256Sync(concat(siteId, new TextEncoder().encode('subscriptions'))),
    });

    this.blockedContent = new Documents({
      id: sha256Sync(concat(siteId, new TextEncoder().encode('blockedContent'))),
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
}
