<template>
  <v-container>
    <v-sheet
      min-height="75vh"
    >
      <video
        v-if="props.category === 'video'"
        controls
        width="100%"
        height="100%"
      >
        <source
          :src="`https://${IPFS_GATEWAY}/ipfs/${props.contentCID}`"
          type="video/mp4"
        />
        Your browser does not support the video tag.
      </video>
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">

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

</script>

