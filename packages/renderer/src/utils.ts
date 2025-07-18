import {base16} from 'multiformats/bases/base16';
import {CID} from 'multiformats/cid';
import {cid as isCID} from 'is-ipfs';
import { IPFS_GATEWAY } from './constants/ipfs';
import type { FeaturedReleaseItem } from './types';
import {Duration} from 'luxon';

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

export function parseUrlOrCid(urlOrCid?: string): string | undefined {
  if (!urlOrCid) return undefined;
  if (!isCID(urlOrCid)) {
    return urlOrCid;
  }

  // Load optional gateway override from environment variable
  const gatewayOverride = import.meta.env.VITE_IPFS_GATEWAY as string | undefined;
  const selectedGateway = gatewayOverride || IPFS_GATEWAY;

  // Use HTTPS for gateways, unless the override specifies a protocol
  const gatewayBase = selectedGateway.startsWith('http://') || selectedGateway.startsWith('https://')
    ? selectedGateway
    : `https://${selectedGateway}`;

  // For codex gateway, we'll maintain the 'codex-' prefix logic if the base gateway doesn't already include it.
  // If the override is a full URL, we might need a more sophisticated way to derive the codex variant.
  // For now, let's assume the override is a domain or IP:port.
  const codexGatewayBase = gatewayOverride
    ? (gatewayOverride.startsWith('http://') || gatewayOverride.startsWith('https://') ? gatewayOverride.replace(/^(https?:\/\/)/, '$1codex-') : `https://codex-${gatewayOverride}`)
    : `https://codex-${IPFS_GATEWAY}`;


  if (urlOrCid.startsWith('zD')) {
    // If the gateway override is a full URL, we might not want to append /api/codex...
    // This logic assumes the override is a base gateway.
    return `${codexGatewayBase}/api/codex/v1/data/${urlOrCid}/network/stream`;
  } else {
    return `${gatewayBase}/ipfs/${urlOrCid}`;
  }
};

export function filterActivedFeatured(featured: FeaturedReleaseItem) {
  const now = new Date();
  const startTime = new Date(featured.startTime);
  const endTime = new Date(featured.endTime);

  return now >= startTime && now <= endTime;
};

export function filterPromotedFeatured(featured: FeaturedReleaseItem) {
  return featured.promoted;
};
