<template>
  <v-card>
    <v-card-text>
      <video
        ref="video"
        :src="videoURL"
        controls
        :autoplay="false"
      ></video>
      <div ref="status"></div>
    </v-card-text>
  </v-card>
</template>
<script setup lang="ts">
import {onMounted, ref} from 'vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';
// import Hls from 'hls.js';
// import HlsjsIpfsLoader from 'hlsjs-ipfs-loader';

const props = defineProps<{file: string}>();

const {orbiter} = useOrbiter();

const videoURL = ref();

const video = ref<HTMLMediaElement | null>(document.getElementById('video') as HTMLMediaElement);
const status = ref<HTMLElement | null>(document.getElementById('status'));

// Inspired by: https://discuss.ipfs.tech/t/using-ipfs-to-stream-audio-video-in-the-browser/4908
// ...and especially https://github.com/ipfs-examples/js-ipfs-examples/blob/master/examples/browser-video-streaming/src/index.js

onMounted(async () => {
  const videoData = await orbiter.constellation.obtFichierSFIP({
    id: props.file,
  });
  console.log(videoData);
  videoURL.value = URL.createObjectURL(new Blob([videoData!.buffer]));
  console.log(videoURL);
  return;
  /*
  if (Hls.isSupported()) {
    const hls = new Hls();
    hls.loadSource(videoURL.value);
    hls.attachMedia(video.value!);
    hls.on(Hls.Events.MANIFEST_PARSED, function () {
      const node = document.createTextNode('Video ready...');
      status.value?.appendChild(node);

      video.value?.play();
    });
  }
  return;
  Hls.DefaultConfig.loader = HlsjsIpfsLoader;
  Hls.DefaultConfig.debug = true;
  if (Hls.isSupported()) {
    const hls = new Hls();
    // @ts-expect-error ipfs extention config for hls is not included in hls types
    hls.config.ipfs = orbiter.constellation.sfip;
    // @ts-expect-error ipfs extention config for hls is not included in hls types
    hls.config.ipfsHash = props.file.cid;

    hls.loadSource('');

    if (!video.value)
      throw new Error(
        "Video element not found. You really shouldn't ever be encountering this error message. If somehow you do, please complain.",
      );

    hls.attachMedia(video.value);
    hls.on(Hls.Events.MANIFEST_PARSED, () => {
      const node = document.createTextNode('Video ready...');
      status.value?.appendChild(node);

      video.value?.play();
    });
    
  }*/
});
</script>
