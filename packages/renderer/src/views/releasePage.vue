<template>
  <v-container>
    <v-sheet
      min-height="75vh"
    >
      <video-player
        :video-source="`https://${IPFS_GATEWAY}/ipfs/${props.contentCID}`"
      />
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';

const props = defineProps({
  contentCID: {
    required: true,
    type: String,
  },
  category: {
    required: true,
    type: String,
  },
});
const IPFS_GATEWAY = 'cdn.riff.cc';

onMounted(() => {
  window.scrollTo(0,0);
})

</script>

