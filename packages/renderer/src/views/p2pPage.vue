<template>
  <v-container fluid class="fill-height d-flex align-center justify-center pa-4">
    <NetworkMapGraph @close="handleClose" />
  </v-container>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import NetworkMapGraph from '/@/components/misc/networkMapGraph.vue';

const router = useRouter();

const handleClose = () => {
  router.push('/');
};
</script>
