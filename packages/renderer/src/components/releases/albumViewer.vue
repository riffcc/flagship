<template>
  <!-- Full viewport background artwork (The Slip specific) -->
  <TransitionGroup name="artwork-fade">
    <div
      v-if="!isLoading && currentTrackArtwork"
      :key="currentTrackArtwork"
      class="track-artwork-background"
      :style="{
        backgroundImage: `linear-gradient(rgba(0, 0, 0, 0.3), rgba(0, 0, 0, 0.3)), url(${currentTrackArtwork})`,
      }"
    ></div>
  </TransitionGroup>

  <v-sheet
    v-if="isLoading"
    color="transparent"
    min-height="75vh"
    class="d-flex align-center justify-center"
  >
    <v-progress-circular indeterminate></v-progress-circular>
  </v-sheet>
  <v-sheet
    v-else
    class="text-body-2 mx-auto my-4 album-content"
    max-width="960px"
  >
    <v-container fluid>
      <v-btn
        icon="$arrow-left"
        class="mb-md-4"
        :size="xs ? 'small' : 'default'"
        :style="{zIndex: 1000}"
        @click="canBack ? router.back() : router.push('/')"
      ></v-btn>
      <v-row>
        <v-col
          cols="12"
          md="3"
        >
          <v-img
            :height="xs ? '148px' : '160px'"
            aspect-ratio="1/1"
            :src="parseUrlOrCid(props.release.thumbnailCID)"
          ></v-img>
        </v-col>

        <v-col
          cols="12"
          md="9"
          class="text-center text-md-start"
        >
          <p class="text-h5 text-md-h4 font-weight-medium">{{ props.release.name }}</p>
          <p v-if="metadata?.description">{{ metadata.description }}</p>
          <p v-if="artistName">
            <span v-if="metadata?.artistId">
              <a
                @click.prevent="router.push(`/artist/${metadata.artistId}`)"
                class="artist-link"
                style="cursor: pointer; color: rgb(var(--v-theme-primary)); text-decoration: none;"
              >{{ artistName }}</a>
            </span>
            <span v-else>{{ artistName }}</span>
          </p>
          <p>{{ albumFiles.length || metadata?.trackCount || 0 }} Songs<span v-if="totalDuration"> • {{ totalDuration }}</span></p>
          <p v-if="metadata?.releaseYear">{{ metadata.releaseYear }}</p>

          <!-- Quality and License Badges -->
          <div class="d-flex mt-2" style="gap: 3.5px">
            <QualityBadge
              v-if="metadata?.audioQuality"
              :quality="metadata.audioQuality"
            />
            <LicenseBadge
              v-if="metadata?.license"
              :license="metadata.license"
              linkable
            />
          </div>
        </v-col>
      </v-row>

      <!-- Show metadata fix option for users who can edit -->
      <v-alert
        v-if="canEditRelease && hasMetadataToSave"
        type="warning"
        class="mb-4"
        variant="outlined"
      >
        <div class="d-flex align-center justify-space-between">
          <div>
            <p class="text-subtitle-2 font-weight-bold mb-1">Track Metadata Needs Fixing</p>
            <p class="text-body-2 text-medium-emphasis">
              Will update: {{ pendingChanges.join(', ') }}
            </p>
          </div>
          <v-btn
            color="warning"
            variant="tonal"
            size="small"
            :loading="isFixingTracks"
            @click="fixAllTrackTitles"
          >
            Fix All
          </v-btn>
        </div>
        <v-progress-linear
          v-if="isFixingTracks"
          color="warning"
          indeterminate
          class="mt-2"
        ></v-progress-linear>
      </v-alert>

      <v-row>
        <v-list class="pb-10 w-100">
          <v-list-item
            v-for="(file, i) in albumFiles"
            :key="i"
            v-ripple="{class: 'text-primary-accent'}"
            :min-height="xs ? '48px' : '64px'"
            :class="i === 0 ? 'cursor-pointer border-t border-b' : 'cursor-pointer border-b'"
            :active="i === activeTrack?.index"
            color="primary-accent"
            @click="selectTrack(i)"
          >
            <template #prepend>
              <v-sheet :width="xs ? '24px' : '48px'">
                <p class="text-h5 text-md-h4 text-center">{{ i + 1 }}</p>
              </v-sheet>
            </template>
            <template #default>
              <div class="ml-2 my-1 d-flex align-center">
                <div
                  class="track-artwork-box"
                  :style="{
                    position: 'relative',
                    width: xs ? '48px' : '60px',
                    height: xs ? '48px' : '60px',
                    border: '1px solid rgba(var(--v-border-color), var(--v-border-opacity))',
                    borderRadius: '4px',
                    ...getTrackArtworkStyle(i)
                  }"
                >
                  <v-btn
                    location="center"
                    variant="tonal"
                    icon="$play"
                    density="comfortable"
                    readonly
                    :size="xs ? 'small' : 'default'"
                  ></v-btn>
                </div>
                <div class="ml-4">
                  <p class="text-subtitle-2 text-md-subtitle-1">{{ file.title }}</p>
                </div>
              </div>
              <!-- <v-divider class="mt-2"></v-divider> -->
            </template>
            <template #append>
              <p class="text-subtitle-2 text-medium-emphasis">{{ file.duration }}</p>

              <v-menu>
                <template #activator="{props: activatorProps}">
                  <v-btn
                    variant="text"
                    icon
                    class="ml-3"
                    v-bind="activatorProps"
                  >
                    <v-icon
                      size="25px"
                      icon="$dots-vertical"
                    />
                  </v-btn>
                </template>

                <v-list>
                  <v-list-item @click="setTrackToDownload(file)">
                    <template #title>
                      <v-icon icon="$download" />
                      Download track
                    </template>

                    <template #append> </template>
                  </v-list-item>
                </v-list>
              </v-menu>
            </template>
          </v-list-item>
        </v-list>
      </v-row>
    </v-container>

    <trackDownloaderDialog ref="trackDownloader" />
  </v-sheet>
