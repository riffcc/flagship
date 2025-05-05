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
      </v-list>
    </v-card>
    <v-sheet
      v-if="debug"
      class="d-flex flex-column flex-sm-row ga-2 mx-auto mt-4 justify-center"
      color="transparent"
    >
      <v-btn @click="logPeers">log peers</v-btn>
      <v-btn @click="logTopics">log topics</v-btn>
      <v-btn @click="logConnections">log connections</v-btn>
    </v-sheet>
  </v-container>
</template>
<script setup lang="ts">
import {computed, ref, watchEffect} from 'vue';
import {selectTranslation} from '/@/utils';

import {suivre as follow, obt} from '@constl/vue';
import {useUserProfilePhoto} from '/@/components/users/utils';
import {useStaticStatus} from '../composables/staticStatus';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import { useCopyToClipboard } from '/@/composables/copyToClipboard';

const {orbiter} = useOrbiter();
// User name
const names = follow(orbiter.listenForNameChange.bind(orbiter));

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
const accountId = follow(orbiter.constellation.suivreIdCompte);
const deviceId = obt(orbiter.constellation.obtIdDispositif);
const peerId = obt(orbiter.constellation.obtIdLibp2p);

const { copy, isCopied } = useCopyToClipboard();

// User avatar
const userAvatar = useUserProfilePhoto(accountId.value);

// Account status
const moderator = follow(orbiter.followIsModerator.bind(orbiter));
const canUpload = follow(orbiter.followCanUpload.bind(orbiter));
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
const ipfsConnections = follow(orbiter.constellation.réseau.suivreConnexionsPostesSFIP);
const orbiterDevices = follow(orbiter.constellation.réseau.suivreConnexionsDispositifs);
const orbiterAccounts = follow(orbiter.constellation.réseau.suivreConnexionsMembres);
const nOrbiterDevices = computed(()=>{
  return orbiterDevices.value?.filter(d => d.infoDispositif.idDispositif !== deviceId.value).length || 0;
});
const nOrbiterAccounts = computed(()=>{
  return orbiterAccounts.value?.filter(acc=>acc.infoMembre.idCompte !== accountId.value).length || 0;
});

const logPeers = async () => {
  const { sfip } = await orbiter.constellation.attendreSfipEtOrbite();
  console.log(sfip.libp2p.getPeers().map(p => p.toString()));
};
const logTopics = async () => {
  const { sfip } = await orbiter.constellation.attendreSfipEtOrbite();
  console.log(sfip.libp2p.services.pubsub.getTopics());
};
const logConnections = async () => {
  const forgetFn = await orbiter.constellation.réseau.suivreConnexionsPostesSFIP({
    f: (connections) =>  console.log(connections.map(c => `${c.pair}\n${c.adresses.join('\n')}`).join('\n\n')),
  });
  await forgetFn();
};
</script>
