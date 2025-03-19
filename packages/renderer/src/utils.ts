import type { Constellation } from '@constl/ipa';
import type { FeaturedItem } from '/@/composables/staticReleases';
import { inject } from 'vue';
import {base16} from 'multiformats/bases/base16';
import {CID} from 'multiformats/cid';
import {cid as isCID} from 'is-ipfs';
import { IPFS_GATEWAY } from './constants/ipfs';
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

export const RIFFCC_PROTOCOL = 'Riff.CC';

export const useConstellation = (): {constl: Constellation} => {
  const constl = inject<Constellation>('constl');
  if (constl) return {constl};
  throw new Error("Constellation n'est pas trouvable.");
};

export const formatTime = (ms: number): string => {
  if (ms === 0 || isNaN(ms)) {
    return '00:00';
  }

  let totalSeconds = Math.floor(ms);
  const hours = Math.floor(totalSeconds / 3600);
  totalSeconds %= 3600;
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  const formattedHours = hours > 0 ? (hours < 10 ? `0${hours}` : `${hours}`) : null;
  const formattedMinutes = minutes < 10 ? `0${minutes}` : `${minutes}`;
  const formattedSeconds = seconds < 10 ? `0${seconds}` : `${seconds}`;

  return formattedHours
    ? `${formattedHours}:${formattedMinutes}:${formattedSeconds}`
    : `${formattedMinutes}:${formattedSeconds}`;
};

// Colors
export const lensColorHash = (sourceSite: string): string => {
  const idSite = sourceSite.replace('/orbitdb/', '');
  console.log('#' + CID.parse(idSite).toString(base16.encoder));
  return '#' + CID.parse(idSite).toString(base16.encoder).slice(-6);
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

export function filterActivedFeature(featured: FeaturedItem) {
  const now = new Date();
  const startTime = new Date(featured.startTime);
  const endTime = new Date(featured.endTime);

  return now >= startTime && now <= endTime;
};

export function parseUrlOrCid(urlOrCid?: string) {
  if (!urlOrCid) return undefined;
  return isCID(urlOrCid) ? `https://${IPFS_GATEWAY}/ipfs/${urlOrCid}` : urlOrCid;
};