</template>
<script setup lang="ts">
import {computed, onMounted, onUnmounted, ref, watch} from 'vue';
import {useRouter} from 'vue-router';
import {useDisplay} from 'vuetify';
import trackDownloaderDialog from './trackDownloader.vue';
import type {AudioTrack} from '/@/composables/audioAlbum';
import {useAudioAlbum} from '/@/composables/audioAlbum';
import {useFloatingVideo} from '/@/composables/floatingVideo';
import { parseUrlOrCid, isArchivistCid as checkIsArchivistCid } from '/@/utils';
import { getPrefetchedManifest } from '/@/composables/useArchivistPrefetch';
import type { ReleaseItem } from '/@/types';
import { useAccountStatusQuery, useEditReleaseMutation } from '/@/plugins/lensService/hooks';
import QualityBadge from '/@/components/badges/QualityBadge.vue';
import LicenseBadge from '/@/components/badges/LicenseBadge.vue';
// @ts-ignore
import jsmediatags from 'jsmediatags/dist/jsmediatags.min.js';

const props = defineProps<{
  release: ReleaseItem;
}>();

// Track metadata mismatch warnings and ID3 data
const trackWarnings = ref<Map<number, string>>(new Map());
const id3TrackData = ref<Map<number, { title: string; artist?: string }>>(new Map());
const isFixingTracks = ref(false);
const metadataJustSaved = ref(false);

// Reset saved flag when release changes
watch(() => props.release.id, () => {
  metadataJustSaved.value = false;
});

