import type { IndexableFederationEntry } from '@riffcc/lens-sdk';
import type { ReleaseItem, AnyObject } from '/@/types';

/**
 * Convert a federation index entry to a release item format for UI compatibility
 */
export function federationEntryToRelease(entry: IndexableFederationEntry): ReleaseItem<AnyObject> {
  // Extract metadata from the entry
  const metadata: AnyObject = {
    sourceSiteId: entry.sourceSiteId,
    timestamp: Number(entry.timestamp),
    isFeatured: entry.isFeatured,
    isPromoted: entry.isPromoted,
    featuredUntil: entry.featuredUntil ? Number(entry.featuredUntil) : undefined,
    promotedUntil: entry.promotedUntil ? Number(entry.promotedUntil) : undefined,
  };

  // Create a release-like object from the federation index entry
  return {
    id: entry.id, // This is sourceSiteId:contentCid
    name: entry.title,
    categoryId: 'video', // Default to video for now
    contentCid: entry.contentCID, // Note: uppercase CID in entry
    thumbnailCid: entry.thumbnailCID, // Note: uppercase CID in entry
    metadata,
  } as ReleaseItem<AnyObject>;
}

/**
 * Convert multiple federation entries to release format
 */
export function federationEntriesToReleases(entries: IndexableFederationEntry[]): ReleaseItem<AnyObject>[] {
  return entries.map(federationEntryToRelease);
}

/**
 * Check if we should use federation index based on site configuration
 */
export function shouldUseFederationIndex(): boolean {
  // This will be determined by whether the site has federation index enabled
  // For now, we'll check if the federation index methods are available
  return true; // Enable by default for new sites
}

/**
 * Extract featured entries from federation index based on isFeatured flag
 */
export function extractFeaturedFromIndex(entries: IndexableFederationEntry[]): IndexableFederationEntry[] {
  const now = Date.now();
  return entries
    .filter(entry => entry.isFeatured && (!entry.featuredUntil || Number(entry.featuredUntil) > now))
    .sort((a, b) => Number(b.timestamp) - Number(a.timestamp))
    .slice(0, 10); // Top 10 featured
}