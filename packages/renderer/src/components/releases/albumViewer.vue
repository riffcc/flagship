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
          <p>{{ props.release.metadata?.['description'] }}</p>
          <p>{{ albumFiles.length }} Songs</p>
          <p>{{ props.release.metadata?.['author'] }} - {{ props.release.metadata?.['releaseYear'] }}</p>
        </v-col>
      </v-row>
      <v-row>
        <v-list class="pb-10 w-100">
          <v-list-item
            v-for="(file, i) in albumFiles"
            :key="i"
            v-ripple="{class: 'text-primary-accent'}"
            :min-height="xs ? '48px' : '64px'"
            :class="i === 0 ? 'cursor-pointer border-t border-b' : 'cursor-pointer border-b'"
            :active="i === activeTrack?.index"
            active-color="primary-accent"
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
                  <p class="text-caption text-md-subtitle-2 text-medium-emphasis">
                    {{ props.release.metadata?.['author'] }}
                  </p>
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
import type { AnyObject } from '@riffcc/lens-sdk';

const props = defineProps<{
  release: ReleaseItem<AnyObject>
}>();

const router = useRouter();
const canBack = computed(() => Boolean(window.history.state.back));
const {xs} = useDisplay();
const isLoading = ref(true);
const trackDownloader = ref();

const {albumFiles, handlePlay, activeTrack} = useAudioAlbum();
const {closeFloatingVideo} = useFloatingVideo();

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
  try {
    const ipfsFiles: AudioTrack[] = [];
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Request failed on fetchIPFSFiles: ${response.status} ${response.statusText}. URL: ${url}`);
    }

    if (cid.startsWith('zD')) {
      const data = await response.json() as {
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
            ipfsFiles.push({
                index: index,
                album: props.release.name,
                cid: file.cid,
                title: file.title.split('.')[0],
                size: file.size,
            });
        } else {
            console.warn('Skipping invalid file entry in JSON response:', file);
        }
      });
      return ipfsFiles;

    }
    else {
      const htmlText = await response.text();
      const parser = new DOMParser();
      const doc = parser.parseFromString(htmlText, 'text/html');

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
              ipfsFiles.push({
                index: key,
                album: props.release.name,
                cid,
                title: fileName.split('.')[0],
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

onMounted(async () => {
  closeFloatingVideo();
  // Only load the audio tracks if they are not currently playing or if the active track's album is different from the browsed album.
  if (!activeTrack.value || (activeTrack.value && activeTrack.value.album !== props.release.name)) {
    albumFiles.value = [];
    activeTrack.value = undefined;
    const ipfsFiles = await fetchIPFSFiles(props.release.contentCID);
    albumFiles.value = ipfsFiles;
  }
  isLoading.value = false;

  // let _albumFiles: albumFile[] = [];
  // ipfsFiles.forEach((ipfsFile) => {
  //   const audio = new Audio();
  //   audio.crossOrigin = 'anonymous';
  //   audio.src = `https://${IPFS_GATEWAY}/ipfs/${ipfsFile.cid}`;
  //   audio.addEventListener('loadedmetadata', () => {
  //     _albumFiles.push({ name: ipfsFile.name, cid: ipfsFile.cid, duration: formatTime(audio.duration)});
  //   });
  // });
});
</script>