// Get list of pending changes to show user
const pendingChanges = computed(() => {
  // If we just saved, no pending changes
  if (metadataJustSaved.value) return [];

  const changes: string[] = [];

  let storedTracks: Array<{ title?: string; artist?: string; duration?: string }> = [];
  if (metadata.value?.trackMetadata) {
    try {
      storedTracks = typeof metadata.value.trackMetadata === 'string'
        ? JSON.parse(metadata.value.trackMetadata)
        : metadata.value.trackMetadata;
    } catch { /* ignore */ }
  }

  let newDurations = 0;
  let titleFixes = 0;
  let artistFixes = 0;

  for (let i = 0; i < albumFiles.value.length; i++) {
    const track = albumFiles.value[i];
    const stored = storedTracks[i];
    const id3 = id3TrackData.value.get(i);

    // Count new durations
    if (track.duration && (!stored || !stored.duration)) newDurations++;
    // Count title fixes
    if (id3?.title && (!stored || stored.title !== id3.title)) titleFixes++;
    // Count artist fixes
    if (id3?.artist && (!stored || stored.artist !== id3.artist)) artistFixes++;
  }

  // Add warnings as title fixes too
  titleFixes += trackWarnings.value.size;

  if (newDurations > 0) changes.push(`${newDurations} duration${newDurations > 1 ? 's' : ''}`);
  if (titleFixes > 0) changes.push(`${titleFixes} title${titleFixes > 1 ? 's' : ''}`);
  if (artistFixes > 0) changes.push(`${artistFixes} artist${artistFixes > 1 ? 's' : ''}`);

  return changes;
});

// Check if we have actual changes to save
const hasMetadataToSave = computed(() => pendingChanges.value.length > 0);

// Abort controller for cancelling in-flight requests
const abortController = ref<AbortController | null>(null);
const audioElements = ref<HTMLAudioElement[]>([]);

// Parse metadata if it's a string
const metadata = computed(() => {
  let meta = props.release.metadata;
  if (typeof meta === 'string') {
    meta = JSON.parse(meta);
  }
  // Parse nested license field if it's a string
  if (meta?.license && typeof meta.license === 'string') {
    try {
      meta = { ...meta, license: JSON.parse(meta.license) };
    } catch (e) {
      console.error('Failed to parse license:', e);
    }
  }
  return meta;
});

// Get artist name - check both 'artist' and 'author' fields for compatibility
const artistName = computed(() => {
  return metadata.value?.artist || metadata.value?.author || null;
});

// Per-track artwork support (generic for any album)
// Track artwork is stored in metadata.trackArtwork as a JSON array
// URLs are parsed through parseUrlOrCid to handle CIDs
const trackArtworkMap = computed(() => {
  if (!metadata.value?.trackArtwork) return null;

  try {
    const artwork = typeof metadata.value.trackArtwork === 'string'
      ? JSON.parse(metadata.value.trackArtwork)
      : metadata.value.trackArtwork;

    // Convert array to object indexed by track number, parsing URLs
    if (Array.isArray(artwork)) {
      const filtered = artwork.reduce((acc, url, index) => {
        if (url && url.trim() !== '') {
          acc[index] = parseUrlOrCid(url);
        }
        return acc;
      }, {} as Record<number, string>);
      return Object.keys(filtered).length > 0 ? filtered : null;
    }

    // If it's already an object, filter empty values and parse URLs
    if (typeof artwork === 'object') {
      const filtered = Object.entries(artwork).reduce((acc, [key, url]) => {
        if (url && typeof url === 'string' && url.trim() !== '') {
          acc[parseInt(key)] = parseUrlOrCid(url);
        }
        return acc;
      }, {} as Record<number, string>);
      return Object.keys(filtered).length > 0 ? filtered : null;
    }
  } catch (e) {
    console.error('Failed to parse trackArtwork:', e);
  }

  return null;
});

// Check if album has per-track artwork
const hasTrackArtwork = computed(() => trackArtworkMap.value !== null);

// Get current track artwork for full background
const currentTrackArtwork = computed(() => {
  if (!hasTrackArtwork.value || !activeTrack.value) return null;
  return trackArtworkMap.value?.[activeTrack.value.index] || null;
});

// Get track artwork style for thumbnail
function getTrackArtworkStyle(trackIndex: number) {
  if (!hasTrackArtwork.value) return {};

  const artwork = trackArtworkMap.value?.[trackIndex];
  if (!artwork) return {};

  return {
    backgroundImage: `url(${artwork})`,
    backgroundSize: 'cover',
    backgroundPosition: 'center',
  };
}

