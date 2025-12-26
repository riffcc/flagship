import {base16} from 'multiformats/bases/base16';
import {CID} from 'multiformats/cid';
import {cid as isCID} from 'is-ipfs';
import { RIFFCC_IPFS_GATEWAY, RIFFCC_ARCHIVIST_GATEWAY } from './constants/config';
import type { FeaturedReleaseItem } from './types';
import {Duration} from 'luxon';

// Archivist multicodec codes
// From archivist/archivisttypes.nim
const ARCHIVIST_MULTICODECS = new Set([
  0xCD01, // archivist-manifest
  0xCD02, // archivist-root (dataset root)
  0xCD03, // archivist-block
  0xCD04, // archivist-directory
  0xCD05, // archivist-slot-root
]);

/**
 * Detects if a CID is an Archivist-style CID.
 * Archivist CIDs use custom multicodecs (archivist-manifest, archivist-directory, archivist-block, etc.)
 *
 * We parse the CID to check its codec rather than relying on prefix matching,
 * which handles all encoding variations (base58btc, base32, etc.).
 */
export function isArchivistCid(cidStr: string): boolean {
  try {
    // Try to parse as a CID
    const parsed = CID.parse(cidStr);
    // Check if the codec is an Archivist multicodec
    return ARCHIVIST_MULTICODECS.has(parsed.code);
  } catch {
    // If parsing fails, fall back to prefix check for known patterns
    // zD and zE are common base58btc prefixes for Archivist CIDs
    return cidStr.startsWith('zD') || cidStr.startsWith('zE');
  }
}

/**
 * Gets the base URL for retrieving Archivist content.
 * Priority: VITE_ARCHIVIST_GATEWAY > VITE_ARCHIVIST_API_URL > default
 *
 * Use case:
 * - Production: VITE_ARCHIVIST_GATEWAY = https://cdn.riff.cc
 * - Dev/Airgapped: Only VITE_ARCHIVIST_API_URL = http://localhost:8080
 */
function getArchivistBaseUrl(): string {
  // Prefer dedicated gateway if set
  const gateway = import.meta.env.VITE_ARCHIVIST_GATEWAY as string | undefined;
  if (gateway) {
    return gateway.startsWith('http://') || gateway.startsWith('https://')
      ? gateway
      : `https://${gateway}`;
  }

  // Fall back to API URL for local/airgapped development
  const apiUrl = import.meta.env.VITE_ARCHIVIST_API_URL as string | undefined;
  if (apiUrl) {
    // Use first URL if comma-separated
    const firstUrl = apiUrl.split(',')[0].trim();
    return firstUrl.startsWith('http://') || firstUrl.startsWith('https://')
      ? firstUrl
      : `https://${firstUrl}`;
  }

  // Default to Riff.CC Archivist gateway
  return `https://${RIFFCC_ARCHIVIST_GATEWAY}`;
}

/**
 * Gets the base URL for retrieving IPFS content.
 */
function getIpfsBaseUrl(): string {
  const gateway = import.meta.env.VITE_IPFS_GATEWAY as string | undefined;
  const selected = gateway || RIFFCC_IPFS_GATEWAY;
  return selected.startsWith('http://') || selected.startsWith('https://')
    ? selected
    : `https://${selected}`;
}

export function downloadFile(filename: string, content: string | Uint8Array) {
  const element = document.createElement('a');

  let url: string;
  if (content instanceof Uint8Array) {
    url = URL.createObjectURL(new Blob([content.buffer as ArrayBuffer]));
  } else {
    url = content;
  }
  element.setAttribute('href', url);
  element.setAttribute('download', filename);

  element.style.display = 'none';
  document.body.appendChild(element);

  element.click();

  document.body.removeChild(element);
}

export function selectTranslation(options?: {[language: string]: string}): string | undefined {
  // Constellation has a multilingual-centric structure, but for now the Riff.CC site is monolingual,
  // so we'll just use any name. Once Riff.CC has an internationalised interface, we can match displayed
  // usernames with the viewer's chosen site language here, and do fancy stuff looking up fallback languages.

  // Another idea: we could also set up community translations of the Riff.CC site interface itself with
  // Kilimukku, which is a Constellation-based community translation software.
  return options && Object.keys(options).length ? Object.values(options)[0] : undefined;
}

export async function copyText(text: string | undefined) {
  if (!text) return;
  await navigator.clipboard.writeText(text);
}

export const formatTime = (ms: number): string => {
  if (ms === 0 || isNaN(ms)) {
    return '00:00';
  }

  const duration = Duration.fromObject({ seconds: ms });
  const hours = duration.as('hours');

  return (hours >= 1) ? duration.toFormat('hh:mm:ss') :  duration.toFormat('mm:ss');
};

// Colors
export const lensColorHash = (siteAddress: string): string => {
  console.log('#' + CID.parse(siteAddress).toString(base16.encoder));
  return '#' + CID.parse(siteAddress).toString(base16.encoder).slice(-6);
};
// export function getStatusColor(status: ItemStatus) {
//   if (status === 'pending') {
//     return 'blue';
//   } else if (status === 'approved') {
//     return 'green';
//   } else if (status === 'rejected') {
//     return 'red';
//   } else {
//     return 'default';
//   }
// };

export function isValidEmail(email: string) {
  const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return regex.test(email);
};

/**
 * Resolves a CID or URL to a full content URL.
 *
 * Routing logic:
 * - Archivist CIDs (zD prefix) → Archivist gateway (VITE_ARCHIVIST_GATEWAY or VITE_ARCHIVIST_API_URL)
 * - IPFS CIDs (Qm, bafy, etc.) → IPFS gateway (VITE_IPFS_GATEWAY)
 * - Already a URL → returns as-is
 */
export function parseUrlOrCid(urlOrCid?: string): string | undefined {
  if (!urlOrCid) return undefined;

  // If it's not a valid CID, assume it's already a URL
  if (!isCID(urlOrCid) && !isArchivistCid(urlOrCid)) {
    return urlOrCid;
  }

  // Route Archivist CIDs to Archivist gateway
  // Use /network/stream for buffering and network fetching
  if (isArchivistCid(urlOrCid)) {
    const archivistBase = getArchivistBaseUrl();
    return `${archivistBase}/api/archivist/v1/data/${urlOrCid}/network/stream`;
  }

  // Route IPFS CIDs to IPFS gateway
  const ipfsBase = getIpfsBaseUrl();
  return `${ipfsBase}/ipfs/${urlOrCid}`;
}

export function filterActivedFeatured(featured: FeaturedReleaseItem) {
  const now = new Date();
  const startTime = new Date(featured.startTime);
  const endTime = new Date(featured.endTime);

  return now >= startTime && now <= endTime;
};

export function filterPromotedFeatured(featured: FeaturedReleaseItem) {
  return featured.promoted;
};
