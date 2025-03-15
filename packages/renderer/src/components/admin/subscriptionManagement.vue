<template>
  <v-container>
    <v-row justify="center">
      <v-col
        cols="12"
        md="6"
        lg="5"
      >
        <v-sheet
          class="px-6 py-4 mx-auto"
          max-width="448px"
        >
          <h6 class="text-h6 font-weight-bold mb-4">New Subscription</h6>
          <v-form
            ref="formRef"
            validate-on="input lazy"
            class="d-flex flex-column ga-2"
            @submit.prevent="handleOnSubmit"
          >
            <v-text-field
              v-model="trustedSiteName"
              label="Site Name"
              :rules="[rules.required]"
            />
            <v-text-field
              v-model="trustedSiteId"
              label="Site Id"
              :rules="[rules.isValidSiteAddress]"
            />
            <v-btn
              color="primary"
              type="submit"
              block
              text="Subscribe"
              :disabled="!readyToSave"
              :loading="loading"
            />
          </v-form>
        </v-sheet>
      </v-col>
      <v-col
        cols="12"
        md="6"
        lg="5"
      >
        <v-sheet
          class="px-6 py-4 mx-auto h-100"
          max-width="448px"
          min-height="256px"
        >
          <h6 class="text-h6 font-weight-bold mb-4">Subscriptions</h6>
          <v-list v-if="trustedSites && trustedSites?.length > 0">
            <v-list-item
              v-for="s in trustedSites"
              :key="s.id"
              :title="`${s.données.siteId.slice(0,17)}...${s.données.siteId.slice(-10)}`"
              :subtitle="s.données.siteName"
            >
              <template
                v-if="showDefederation"
                #prepend
              >
                <v-menu>
                  <template #activator="{ props }">
                    <v-btn
                      v-bind="props"
                      icon="mdi-circle"
                      variant="text"
                      density="compact"
                      size="x-small"
                      class="mr-2"
                      :color="getSiteColor(s.données.siteId)"
                    />
                  </template>
                  <v-color-picker
                    v-model="selectedColors[s.données.siteId]"
                    @update:model-value="saveColor(s.données.siteId, $event)"
                  />
                </v-menu>
              </template>
              <template #append>
                <v-btn
                  icon="mdi-delete"
                  density="comfortable"
                  size="small"
                  @click="() => untrustSite({siteId: s.id})"
                ></v-btn>
              </template>
            </v-list-item>
          </v-list>
          <div
            v-else
            class="d-flex h-75"
          >
            <span class="ma-auto text-body-2 text-medium-emphasis">No Subscriptions found.</span>
          </div>
        </v-sheet>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import {adresseOrbiteValide} from '@constl/utils-ipa';
import {suivre as follow} from '@constl/vue';
import {computed, ref} from 'vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import { useSiteColors } from '/@/composables/siteColors';
import { useShowDefederation } from '/@/composables/showDefed';

const {orbiter} = useOrbiter();
const formRef = ref();

const trustedSiteId = ref<string>();
const trustedSiteName = ref<string>();

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidSiteAddress: (v: string) =>
    adresseOrbiteValide(v) || 'Please enter a valid site address (`/orbitdb/...`).',
};

const readyToSave = computed(() => {
  if (trustedSiteId.value && trustedSiteName.value && formRef.value.isValid) {
    return {
      trustedSiteIdValue: trustedSiteId.value,
      trustedSiteNameValue: trustedSiteName.value,
    };
  } else return undefined;
});

const loading = ref(false);
const handleOnSubmit = async () => {
  if (!readyToSave.value) return;
  const {trustedSiteIdValue, trustedSiteNameValue} = readyToSave.value;
  loading.value = true;

  await orbiter.trustSite({
    siteId: trustedSiteIdValue,
    siteName: trustedSiteNameValue,
  });
  clearForm();
  loading.value = false;
};

const clearForm = () => {
  trustedSiteId.value = undefined;
  trustedSiteName.value = undefined;
};

// const siteConfig = obt(orbiter.siteConfigured.bind(orbiter));
// const siteId = computed(() => siteConfig.value?.siteId);

const trustedSites = follow(orbiter.followTrustedSites.bind(orbiter));

// const siteDomainName = computed(() => {
//   return document.location.hostname;
// });

const untrustSite = async ({siteId}: {siteId: string}) => {
  await orbiter.untrustSite({siteId});
};

const {getSiteColor, saveColor, selectedColors} = useSiteColors();
const {showDefederation} = useShowDefederation();
</script>
