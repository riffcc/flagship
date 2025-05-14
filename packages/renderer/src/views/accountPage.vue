<template>
  <v-container>
    <v-sheet
      class="d-flex position-relative py-4 px-2 px-md-12"
      max-height="160"
    >
      <v-img
        :src="userAvatar"
        max-width="120"
      />
      <p class="ml-4 my-auto">
        {{ displayName }}
      </p>
      <v-switch
        v-model="staticModeSwitch"
        class="position-absolute right-0 top-0 mr-4 mr-md-12 mt-md-2"
        label="Static mode"
        :color="staticModeSwitch ? 'primary' : 'secondary'"
      />
    </v-sheet>

    <v-card
      class="mt-4 text-center"
    >
      <v-card-title>
        <h3>
          Account info
        </h3>
      </v-card-title>
      <v-divider></v-divider>
      <v-list lines="two">
        <v-list-item
          title="Account ID"
          :subtitle="isCopied(accountId!) ? 'Copied!' : accountId"
          :ripple="false"
          @click="copy(accountId!, accountId!)"
        >
        </v-list-item>
        <v-list-item
          title="Device ID"
          :subtitle="deviceId"
        >
        </v-list-item>
        <v-list-item
          title="Peer ID"
          :subtitle="peerId"
        >
        </v-list-item>
        <v-list-item
          title="Account status"
          :subtitle="`${accountStatus} (${statusExplanation})`"
        >
        </v-list-item>
      </v-list>
    </v-card>
    <v-card
      class="mt-4 text-center"
    >
      <v-card-title>
        <h3>
          Connectivity
        </h3>
      </v-card-title>
      <v-divider></v-divider>
      <v-list lines="two">
        <v-list-item>
          <p>
            You are currently connected to {{ ipfsConnections?.length || 0 }} IPFS nodes, including {{ nOrbiterDevices }} user devices from {{ nOrbiterAccounts }} Orbiter accounts.
          </p>
        </v-list-item>
        <template v-if="debug">
          <v-list-item
            v-for="conn in ipfsConnections"
            :key="conn.pair"
            v-list
            :title="conn.pair"
            :subtitle="conn.adresses.join(',\n')"
          >
          </v-list-item>
        </template>
      </v-list>
    </v-card>
  </v-container>
</template>
<script setup lang="ts">
import {computed, ref, watchEffect, onMounted} from 'vue';
import {selectTranslation} from '/@/utils';

import {useUserProfilePhoto} from '/@/components/users/utils';
import {useStaticStatus} from '../composables/staticStatus';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import { useCopyToClipboard } from '/@/composables/copyToClipboard';

const {orbiter} = useOrbiter();
// User name
const names = computed(() => orbiter?.listenForNameChange ? orbiter.listenForNameChange() : {});

const displayName = computed(() => {
  return selectTranslation(names.value) || 'Anonymous';
});

// Dev static mode
const {staticStatus} = useStaticStatus();
const debug = import.meta.env.VITE_DEBUG;

const staticModeSwitch = ref(staticStatus.value === 'static');
watchEffect(() => {
  staticStatus.value = staticModeSwitch.value ? 'static' : 'live';
});
watchEffect(() => {
  staticModeSwitch.value = staticStatus.value === 'static';
});

// Account and device ids
const accountId = ref<string | undefined>();
const deviceId = ref<string | undefined>();
const peerId = ref<string | undefined>();

onMounted(async () => {
  if (orbiter) {
    try {
      accountId.value = await orbiter.getAccountId();
      deviceId.value = await orbiter.getDeviceId();
      peerId.value = await orbiter.getPeerId();
    } catch (error) {
      console.error("Error fetching account/device/peer IDs:", error);
    }
  }
});

const { copy, isCopied } = useCopyToClipboard();

// User avatar
const userAvatar = useUserProfilePhoto(accountId); // useUserProfilePhoto might need adjustment if it expects a direct value not a ref

// Account status
const moderator = computed(() => orbiter?.followIsModerator ? orbiter.followIsModerator() : false);
const canUpload = computed(() => orbiter?.followCanUpload ? orbiter.followCanUpload() : false);
const accountStatus = computed(()=>{
  return moderator.value || (canUpload.value ? 'MEMBER' : 'GUEST');
});
const statusExplanation = computed(()=>{
  switch (accountStatus.value) {
    case 'ADMIN':
      return 'Can moderate content and invite other moderators or administrators.';
    case 'MODERATOR':
      return 'Can moderate content.';
    case 'MEMBER':
      return 'Can add content.';
    case 'GUEST':
      return 'View-only access to site.';
    default:
      return 'Unknown role';
  }
});

// Connectivity
const ipfsConnections = computed(() => orbiter?.constellation?.réseau?.suivreConnexionsPostesSFIP ? orbiter.constellation.réseau.suivreConnexionsPostesSFIP() : []);
const orbiterDevices = computed(() => orbiter?.constellation?.réseau?.suivreConnexionsDispositifs ? orbiter.constellation.réseau.suivreConnexionsDispositifs() : []);
const orbiterAccounts = computed(() => orbiter?.constellation?.réseau?.suivreConnexionsMembres ? orbiter.constellation.réseau.suivreConnexionsMembres() : []);
const nOrbiterDevices = computed(()=>{
  return orbiterDevices.value?.filter(d => d.infoDispositif.idDispositif !== deviceId.value).length || 0;
});
const nOrbiterAccounts = computed(()=>{
  return orbiterAccounts.value?.filter(acc=>acc.infoMembre.idCompte !== accountId.value).length || 0;
});

</script>
