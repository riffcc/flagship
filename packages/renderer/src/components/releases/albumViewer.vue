<template>
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
    class="text-body-2 mx-auto my-4"
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
          <p v-if="metadata?.author">{{ metadata.author }}</p>
          <p>{{ albumFiles.length }} Songs<span v-if="totalDuration"> • {{ totalDuration }}</span></p>
          <p v-if="metadata?.releaseYear">{{ metadata.releaseYear }}</p>
        </v-col>
      </v-row>
      
      <!-- Show ID3 tag warnings for users who can edit -->
      <v-alert
        v-if="canEditRelease && trackWarnings.size > 0"
        type="warning"
        class="mb-4"
        variant="outlined"
      >
        <div class="d-flex align-center justify-space-between mb-2">
          <p class="text-subtitle-2 font-weight-bold">ID3 Tag Mismatches Detected:</p>
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
          class="mb-2"
        ></v-progress-linear>
        <ul class="ml-4">
          <li v-for="[index, warning] in trackWarnings" :key="index">
            Track {{ index + 1 }}: {{ warning }}
          </li>
        </ul>
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
            @click="async () => await selectTrack(i)"
          >
            <template #prepend>
              <v-sheet :width="xs ? '24px' : '48px'">
                <p class="text-h5 text-md-h4 text-center">{{ i + 1 }}</p>
              </v-sheet>
            </template>
            <template #default>
              <div class="ml-2 my-1 d-flex align-center">
                <v-sheet
                  position="relative"
                  :width="xs ? '48px' : '60px'"
                  :height="xs ? '48px' : '60px'"
                  border
                >
                  <v-btn
                    location="center"
                    variant="tonal"
                    icon="$play"
                    density="comfortable"
                    readonly
                    :size="xs ? 'small' : 'default'"
                  ></v-btn>
                </v-sheet>
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
import {computed, onMounted, ref} from 'vue';
import {useRouter} from 'vue-router';
import {useDisplay} from 'vuetify';
import trackDownloaderDialog from './trackDownloader.vue';
import type {AudioTrack} from '/@/composables/audioAlbum';
import {useAudioAlbum} from '/@/composables/audioAlbum';
import {useFloatingVideo} from '/@/composables/floatingVideo';
import { parseUrlOrCid } from '/@/utils';
import type { ReleaseItem } from '/@/types';
import { useAccountStatusQuery, useEditReleaseMutation } from '/@/plugins/lensService/hooks';
// @ts-ignore
import jsmediatags from 'jsmediatags/dist/jsmediatags.min.js';

const props = defineProps<{
  release: ReleaseItem
}>();

// Track metadata mismatch warnings and ID3 data
const trackWarnings = ref<Map<number, string>>(new Map());
const id3TrackData = ref<Map<number, { title: string; artist?: string }>>(new Map());
const isFixingTracks = ref(false);

// Parse metadata if it's a string
const metadata = computed(() => {
  if (typeof props.release.metadata === 'string') {
    return JSON.parse(props.release.metadata);
  }
  return props.release.metadata;
});

