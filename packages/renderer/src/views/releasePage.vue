<template>
  <v-container
    fluid
    class="pa-0"
  >
    <video-player
      v-if="route.query.category === 'video'"
      :content-cid="route.query.contentCID as string"
    />
    <album-viewer
      v-if="route.query.category === 'audio'"
      :content-cid="route.query.contentCID as string"
      :title="route.query.title as string"
      :thumbnail="route.query.thumbnail as string"
      :author="route.query.author as string"
      :description="route.query.description as string"
      :release-year="route.query.releaseYear as string"
    ></album-viewer>
  </v-container>
</template>

<script setup lang="ts">
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import albumViewer from '/@/components/releases/albumViewer.vue';
import {useRoute} from 'vue-router';

const route = useRoute();
</script>