// Calculate total album duration
const totalDuration = computed(() => {
  let totalSeconds = 0;

  albumFiles.value.forEach(track => {
    if (track.duration) {
      // Handle both string "mm:ss" format and numeric seconds
      if (typeof track.duration === 'number') {
        totalSeconds += track.duration;
      } else if (typeof track.duration === 'string' && track.duration.includes(':')) {
        const [minutes, seconds] = track.duration.split(':').map(Number);
        if (!isNaN(minutes) && !isNaN(seconds)) {
          totalSeconds += minutes * 60 + seconds;
        }
      }
    }
  });

  if (totalSeconds === 0) return '';

  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    // Format as "1h 37m" for albums
    if (minutes > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${hours}h`;
    }
  } else if (totalSeconds >= 3600) {
    // Exactly 1 hour
    return '1h';
  } else {
    // Format as "48m 5s" for albums under an hour
    if (seconds > 0) {
      return `${minutes}m ${seconds}s`;
    } else {
      return `${minutes}m`;
    }
  }
});

const router = useRouter();
const canBack = computed(() => Boolean(window.history.state.back));
const {xs} = useDisplay();
const isLoading = ref(true);
const trackDownloader = ref();

// Check if user is admin or moderator
const { data: accountStatus } = useAccountStatusQuery();
const isAdmin = computed(() => accountStatus.value?.isAdmin || false);
const isModerator = computed(() => accountStatus.value?.hasRole?.('moderator') || false);
const canEditRelease = computed(() => {
  // User can edit if they are admin, moderator, or the original poster
  return isAdmin.value ||
         isModerator.value ||
         (accountStatus.value?.publicKey && props.release.postedBy?.toString() === accountStatus.value.publicKey);
});

const {albumFiles, handlePlay, activeTrack, albumQuality, currentAlbumId} = useAudioAlbum();
const {closeFloatingVideo} = useFloatingVideo();

// Edit release mutation
const editReleaseMutation = useEditReleaseMutation({
  onSuccess: () => {
    console.log('Track metadata updated successfully');
    trackWarnings.value.clear();
    id3TrackData.value.clear();
  },
  onError: (error) => {
    console.error('Failed to update track metadata:', error);
    if (error.message === 'Access denied') {
      alert('You do not have permission to edit this release. Only the original uploader, moderators, or admins can edit releases.');
    }
  }
});

// Pending play intent - for optimistic playback when CIDs aren't loaded yet
const pendingPlayIndex = ref<number | null>(null);

const selectTrack = (i: number) => {
  const track = albumFiles.value[i];

  // If track has a CID, play immediately
  if (track?.cid) {
    handlePlay(i);
    return;
  }

  // No CID yet - store intent, will auto-play when CIDs arrive
  pendingPlayIndex.value = i;
};

async function fetchIPFSFiles(cid: string): Promise<AudioTrack[]> {
  const url = parseUrlOrCid(cid);
  if (!url) {
    console.error(`Could not construct a valid IPFS URL for CID: ${cid}`);
    return [];
  }

  // Check if we have stored track metadata
  let storedTracks = null;
  if (metadata.value?.trackMetadata) {
    try {
      storedTracks = typeof metadata.value.trackMetadata === 'string'
        ? JSON.parse(metadata.value.trackMetadata)
        : metadata.value.trackMetadata;
    } catch (e) {
      console.warn('Failed to parse trackMetadata:', e);
    }
  }

  try {
    const ipfsFiles: AudioTrack[] = [];

    // Check if this is an Archivist CID - handle directory listing first
    const isArchivistCidVal = checkIsArchivistCid(cid);

    if (isArchivistCidVal) {
      // For Archivist CIDs, go directly to directory/file handling
      // Build the correct Archivist data URL (without /network/stream suffix)
      const archivistGateway = import.meta.env.VITE_ARCHIVIST_GATEWAY as string | undefined
        || import.meta.env.VITE_ARCHIVIST_API_URL as string | undefined
        || 'https://uploads.island.riff.cc';  // Fallback to known working gateway
      const baseUrl = archivistGateway.startsWith('http') ? archivistGateway : `https://${archivistGateway}`;
      const dataUrl = `${baseUrl}/api/archivist/v1/data/${cid}`;

      // CHECK PREFETCH CACHE FIRST - may have started on click before navigation
      const prefetchedPromise = getPrefetchedManifest(cid);
      let data: any = null;

      if (prefetchedPromise) {
        console.log('[albumViewer] Using prefetched Archivist manifest');
        try {
          data = await prefetchedPromise;
        } catch (e) {
          console.warn('[albumViewer] Prefetch failed, falling back to direct fetch');
          data = null;
        }
      }

      // If no prefetch or it failed, fetch normally
      if (!data) {
        console.log('[albumViewer] Fetching Archivist CID:', dataUrl);

        // First, check what type of content this is
        const headResponse = await fetch(dataUrl, { method: 'HEAD' });

        if (!headResponse.ok) {
          console.error('[albumViewer] HEAD request failed:', headResponse.status);
          throw new Error(`Failed to fetch CID: ${headResponse.status}`);
        }

        const contentType = headResponse.headers.get('content-type');
        console.log('[albumViewer] Content-Type:', contentType);

        // If it's an audio file, treat as single file
        if (contentType && (contentType.includes('audio/') || contentType.includes('application/octet-stream'))) {
          const contentLength = headResponse.headers.get('content-length');
          const fileName = props.release.name || 'Unknown Track';

          ipfsFiles.push({
            index: 0,
            album: props.release.name,
            cid: cid,
            title: storedTracks?.[0]?.title || fileName,
            artist: storedTracks?.[0]?.artist || artistName.value,
            duration: storedTracks?.[0]?.duration,
            size: contentLength ? `${(parseInt(contentLength) / 1024 / 1024).toFixed(2)} MB` : 'Unknown',
          });
          return ipfsFiles;
        }

        // Otherwise, fetch as directory (request JSON)
        const jsonResponse = await fetch(dataUrl, {
          headers: { 'Accept': 'application/json' }
        });

        if (!jsonResponse.ok) {
          console.error('[albumViewer] Failed to fetch directory:', jsonResponse.status, jsonResponse.statusText);
          throw new Error(`Failed to fetch directory: ${jsonResponse.status}`);
        }

        data = await jsonResponse.json();
      }

      // Type the data we received (either from prefetch or fresh fetch)
      const typedData = data as {
        cid?: string;
        name?: string;
        totalSize?: number;
        // Archivist format
        entries?: {
          name: string;
          cid: string;
          size: number;
          isDirectory: boolean;
          mimetype?: string;
        }[];
        // Legacy format
        files?: {
          title: string;
          cid: string;
          size: string;
        }[];
      };

      console.log('[albumViewer] Archivist response:', typedData);

      // Support both Archivist 'entries' format and legacy 'files' format
      const entries = typedData.entries || typedData.files?.map(f => ({
        name: f.title,
        cid: f.cid,
        size: typeof f.size === 'string' ? parseInt(f.size) || 0 : f.size,
        isDirectory: false,
      }));

      if (!entries || !Array.isArray(entries)) {
        console.error('[albumViewer] Invalid directory structure:', typedData);
        throw new Error(`Invalid directory structure received from ${dataUrl}`);
      }

      console.log('[albumViewer] Found entries:', entries.length);

      // Filter to audio files only and sort by name
      const audioExtensions = ['flac', 'mp3', 'ogg', 'opus', 'm4a', 'aac', 'wav'];
      const audioEntries = entries
        .filter(entry => {
          if (entry.isDirectory) return false;
          const ext = entry.name.split('.').pop()?.toLowerCase() || '';
          return audioExtensions.includes(ext);
        })
        .sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true }));

      console.log('[albumViewer] Audio entries after filter:', audioEntries.length);

      audioEntries.forEach((entry, index) => {
        const storedTrack = storedTracks?.[index];
        // Remove file extension for title
        const titleWithoutExt = entry.name.replace(/\.[^/.]+$/, '');

        ipfsFiles.push({
          index: index,
          album: props.release.name,
          cid: entry.cid,
          title: storedTrack?.title || titleWithoutExt,
          artist: storedTrack?.artist || artistName.value,
          duration: storedTrack?.duration,
          size: typeof entry.size === 'number'
            ? `${(entry.size / 1024 / 1024).toFixed(2)} MB`
            : String(entry.size),
        });
      });

      return ipfsFiles;
    }

    // For IPFS CIDs, use the original flow
    const response = await fetch(url, { method: 'HEAD' });
    if (!response.ok) {
      throw new Error(`Request failed on fetchIPFSFiles: ${response.status} ${response.statusText}. URL: ${url}`);
    }

    // Check if it's a single audio file by content-type
    const contentType = response.headers.get('content-type');
    if (contentType && (contentType.includes('audio/') || contentType.includes('application/octet-stream'))) {
      // It's a single audio file
      const contentLength = response.headers.get('content-length');
      const fileName = props.release.name || 'Unknown Track';

      ipfsFiles.push({
        index: 0,
        album: props.release.name,
        cid: cid,
        title: storedTracks?.[0]?.title || fileName,
        artist: storedTracks?.[0]?.artist || artistName.value,
        duration: storedTracks?.[0]?.duration,
        size: contentLength ? `${(parseInt(contentLength) / 1024 / 1024).toFixed(2)} MB` : 'Unknown',
      });
      return ipfsFiles;
    }

    // Fallback: parse HTML directory listing (legacy IPFS gateway format)
    const fullResponse = await fetch(url);
    const responseText = await fullResponse.text();

    const parser = new DOMParser();
    const doc = parser.parseFromString(responseText, 'text/html');

    const ipfsLinks = doc.querySelectorAll<HTMLAnchorElement>('a.ipfs-hash');
    const ipfsSizesData = doc.querySelectorAll<HTMLAnchorElement>(
      '[title="Cumulative size of IPFS DAG (data + metadata)"]',
    );

    ipfsLinks.forEach((link, key) => {
      const href = link.getAttribute('href');
      if (href) {
        const cidMatch = href.match(/\/ipfs\/([^?]+)/);
        const fileCid = cidMatch ? cidMatch[1] : null;

        const urlParams = new URLSearchParams(href.split('?')[1]);
        const encodedName = urlParams.get('filename');
        const fileName = encodedName ? decodeURIComponent(encodedName) : null;
        const fileSize = ipfsSizesData[key + 1]?.innerText || 'Unknown';

        if (fileCid && fileName) {
          const ext = fileName.split('.').pop()?.toLowerCase() || '';
          if (['flac', 'mp3', 'ogg', 'opus', 'm4a', 'aac', 'wav'].includes(ext)) {
            const storedTrack = storedTracks?.[ipfsFiles.length];
            ipfsFiles.push({
              index: key,
              album: props.release.name,
              cid: fileCid,
              title: storedTrack?.title || fileName.replace(/\.[^/.]+$/, ''),
              artist: storedTrack?.artist || artistName.value,
              duration: storedTrack?.duration,
              size: fileSize,
            });
          }
        }
      }
    });
    return ipfsFiles;
  } catch (error) {
    console.error(`Error fetching or processing IPFS data for CID ${cid} from ${url}:`, error);
    return [];
  }
};


