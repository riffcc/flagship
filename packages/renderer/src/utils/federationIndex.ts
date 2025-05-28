import type { IndexableFederationEntry } from '@riffcc/lens-sdk';
import type { ReleaseItem, AnyObject } from '/@/types';

/**
 * Convert a federation index entry to a release item format for UI compatibility
 */
export function federationEntryToRelease(entry: IndexableFederationEntry): ReleaseItem<AnyObject> {
  // Extract metadata from the entry
  const metadata: AnyObject = {
    description: entry.description,
    tags: entry.tags,
    contentType: entry.contentType,
    sourceSite: entry.sourceSiteName,
    sourceSiteId: entry.sourceSiteId,
  };

  // Create a release-like object from the federation index entry
  return {
    id: entry.id, // This is sourceSiteId:contentCid
    name: entry.title,
    categoryId: entry.categoryId,
    contentCID: entry.contentCid,
    thumbnailCID: entry.thumbnailCid,
    metadata: JSON.stringify(metadata),
    // Federation info
    federatedFrom: entry.sourceSiteId,
    federatedAt: new Date(entry.timestamp).toISOString(),
    federatedRealtime: true,
    // Context fields that might be used by UI
    created: BigInt(entry.timestamp),
    modified: BigInt(entry.timestamp),
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
 * Extract featured entries from federation index based on tags or recent high-quality content
 */
export function extractFeaturedFromIndex(entries: IndexableFederationEntry[]): IndexableFederationEntry[] {
  // For now, return the most recent entries as "featured"
  // In production, you might want to look for specific tags or quality indicators
  return entries
    .sort((a, b) => b.timestamp - a.timestamp)
    .slice(0, 10); // Top 10 most recent
}