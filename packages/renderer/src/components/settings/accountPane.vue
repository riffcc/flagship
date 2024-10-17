<template>
  <div>
    <h2>Account info</h2>
    <p>
      <v-chip
        label
        class="my-2 me-2"
      >
        Username: {{ displayName }}
      </v-chip>
      <v-chip
        label
        class="my-2 me-2"
      >
        Account ID: {{ account ? account.slice(9, 25) + '...' : '' }}
        <v-icon
          small
          @click="() => copyAccountId()"
        >
          {{ accountIdCopied ? 'mdi-check' : 'mdi-content-copy' }}
        </v-icon>
      </v-chip>
    </p>
    <v-divider class="my-4" />
    <h2>Advanced</h2>
    <p>
      You are on the site <code>{{ siteDomainName }}</code>. If you are the moderator of another Orbiter site and wish to add this site to your list of
      trusted sites, copy the information below:
      <v-text-field
        class="my-2"
        variant="outlined"
        readonly
        :append-inner-icon="siteInfoCopied ? 'mdi-check' : 'mdi-content-copy'"
        @click:append-inner="() => copySiteInfo()"
      >
        Site info: {{ siteId ? hashedSiteInfo.slice(0, 35) + '...' : '' }}
      </v-text-field>
    </p>

    <v-divider />

    <v-btn
      color="error"
      class="my-6"
      variant="text"
      outlined
      @click="deleteAccount"
    >
      Delete my account
    </v-btn>
  </div>
</template>

<script setup lang="ts">
import {computed, onMounted, ref} from 'vue';

import {suivre as follow} from '@constl/vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import {copyText, selectTranslation} from '/@/utils';

const {orbiter} = useOrbiter();

const account = follow(orbiter.listenForAccountId);
const names = follow(orbiter.listenForNameChange);

const accountIdCopied = ref(false);
const copyAccountId = async () => {
  await copyText(account.value);
  accountIdCopied.value = true;
};

const displayName = computed(() => {
  return selectTranslation(names.value) || 'Anonymous';
});

const siteDomainName = computed(() => {
  return document.location.hostname;
});

const siteInfoCopied = ref(false);
const siteId = ref<string>();

const hashedSiteInfo = computed(() => {
  return btoa(JSON.stringify({siteId: siteId.value, siteName: siteDomainName.value}));
});
const copySiteInfo = async () => {
  await copyText(hashedSiteInfo.value);
  siteInfoCopied.value = true;
};

async function deleteAccount() {
  await orbiter.deleteAccount();
}

onMounted(async () => {
  // These don't need to be dynamically followed, just noted once ready
  siteId.value = (await orbiter.siteConfigured()).siteId;
});
</script>