function setTrackToDownload(track: AudioTrack) {
  trackDownloader.value.setTrack(track);
}

async function fixAllTrackTitles() {
  isFixingTracks.value = true;

  try {
    console.log('Starting fixAllTrackTitles for release:', props.release.id);
    console.log('Current metadata:', metadata.value);
    console.log('ID3 track data:', Array.from(id3TrackData.value.entries()));
    console.log('Account status in fixAllTrackTitles:', {
      isAdmin: isAdmin.value,
      isModerator: isModerator.value,
      accountStatus: accountStatus.value,
      canEditRelease: canEditRelease.value
    });

    // Build updated track metadata from ID3 data + cached durations
    const updatedTracks = albumFiles.value.map((track, index) => {
      const id3Data = id3TrackData.value.get(index);
      return {
        title: id3Data?.title || track.title,
        artist: id3Data?.artist || track.artist,
        // Cache duration so we don't need to load audio again
        ...(track.duration ? { duration: track.duration } : {}),
        // Include track number for proper ordering
        trackNumber: index + 1,
      };
    });

    console.log('Updated tracks:', updatedTracks);

    // Update the release metadata
    const updatedMetadata = {
      ...metadata.value,
      trackMetadata: JSON.stringify(updatedTracks)
    };

    console.log('Updated metadata to save:', updatedMetadata);

    // Save the updated release
    const mutationPayload = {
      id: props.release.id,
      postedBy: props.release.postedBy,
      siteAddress: props.release.siteAddress,
      name: props.release.name,
      categoryId: props.release.categoryId,
      contentCID: props.release.contentCID,
      thumbnailCID: props.release.thumbnailCID,
      metadata: updatedMetadata
    };

    console.log('Mutation payload:', mutationPayload);
    console.log('postedBy type:', typeof props.release.postedBy);
    console.log('postedBy value:', props.release.postedBy);

    const result = await editReleaseMutation.mutateAsync(mutationPayload);
    console.log('Mutation result:', result);

    // Success! Mark as saved so the alert disappears
    metadataJustSaved.value = true;
    trackWarnings.value.clear();
    id3TrackData.value.clear();
    console.log('Fix complete - metadata saved successfully');
  } catch (error) {
    console.error('Error in fixAllTrackTitles:', error);
  } finally {
    isFixingTracks.value = false;
  }
}

