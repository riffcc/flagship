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
          v-if="publicKey"
          title="Public Key"
          :subtitle="isCopied(publicKey) ? 'Copied!' : publicKey"
          @click="copy(publicKey, publicKey)"
        >
        </v-list-item>
        <v-list-item
          v-if="peerId"
          title="Peer ID"
          :subtitle="isCopied(peerId) ? 'Copied!' : peerId"
          @click="copy(peerId, peerId)"
        >
        </v-list-item>
        <v-list-item
          v-if="accountStatus !== undefined"
          title="Account status"
          :subtitle="`${statusExplanation.title} ${statusExplanation.description}`"
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
import { computed } from 'vue';

import { useUserSession } from '/@/composables/userSession';
import { useAccountStatusQuery, usePeerIdQuery, usePublicKeyQuery } from '/@/plugins/lensService/hooks';
import { useCopyToClipboard } from '../composables/copyToClipboard';

const { userData } = useUserSession();
const { copy, isCopied } = useCopyToClipboard();

const { data: publicKey } = usePublicKeyQuery();
const { data: peerId } = usePeerIdQuery();
const { data: accountStatus } = useAccountStatusQuery();

const statusExplanation = computed(() => {
  switch (accountStatus.value) {
    case 0:
      return {
        title: 'GUEST',
        description: '(View-only access to site.)',
      };
    case 1:
      return {
        title: 'MEMBER',
        description: '(Can add content.)',
      };
    case 2:
      return {
        title: 'ADMIN',
        description: '(Can moderate content and invite other moderators or administrators.)',
      };
    default:
      return {
        title: 'Unknown role',
        description: '',
      };
  }
});

</script>
