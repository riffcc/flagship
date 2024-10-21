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
        icon="mdi-arrow-left"
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
            :src="
              cid(props.thumbnail ?? '')
                ? `https://${IPFS_GATEWAY}/ipfs/${props.thumbnail}`
                : props.thumbnail
            "
          ></v-img>
        </v-col>

        <v-col
          cols="12"
          md="9"
          class="text-center text-md-start"
        >
          <p class="text-h5 text-md-h4 font-weight-medium">{{ props.title }}</p>
          <p>{{ props.description }}</p>
          <p>{{ albumFiles.length }} Songs</p>
          <p>{{ props.author }} - {{ props.releaseYear }}</p>
        </v-col>
      </v-row>
      <v-row>
        <v-list class="pb-10 w-100">
          <v-list-item
            v-for="(file, i) in albumFiles"
            :key="i"
            :min-height="xs ? '48px' : '64px'"
            class="my-1"
          >
            <template #prepend>
              <v-sheet :width="xs ? '24px' : '48px'">
                <p class="text-h5 text-md-h4 text-center">{{ i + 1 }}</p>
              </v-sheet>
            </template>
            <template #default>
              <v-sheet class="ml-2 d-flex align-center">
                <v-sheet
                  position="relative"
                  :width="xs ? '48px' : '60px'"
                  :height="xs ? '48px' : '60px'"
                  border
                >
                  <!-- <v-img
                    cover
                    inline
                    width="60px"
                    aspect-ratio="1"
                    src="https://cdn.vuetifyjs.com/images/cards/sunshine.jpg"
                  ></v-img> -->
                  <v-btn
                    location="center"
                    variant="tonal"
                    icon="mdi-play"
                    density="comfortable"
                    :size="xs ? 'small' : 'default'"
                    @click="() => handlePlay(i)"
                  ></v-btn>
                </v-sheet>
                <div class="ml-4">
                  <p class="text-subtitle-2 text-md-subtitle-1">{{ file.name }}</p>
                  <p class="text-caption text-md-subtitle-2 text-medium-emphasis">
                    {{ props.author }}
                  </p>
                </div>
              </v-sheet>
              <v-divider class="mt-2"></v-divider>
            </template>
            <template #append>
              <p class="text-subtitle-2 text-medium-emphasis">{{ file.duration }}</p>
            </template>
          </v-list-item>
        </v-list>
      </v-row>
    </v-container>
    <audio-player
      v-if="selectedAudio"
      :selected-audio="selectedAudio"
      :on-close-callback="onCloseCallback"
      :handle-next="handleNext"
      :handle-previous="handlePrevious"
      :repeat="repeat"
      :toggle-repeat="toggleRepeat"
      :shuffle="shuffle"
      :toggle-shuffle="toggleShuffle"
    />
  </v-sheet>
</template>
<script setup lang="ts">
import {computed, onMounted, ref} from 'vue';
import audioPlayer from '/@/components/releases/audioPlayer.vue';
import {IPFS_GATEWAY} from '/@/constants/ipfs';
import {useRouter} from 'vue-router';
// import { formatTime } from '/@/utils';
import {cid} from 'is-ipfs';
import {useDisplay} from 'vuetify';

type Props = {
  contentCid: string;
  title: string;
  thumbnail?: string;
  author?: string;
  description?: string;
  releaseYear?: number | string;
};

interface IPFSFile {
  cid: string;
  name: string;
}

interface albumFile extends IPFSFile {
  duration?: string;
}
const props = defineProps<Props>();
const albumFiles = ref<albumFile[]>([]);

const router = useRouter();
const canBack = computed(() => Boolean(window.history.state.back));
const {xs} = useDisplay();

const isLoading = ref(true);
const selectedAudio = ref<{index: number; cid: string; name: string}>();
const repeat = ref(false);
const shuffle = ref(false);
const toggleRepeat = () => (repeat.value ? (repeat.value = false) : (repeat.value = true));
const toggleShuffle = () => (shuffle.value ? (shuffle.value = false) : (shuffle.value = true));

const handleOnSelectAndPlay = (index: number) => {
  selectedAudio.value = {
    index,
    name: albumFiles.value[index].name,
    cid: albumFiles.value[index].cid,
  };
  if ('mediaSession' in window.navigator) {
    window.navigator.mediaSession.metadata = new MediaMetadata({
      title: selectedAudio.value.name,
      album: props.title,
      artist: props.author,
    });
  }
};
const handlePlay = (index: number) => {
  handleOnSelectAndPlay(index);
};

const handlePrevious = () => {
  if (selectedAudio.value && selectedAudio.value.index > 0) {
    handlePlay(selectedAudio.value.index - 1);
  }
};

const handleNext = () => {
  if (selectedAudio.value) {
    if (shuffle.value) {
      const randomIndex = Math.floor(Math.random() * albumFiles.value.length);
      handlePlay(randomIndex !== selectedAudio.value.index ? randomIndex : randomIndex + 1);
    } else {
      if (selectedAudio.value.index !== albumFiles.value.length - 1) {
        handlePlay(selectedAudio.value.index + 1);
      } else {
        if (repeat.value) {
          handlePlay(0);
        } else {
          selectedAudio.value = undefined;
        }
      }
    }
  }
};

const onCloseCallback = () => {
  if (selectedAudio.value) {
    selectedAudio.value = undefined;
  }
};

async function extractIPFSFilesFromFolder(url: string): Promise<IPFSFile[]> {
  try {
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Request error: ${response.statusText}`);
    }

    const htmlText = await response.text();
    const parser = new DOMParser();
    const doc = parser.parseFromString(htmlText, 'text/html');

    const ipfsLinks = doc.querySelectorAll<HTMLAnchorElement>('a.ipfs-hash');

    const ipfsFiles: IPFSFile[] = [];

    ipfsLinks.forEach(link => {
      const href = link.getAttribute('href');
      if (href) {
        const cidMatch = href.match(/\/ipfs\/([^?]+)/);
        const cid = cidMatch ? cidMatch[1] : null;

        const urlParams = new URLSearchParams(href.split('?')[1]);
        const encodedName = urlParams.get('filename');
        const fileName = encodedName ? decodeURIComponent(encodedName) : null;

        if (cid && fileName) {
          if (['flac', 'mp3', 'ogg'].includes(fileName.split('.')[1])) {
            ipfsFiles.push({cid, name: fileName.split('.')[0]});
          }
        }
      }
    });

    return ipfsFiles;
  } catch (error) {
    console.error('An error has occurred:', error);
    return [];
  }
}

onMounted(async () => {
  const ipfsFiles = await extractIPFSFilesFromFolder(
    `https://${IPFS_GATEWAY}/ipfs/${props.contentCid}`,
  );
  // let _albumFiles: albumFile[] = [];
  // ipfsFiles.forEach((ipfsFile) => {
  //   const audio = new Audio();
  //   audio.crossOrigin = 'anonymous';
  //   audio.src = `https://${IPFS_GATEWAY}/ipfs/${ipfsFile.cid}`;
  //   audio.addEventListener('loadedmetadata', () => {
  //     _albumFiles.push({ name: ipfsFile.name, cid: ipfsFile.cid, duration: formatTime(audio.duration)});
  //   });
  // });
  albumFiles.value = ipfsFiles;
  isLoading.value = false;
});
</script>
