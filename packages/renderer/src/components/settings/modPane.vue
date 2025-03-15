<template>
  <div>
    <h2>Trusted sites</h2>
    <div class="my-2 text-center">
      <TrustSiteDialog>
        <template #activator="{props}">
          <v-btn
            v-bind="props"
            color="primary"
          >
            Add trusted site
          </v-btn>
        </template>
      </TrustSiteDialog>
    </div>

    <v-list>
      <TrustedSiteListItem
        v-for="site in trustedSites"
        :key="site.id"
        :site="site"
      />
    </v-list>

    <v-divider class="my-2" />

    <!-- <h2>Blocked CIDs</h2>
    <v-list>
      <v-list-item>
        <v-list-item-title>
          <v-text-field
            v-model="toBlock"
            class="mt-2"
            label="Block CID"
            variant="outlined"
            append-icon="mdi-check"
            @click:append="() => toBlock && blockRelease(toBlock)"
            @keyup.enter="() => toBlock && blockRelease(toBlock)"
          />
        </v-list-item-title>
      </v-list-item>
      <v-list-item
        v-for="b in blockedCIDs"
        :key="b.id"
      >
        <v-list-item-title>{{ b.cid }}</v-list-item-title>
        <template #append>
          <v-btn
            variant="text"
            @click="() => unblockRelease(b.id)"
          >
            Unblock
          </v-btn>
        </template>
      </v-list-item>
    </v-list> -->
  </div>
</template>

<script setup lang="ts">
import {onMounted, onUnmounted, ref} from 'vue';

import type {tableaux} from '@constl/ipa';
import type {types as orbiterTypes} from '@riffcc/orbiter';

import TrustSiteDialog from './trustedSites/AddTrustedSite.vue';
import TrustedSiteListItem from './trustedSites/TrustedSiteItem.vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();

// const blockedCIDs = ref<{cid: string; id: string}[]>();
// const toBlock = ref<string>();

const trustedSites = ref<tableaux.élémentDonnées<orbiterTypes.TrustedSite>[]>();

// const blockRelease = async (cid: string) => {
//   await orbiter.blockRelease({cid});
//   toBlock.value = undefined;
// };
// const unblockRelease = async (id: string) => {
//   await orbiter.unblockRelease({id});
// };

// let forgetBlockedCIDs: (() => Promise<void>) | undefined = undefined;
let forgetTrustedSites: (() => Promise<void>) | undefined = undefined;

onMounted(async () => {
  // forgetBlockedCIDs = await orbiter.listenForSiteBlockedReleases({f: x => (blockedCIDs.value = x)});
  forgetTrustedSites = await orbiter.followTrustedSites({f: x => (trustedSites.value = x)});
});

onUnmounted(async () => {
  // if (forgetBlockedCIDs) await forgetBlockedCIDs();
  if (forgetTrustedSites) await forgetTrustedSites();
});
</script>