function formatTime(seconds: number): string {
  if (!seconds || isNaN(seconds)) return '';

  const totalMinutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.floor(seconds % 60);

  // If track is 60 minutes or longer, use hour format
  if (totalMinutes >= 60) {
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    if (minutes > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${hours}h`;
    }
  }

  // Otherwise use traditional minute:second format
  return `${totalMinutes}:${remainingSeconds.toString().padStart(2, '0')}`;
}

async function loadTrackMetadataAndVerify() {
  // Cancel any previous requests
  if (abortController.value) {
    abortController.value.abort();
  }

  // Clear previous audio elements
  audioElements.value.forEach(audio => {
    audio.src = '';
    audio.remove();
  });
  audioElements.value = [];

  // Create new abort controller
  abortController.value = new AbortController();
  const signal = abortController.value.signal;

  // Store the album ID we're loading metadata for
  const releaseId = props.release.id;

  // Load durations for all tracks, and ID3 tags only for admins
  const updatedTracks = await Promise.all(
    albumFiles.value.map(async (track, index) => {
      if (signal.aborted) return track;

      try {
        const url = parseUrlOrCid(track.cid);

        // Load audio metadata for duration if not already stored
        if (!track.duration) {
          const audio = new Audio();
          audioElements.value.push(audio); // Track for cleanup
          audio.crossOrigin = 'anonymous';
          audio.preload = 'metadata'; // Only load metadata, not full file
          audio.src = url;

          const trackWithDuration = await new Promise<AudioTrack>((resolve) => {
            let resolved = false;
            const cleanup = () => {
              audio.removeEventListener('loadedmetadata', onLoadedMetadata);
              audio.removeEventListener('error', onError);
            };

            const onLoadedMetadata = () => {
              if (resolved) return;
              resolved = true;
              cleanup();
              if (!signal.aborted) {
                console.log(`[albumViewer] Duration loaded for track ${index}: ${audio.duration}s`);
                resolve({
                  ...track,
                  duration: formatTime(audio.duration)
                });
              } else {
                resolve(track);
              }
            };

            const onError = (e: Event) => {
              if (resolved) return;
              resolved = true;
              cleanup();
              console.warn(`[albumViewer] Failed to load duration for track ${index}:`, e);
              resolve(track); // Return track without duration on error
            };

            audio.addEventListener('loadedmetadata', onLoadedMetadata);
            audio.addEventListener('error', onError);

            // Check if aborted
            signal.addEventListener('abort', () => {
              if (resolved) return;
              resolved = true;
              cleanup();
              audio.src = '';
              resolve(track);
            });

            // Timeout after 30 seconds (increased for slower connections)
            setTimeout(() => {
              if (!resolved && !signal.aborted) {
                resolved = true;
                cleanup();
                console.warn(`[albumViewer] Timeout loading duration for track ${index}`);
                audio.src = ''; // Cancel loading
                resolve(track);
              }
            }, 30000);
          });

          track = trackWithDuration;
        }

        // Only check ID3 tags if user can edit the release
        if (canEditRelease.value) {
          console.log('[albumViewer] Reading ID3 tags from:', url);
          return new Promise<AudioTrack>((resolve) => {
            jsmediatags.read(url, {
              onSuccess: (tag) => {
                const tags = tag.tags;
                console.log('[albumViewer] ID3 tags for track:', track.title, tags);

                // Store ID3 data
                if (tags.title || tags.artist) {
                  id3TrackData.value.set(index, {
                    title: tags.title,
                    artist: tags.artist
                  });
                }

                // Check for mismatches
                if (tags.title && tags.title !== track.title) {
                  trackWarnings.value.set(index, `ID3 title "${tags.title}" doesn't match stored title "${track.title}"`);
                }

                resolve(track); // Keep stored data, just log warnings
              },
              onError: (error) => {
                console.error(`[albumViewer] Failed to read ID3 tags for ${track.title} from ${url}:`, error);
                resolve(track);
              }
            });
          });
        }

        return track;
      } catch (error) {
        console.warn(`Error loading metadata for track: ${track.title}`, error);
        return track;
      }
    })
  );

  // Only update if we're still viewing the same album
  if (currentAlbumId.value === releaseId) {
    albumFiles.value = updatedTracks;
  }
}

