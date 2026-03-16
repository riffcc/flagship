/* eslint-disable @typescript-eslint/no-explicit-any */
/**
 * Citadel SDK Types
 * Lightweight TypeScript definitions for the Citadel API
 */

// Base types
export type AnyObject = any;

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
  contentCID?: string;
  thumbnailCID?: string;
  metadata?: T | AnyObject;
  siteAddress?: string;
  postedBy?: string;
  artistId?: string;
  /** Moderation status - defaults to 'approved' for backward compatibility */
  status?: ReleaseStatus;
  [key: string]: unknown;
}

export interface Release extends ImmutableProps, ReleaseData {
  id: string;
  name: string;
  categoryId: string;
  categorySlug?: string;
  contentCID?: string;
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
  startTime: string;
  endTime: string;
  promoted?: boolean;
  priority?: number;
  order?: number;
  customTitle?: string;
  customDescription?: string;
  customThumbnail?: string;
  regions?: string[] | null;
  languages?: string[] | null;
  tags?: string[];
  variant?: string;
  metadata?: Record<string, unknown>;
  // Analytics (read-only, set by backend)
  views?: number;
  clicks?: number;
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
  [key: string]: any;
}

export interface ContentCategoryData<T = ContentCategoryMetadataField> {
  name: string;
  slug: string;
  description?: string;
  icon?: string;
  metadataFields?: T[];
  metadataSchema?: T[] | Record<string, unknown> | string;
  displayName?: string;
  categoryId?: string;
  featured?: boolean;
  siteAddress?: string;
  allIds?: string[];
  parentId?: string;
  [key: string]: unknown;
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
  endpoint?: string;
  type?: 'webhook' | 'websocket';
  events?: string[];
  active?: boolean;
  secret?: string;
  to?: string;
  [key: string]: unknown;
}

export interface Subscription extends ImmutableProps, SubscriptionData {}

// API response types
export interface IdResponse {
  id: string;
  success: boolean;
  error?: string;
}

export interface HashResponse {
  hash: string;
  success: boolean;
  error?: string;
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
  fetch?: number;
  retry?: boolean;
  [key: string]: unknown;
}

// Input types for mutations
export interface AddInput<T = AnyObject> {
  name?: string;
  categoryId?: string;
  categorySlug?: string;
  contentCID?: string;
  thumbnailCID?: string;
  metadata?: T | AnyObject;
  status?: ReleaseStatus; // For admin uploads to moderation queue
  [key: string]: unknown;
}

export interface EditInput<T = AnyObject> {
  id: string;
  name?: string;
  categoryId?: string;
  categorySlug?: string;
  contentCID?: string;
  thumbnailCID?: string;
  metadata?: T | AnyObject;
  siteAddress?: string;
  postedBy?: string;
  artistId?: string;
  [key: string]: unknown;
}

// Site types
export interface SiteData {
  name: string;
  address: string;
  description?: string;
  logo?: string;
  url?: string;
  theme?: AnyObject;
}

export interface Site extends ImmutableProps, SiteData {}
