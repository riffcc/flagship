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
        {{ displayName }}
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
          :subtitle="publicKey || 'Fetching Public Key...'"
        >
        </v-list-item>
        <v-list-item
          title="Peer ID"
          :subtitle="peerId || 'Fetching Peer ID...'"
        >
        </v-list-item>
        <v-list-item
          title="Account status"
          :subtitle="`${accountStatus} (${statusExplanation})`"
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
          <p>
            You are currently connected to {{ ipfsConnections?.length || 0 }} IPFS nodes, including {{ nOrbiterDevices
            }} user devices from {{ nOrbiterAccounts }} Orbiter accounts.
          </p>
        </v-list-item>
      </v-list>
    </v-card>
  </v-container>
</template>
<script setup lang="ts">
import { computed, onMounted, ref, watchEffect, inject, type Ref } from 'vue';
import { selectTranslation } from '/@/utils';

import { useStaticStatus } from '../composables/staticStatus';
import { useOrbiter } from '/@/plugins/peerbit/utils';
import { useCopyToClipboard } from '/@/composables/copyToClipboard';
import { AccountType } from '/@/lib/schema';
import { useUserSession } from '../composables/userSession';
import type { IPeerbitService } from '/@/lib/types';

const { orbiter } = useOrbiter();
const { userData } = useUserSession();

const peerbitServiceRef = inject<Ref<IPeerbitService | undefined>>('peerbitService');

const names = computed(() => {
  return orbiter.listenForNameChange();
});

const displayName = computed(() => {
  return selectTranslation(names.value) || 'Anonymous';
});

const { staticStatus } = useStaticStatus();

const staticModeSwitch = ref(staticStatus.value === 'static');
watchEffect(() => {
  staticStatus.value = staticModeSwitch.value ? 'static' : 'live';
});
watchEffect(() => {
  staticModeSwitch.value = staticStatus.value === 'static';
});

const publicKey = ref<string | undefined>();
const peerId = ref<string | undefined>();
const fetchError = ref<string | null>(null);

watchEffect(async () => {
  if (peerbitServiceRef?.value) {
    console.log('[AccountPage] watchEffect: Peerbit service is now available. Fetching account details...');
    fetchError.value = null;
    try {
      publicKey.value = await peerbitServiceRef.value.getPublicKey();
      peerId.value = await peerbitServiceRef.value.getPeerId();
      console.log('[AccountPage] watchEffect: Fetched PK:', publicKey.value, 'PeerID:', peerId.value);
    } catch (e: any) {
      console.error('[AccountPage] watchEffect: Error fetching account details:', e);
      fetchError.value = e.message || 'Failed to fetch account details.';
    }
  } else {
    console.log('[AccountPage] watchEffect: Peerbit service not yet available.');
  }
});

const accountStatus = computed<AccountType>(() => {
  return AccountType.GUEST;
});
const statusExplanation = computed(() => {
  switch (accountStatus.value) {
    case AccountType.ADMIN:
      return 'Can moderate content and invite other moderators or administrators.';
    case AccountType.MODERATOR:
      return 'Can moderate content.';
    case AccountType.USER:
      return 'Can add content.';
    case AccountType.GUEST:
      return 'View-only access to site.';
    default:
      return 'Unknown role';
  }
});

const ipfsConnections = computed(() => orbiter?.constellation?.réseau?.suivreConnexionsPostesSFIP ? orbiter.constellation.réseau.suivreConnexionsPostesSFIP() : []);
const nOrbiterDevices = computed(() => {
  return 0;
});
const nOrbiterAccounts = computed(() => {
  return 0;
});

onMounted(async () => {
  // accountId.value = await orbiter.getAccountId();
  // peerId.value = await orbiter.getPeerId();
});

</script>