// Calculate total album duration
const totalDuration = computed(() => {
  let totalSeconds = 0;
  
  albumFiles.value.forEach(track => {
    if (track.duration) {
      const [minutes, seconds] = track.duration.split(':').map(Number);
      if (!isNaN(minutes) && !isNaN(seconds)) {
        totalSeconds += minutes * 60 + seconds;
      }
    }
  });
  
  if (totalSeconds === 0) return '';
  
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  
  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  } else {
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
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

const {albumFiles, handlePlay, activeTrack} = useAudioAlbum();
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

const selectTrack = async (i: number) => {
  await new Promise(r => setTimeout(r, 200));
  handlePlay(i);
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
        artist: storedTracks?.[0]?.artist || metadata.value?.author,
        duration: storedTracks?.[0]?.duration,
        size: contentLength ? `${(parseInt(contentLength) / 1024 / 1024).toFixed(2)} MB` : 'Unknown',
      });
      return ipfsFiles;
    }

    // Otherwise, try to parse as directory
    const fullResponse = await fetch(url);
    const responseText = await fullResponse.text();

    if (cid.startsWith('zD')) {
      const data = JSON.parse(responseText) as {
        files: {
          title: string;
          cid: string;
          size: string;
        }[];
      };

      if (!data || !Array.isArray(data.files)) {
          throw new Error(`Invalid JSON structure received from ${url}`);
      }

      data.files.forEach((file, index) => {
        if (file.cid && file.title && file.size) {
            const storedTrack = storedTracks?.[index];
            ipfsFiles.push({
                index: index,
                album: props.release.name,
                cid: file.cid,
                title: storedTrack?.title || file.title.split('.')[0],
                artist: storedTrack?.artist || metadata.value?.author,
                duration: storedTrack?.duration,
                size: file.size,
            });
        } else {
            console.warn('Skipping invalid file entry in JSON response:', file);
        }
      });
      return ipfsFiles;

    }
    else {
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
          const cid = cidMatch ? cidMatch[1] : null;

          const urlParams = new URLSearchParams(href.split('?')[1]);
          const encodedName = urlParams.get('filename');
          const fileName = encodedName ? decodeURIComponent(encodedName) : null;
          const fileSize = ipfsSizesData[key + 1].innerText;

          if (cid && fileName) {
            if (['flac', 'mp3', 'ogg'].includes(fileName.split('.')[1])) {
              const storedTrack = storedTracks?.[ipfsFiles.length];
              ipfsFiles.push({
                index: key,
                album: props.release.name,
                cid,
                title: storedTrack?.title || fileName.split('.')[0],
                artist: storedTrack?.artist || metadata.value?.author,
                duration: storedTrack?.duration,
                size: fileSize,
              });
            }
          }
        }
      });
      return ipfsFiles;
    }
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
    
    // Build updated track metadata from ID3 data
    const updatedTracks = albumFiles.value.map((track, index) => {
      const id3Data = id3TrackData.value.get(index);
      if (id3Data?.title) {
        return {
          title: id3Data.title,
          artist: id3Data.artist || track.artist,
          duration: track.duration
        };
      }
      return {
        title: track.title,
        artist: track.artist,
        duration: track.duration
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
  } catch (error) {
    console.error('Error in fixAllTrackTitles:', error);
  } finally {
    isFixingTracks.value = false;
  }
}

function formatTime(seconds: number): string {
  if (!seconds || isNaN(seconds)) return '';
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.floor(seconds % 60);
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
}

async function loadTrackMetadataAndVerify() {
  // Load durations for all tracks, and ID3 tags only for admins
  const updatedTracks = await Promise.all(
    albumFiles.value.map(async (track, index) => {
      try {
        const url = parseUrlOrCid(track.cid);
        
        // Load audio metadata for duration if not already stored
        if (!track.duration) {
          const audio = new Audio();
          audio.crossOrigin = 'anonymous';
          audio.src = url;
          
          const trackWithDuration = await new Promise<AudioTrack>((resolve) => {
            audio.addEventListener('loadedmetadata', () => {
              resolve({
                ...track,
                duration: formatTime(audio.duration)
              });
            });
            
            audio.addEventListener('error', () => {
              console.warn(`Failed to load duration for track: ${track.title}`);
              resolve(track); // Return track without duration on error
            });
            
            // Timeout after 10 seconds
            setTimeout(() => {
              audio.src = ''; // Cancel loading
              resolve(track);
            }, 10000);
          });
          
          track = trackWithDuration;
        }
        
        // Only check ID3 tags if user can edit the release
        if (canEditRelease.value) {
          return new Promise<AudioTrack>((resolve) => {
            jsmediatags.read(url, {
              onSuccess: (tag) => {
                const tags = tag.tags;
                console.log('ID3 tags for track:', track.title, tags);
                
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
                console.warn(`Failed to read ID3 tags for ${track.title}:`, error);
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
  
  albumFiles.value = updatedTracks;
}

onMounted(async () => {
  closeFloatingVideo();
  // Only load the audio tracks if they are not currently playing or if the active track's album is different from the browsed album.
  if (!activeTrack.value || (activeTrack.value && activeTrack.value.album !== props.release.name)) {
    albumFiles.value = [];
    activeTrack.value = undefined;
    const ipfsFiles = await fetchIPFSFiles(props.release.contentCID);
    albumFiles.value = ipfsFiles;
    
    // Load track metadata after initial files are loaded
    isLoading.value = false; // Show content immediately
    loadTrackMetadataAndVerify(); // Load metadata and verify in background
  } else {
    isLoading.value = false;
  }
});
</script>