onMounted(async () => {
  closeFloatingVideo();

  // Show content immediately - tracks load in background
  isLoading.value = false;

  // Set album quality from metadata
  if (metadata.value?.audioQuality) {
    albumQuality.value = metadata.value.audioQuality;
  } else {
    albumQuality.value = null;
  }

  // Track which album we're loading to prevent state pollution
  const releaseId = props.release.id;
  currentAlbumId.value = releaseId;

  // Load the album's tracks for display, but DON'T touch activeTrack
  // The player is a global singleton - playback persists through navigation
  if (!activeTrack.value || activeTrack.value.album !== props.release.name) {
    // INSTANT RENDER: If we have pre-parsed tracks from cache, show them immediately
    const parsedTracks = metadata.value?._parsedTracks;
    if (parsedTracks && Array.isArray(parsedTracks)) {
      albumFiles.value = parsedTracks.map((track, index) => ({
        index,
        album: props.release.name,
        cid: '', // CID not known yet - will be populated by IPFS fetch
        title: track.title || `Track ${index + 1}`,
        artist: track.artist || metadata.value?.author || '',
        duration: track.duration,
      }));
    } else {
      albumFiles.value = [];
    }

    // NOTE: We intentionally do NOT clear activeTrack here
    // Playback continues even when browsing other albums

    // Fetch CIDs in background (needed for playback)
    const ipfsFiles = await fetchIPFSFiles(props.release.contentCID);

    // Only update albumFiles if we're still viewing the same album
    // This prevents race conditions when quickly switching between albums
    if (currentAlbumId.value === releaseId) {
      albumFiles.value = ipfsFiles;

      // OPTIMISTIC PLAYBACK: If user clicked a track while CIDs were loading, play it now
      if (pendingPlayIndex.value !== null) {
        const pendingIndex = pendingPlayIndex.value;
        pendingPlayIndex.value = null;
        handlePlay(pendingIndex);
      }

      loadTrackMetadataAndVerify(); // Load metadata and verify in background
    }
  }
});

onUnmounted(() => {
  // Cancel any in-flight requests
  if (abortController.value) {
    abortController.value.abort();
  }

  // Clean up audio elements
  audioElements.value.forEach(audio => {
    audio.src = '';
    audio.remove();
  });
  audioElements.value = [];
});
</script>

<style scoped>
.track-artwork-background {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: calc(100% - var(--v-layout-footer-height, 0px));
  background-size: cover;
  background-position: center top;
  background-repeat: no-repeat;
  opacity: 0.9;
  z-index: 0;
}

.artwork-fade-enter-active,
.artwork-fade-leave-active {
  transition: opacity 0.25s ease-in-out;
}

.artwork-fade-enter-from {
  opacity: 0;
}

.artwork-fade-leave-to {
  opacity: 0;
}

.album-content {
  position: relative;
  z-index: 1;
  backdrop-filter: blur(2px);
  background-color: rgba(var(--v-theme-surface), 0.85) !important;
}
</style>

<style>
/* Make main transparent when background artwork is showing */
main:has(.track-artwork-background) {
  background-color: transparent !important;
}
</style>
