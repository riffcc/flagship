<template>
  <v-app>
    <app-bar />
    <v-main min-height="100vh">
      <router-view />
    </v-main>
    <app-footer />
  </v-app>
</template>

<script setup lang="ts">
import appBar from '/@/components/layout/appBar.vue';
import appFooter from '/@/components/layout/appFooter.vue';
// import {suivre as follow} from '@constl/vue';
// import {useOrbiter} from '/@/plugins/orbiter/utils';

// const {orbiter} = useOrbiter();
// const orbiterReady = follow(({f}) => orbiter.listenForSiteConfigured({f}));
</script>
