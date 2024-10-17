<template>
  <v-container class="text-center">
    <h1>Admin</h1>
    <h2>My site id</h2>
    <h3>{{ siteId }} </h3>
    <h2>Trusted sites</h2>
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
        rounded="0"
        color="primary"
        type="submit"
        block
        text="Trust site"
        :disabled="!readyToSave"
        :loading="loading"
      />
    </v-form>
  </v-container>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useOrbiter } from '../plugins/orbiter/utils';
import { adresseOrbiteValide } from '@constl/utils-ipa';
import { obt } from '@constl/vue';

const {orbiter} = useOrbiter();
const formRef = ref();

const trustedSiteId = ref<string>();
const trustedSiteName = ref<string>();

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidSiteAddress: (v: string) => adresseOrbiteValide(v) || 'Please enter a valid site address (`/orbitdb/...`).',
};

const readyToSave = computed(() => {
  if (
    trustedSiteId.value &&
    trustedSiteName.value &&
    formRef.value.isValid
  ) {

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

const siteConfig = obt(orbiter.siteConfigured.bind(orbiter));
const siteId = computed(()=>siteConfig.value?.siteId);
</script>
