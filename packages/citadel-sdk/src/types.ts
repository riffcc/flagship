/**
 * Citadel SDK Types
 * Lightweight TypeScript definitions for the Citadel API
 */

// Base types
export type AnyObject = Record<string, unknown>;

export interface ImmutableProps {
  id: string;
  createdAt: string;
  updatedAt?: string;
}

// Release types

/**
 * Moderation status for releases
 */
export type ReleaseStatus = 'pending' | 'approved' | 'rejected';

export interface ReleaseData<T = AnyObject> {
  name: string;
  categoryId: string;
  categorySlug?: string;
  contentCID: string;
  thumbnailCID?: string;
  metadata?: T;
  siteAddress?: string;
  postedBy?: string;
  /** Moderation status - defaults to 'approved' for backward compatibility */
  status?: ReleaseStatus;
}

export interface Release extends ImmutableProps, ReleaseData {
  id: string;
  name: string;
  categoryId: string;
  categorySlug: string;
  contentCID: string;
  thumbnailCID?: string;
  metadata?: AnyObject;
  siteAddress: string;
  postedBy: string;
  createdAt: string;
  /** Moderation status */
  status: ReleaseStatus;
  /** Public key of moderator who approved/rejected */
  moderatedBy?: string;
  /** Timestamp of moderation action (ISO 8601) */
  moderatedAt?: string;
  /** Reason for rejection */
  rejectionReason?: string;
}

/**
 * Moderation statistics response
 */
export interface ModerationStats {
  pending: number;
  approved: number;
  rejected: number;
  total: number;
}

// Featured release types
export interface FeaturedReleaseData {
  releaseId: string;
  position: number;
  title?: string;
  description?: string;
}

export interface FeaturedRelease extends ImmutableProps, FeaturedReleaseData {}

// Content category types
export interface ContentCategoryMetadataField {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'date' | 'select';
  required?: boolean;
  options?: string[];
  description?: string;
  default?: unknown;
}

export interface ContentCategoryData<T = ContentCategoryMetadataField> {
  name: string;
  slug: string;
  description?: string;
  icon?: string;
  metadataFields?: T[];
  parentId?: string;
}

export interface ContentCategory extends ImmutableProps, ContentCategoryData {}

// Account types
export interface AccountStatusResponse {
  isAdmin: boolean;
  roles: string[];
  permissions: string[];
  publicKey?: string;
}

// Subscription types
export interface SubscriptionData {
  endpoint: string;
  type: 'webhook' | 'websocket';
  events: string[];
  active: boolean;
  secret?: string;
}

export interface Subscription extends ImmutableProps, SubscriptionData {}

// API response types
export interface IdResponse {
  id: string;
  success: boolean;
}

export interface HashResponse {
  hash: string;
  success: boolean;
}

// Search types
export interface SearchOptions {
  query?: string;
  categoryId?: string;
  categorySlug?: string;
  limit?: number;
  offset?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

// Input types for mutations
export interface AddInput<T = AnyObject> {
  name: string;
  categoryId: string;
  categorySlug?: string;
  contentCID: string;
  thumbnailCID?: string;
  metadata?: T;
  status?: ReleaseStatus; // For admin uploads to moderation queue
}

export interface EditInput<T = AnyObject> {
  id: string;
  name?: string;
  categoryId?: string;
  categorySlug?: string;
  contentCID?: string;
  thumbnailCID?: string;
  metadata?: T;
}

// Site types
export interface SiteData {
  name: string;
  address: string;
  description?: string;
  logo?: string;
  theme?: AnyObject;
}

export interface Site extends ImmutableProps, SiteData {}
