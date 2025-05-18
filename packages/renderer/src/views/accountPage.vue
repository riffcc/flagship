<template>
  <v-container>
    <v-sheet
      class="d-flex position-relative py-4 px-2 px-md-12"
      max-height="160"
    >
      <v-img
        :src="userData?.avatar"
        max-width="120"
      />
      <p class="ml-4 my-auto">
        Anonymous
      </p>
      <v-switch
        v-model="staticModeSwitch"
        class="position-absolute right-0 top-0 mr-4 mr-md-12 mt-md-2"
        label="Static mode"
        :color="staticModeSwitch ? 'primary' : 'secondary'"
      />
    </v-sheet>

    <v-card class="mt-4 text-center">
      <v-card-title>
        <h3>
          Account info
        </h3>
      </v-card-title>
      <v-divider></v-divider>
      <v-list lines="two">
        <v-list-item
          title="Public Key"
          :subtitle="publicKey"
        >
        </v-list-item>
        <v-list-item
          title="Peer ID"
          :subtitle="peerId"
        >
        </v-list-item>
        <v-list-item
          title="Account status"
          :subtitle="`GUEST (View-only access to the site)`"
        >
        </v-list-item>
      </v-list>
    </v-card>
    <v-card class="mt-4 text-center">
      <v-card-title>
        <h3>
          Connectivity
        </h3>
      </v-card-title>
      <v-divider></v-divider>
      <v-list lines="two">
        <v-list-item>
          <p>Not implemented ⚠️</p>
        </v-list-item>
      </v-list>
    </v-card>
  </v-container>
</template>
<script setup lang="ts">
import { onMounted, ref, watchEffect } from 'vue';

import { useStaticStatus } from '/@/composables/staticStatus';
import { useUserSession } from '/@/composables/userSession';
import { useLensService } from '/@/plugins/lensService/utils';

const { userData } = useUserSession();
const { staticStatus } = useStaticStatus();

const { lensService } = useLensService();

const staticModeSwitch = ref(staticStatus.value === 'static');
watchEffect(() => {
  staticStatus.value = staticModeSwitch.value ? 'static' : 'live';
});
watchEffect(() => {
  staticModeSwitch.value = staticStatus.value === 'static';
});

const publicKey = ref<string | undefined>();
const peerId = ref<string | undefined>();

onMounted(async () => {
  publicKey.value = await lensService.getPublicKey();
  peerId.value = await lensService.getPeerId();
});

</script>
