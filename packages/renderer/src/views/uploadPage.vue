<template>
  <v-container>
    <v-sheet
      width="480px"
      class="px-8 pb-16 pt-10 mx-auto"
    >
      <new-release-form v-if="canUpload" />
      <v-alert
        v-else-if="canUpload === false"
        type="info"
        class="mt-4"
        color="black"
        text-color="white"
      >
        You aren't currently authorised to add releases to this instance of Riff.CC.
      </v-alert>
      <div
        v-else
      >
        <v-alert
          type="info"
          class="mt-4"
          color="black"
          text-color="white"
        >
          Loading authorisation data...
        </v-alert>
        <v-skeleton-loader
          type="list-item"
        />
      </div>
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import {suivre as follow} from '@constl/vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';

import NewReleaseForm from '/@/components/releases/newReleaseForm.vue';

const {orbiter} = useOrbiter();
const canUpload = follow(orbiter.followCanUpload.bind(orbiter));
</script>
