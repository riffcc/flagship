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
          <v-list v-if="subscriptions && subscriptions.length > 0">
            <v-list-item
              v-for="s in subscriptions"
              :key="s.id"
              :title="`${s.value[SUBSCRIPTION_SITE_ID_PROPERTY].slice(0,17)}...${s.value[SUBSCRIPTION_SITE_ID_PROPERTY].slice(-10)}`"
              :subtitle="s.value[SUBSCRIPTION_NAME_PROPERTY] || 'Unnamed subscription'"
            >
              <template
                v-if="showDefederation"
                #prepend
              >
                <v-menu>
                  <template #activator="{ props }">
                    <v-btn
                      v-bind="props"
                      icon="$circle"
                      variant="text"
                      density="compact"
                      size="x-small"
                      class="mr-2"
                      :color="getSiteColor(s.value[SUBSCRIPTION_SITE_ID_PROPERTY])"
                    />
                  </template>
                  <v-color-picker
                    v-model="selectedColors[s.value[SUBSCRIPTION_SITE_ID_PROPERTY]]"
                    @update:model-value="saveColor(s.value[SUBSCRIPTION_SITE_ID_PROPERTY], $event)"
                  />
                </v-menu>
              </template>
              <template #append>
                <v-btn
                  icon="$delete"
                  density="comfortable"
                  size="small"
                  @click="() => untrustSite({siteId: s.id})"
                ></v-btn>
              </template>
            </v-list-item>
          </v-list>
          <div
            v-else-if="!isLoading"
            class="d-flex h-75"
          >
            <span class="ma-auto text-body-2 text-medium-emphasis">No Subscriptions found.</span>
          </div>
          <v-progress-linear
            v-else
            indeterminate
            color="primary"
          />
        </v-sheet>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import {computed, ref} from 'vue';
// import { useLensService } from '/@/plugins/lensService';
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { SUBSCRIPTION_SITE_ID_PROPERTY, SUBSCRIPTION_NAME_PROPERTY } from '@riffcc/lens-sdk';
import { useSiteColors } from '/@/composables/siteColors';
import { useShowDefederation } from '/@/composables/showDefed';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';

const formRef = ref();
// const lensService = useLensService();
const queryClient = useQueryClient();
const { openSnackbar } = useSnackbarMessage();

const trustedSiteId = ref<string>();
const trustedSiteName = ref<string>();

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidSiteAddress: (v: string) =>
    /^[1-9A-HJ-NP-Za-km-z]+$/.test(v) || 'Please enter a valid site address (e.g., `zb2rhdS7GgY88eLJe1WptwXa9Zmibh1xTc5WCkSCox2sTDwuX`).',
};

// Query subscriptions
const { data: subscriptions, isLoading } = useQuery({
  queryKey: ['subscriptions'],
  queryFn: async () => {
    // TODO: Implement getSubscriptions in SDK
    console.warn('getSubscriptions not yet implemented in SDK');
    // Return typed empty array to avoid TypeScript issues
    type SubscriptionItem = {
      id: string;
      value: {
        [SUBSCRIPTION_SITE_ID_PROPERTY]: string;
        [SUBSCRIPTION_NAME_PROPERTY]?: string;
      }
    };
    return [] as SubscriptionItem[];
  },
  staleTime: 30000,
});

// Add subscription mutation
const addSubscriptionMutation = useMutation({
  mutationFn: async ({ siteId: _siteId, name: _name }: { siteId: string; name: string }) => {
    // TODO: Implement addSubscription in SDK
    console.warn('addSubscription not yet implemented in SDK');
    return { success: false, error: 'addSubscription not yet implemented' };
  },
  onSuccess: (result) => {
    if (result.success) {
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      openSnackbar('Subscription added successfully', 'success');
      clearForm();
    } else {
      openSnackbar(result.error || 'Failed to add subscription', 'error');
    }
  },
  onError: (error) => {
    openSnackbar(`Error: ${error.message}`, 'error');
  },
});

// Delete subscription mutation
const deleteSubscriptionMutation = useMutation({
  mutationFn: async (id: string) => {
    // TODO: Implement deleteSubscription in SDK
    console.warn('deleteSubscription not yet implemented in SDK', id);
    return { success: false, error: 'deleteSubscription not yet implemented' };
  },
  onSuccess: (result: { success?: boolean; error?: string }) => {
    if (result && result.success) {
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
      openSnackbar('Subscription removed', 'success');
    } else {
      openSnackbar((result && result.error) || 'Failed to remove subscription', 'error');
    }
  },
});

const readyToSave = computed(() => {
  if (trustedSiteId.value && trustedSiteName.value && formRef.value?.isValid) {
    return {
      trustedSiteIdValue: trustedSiteId.value,
      trustedSiteNameValue: trustedSiteName.value,
    };
  } else return undefined;
});

const loading = computed(() => addSubscriptionMutation.isPending.value);

const handleOnSubmit = async () => {
  if (!readyToSave.value) return;
  const {trustedSiteIdValue, trustedSiteNameValue} = readyToSave.value;
  
  await addSubscriptionMutation.mutateAsync({
    siteId: trustedSiteIdValue,
    name: trustedSiteNameValue,
  });
};

const clearForm = () => {
  trustedSiteId.value = undefined;
  trustedSiteName.value = undefined;
};

const untrustSite = ({siteId}: {siteId: string}) => {
  deleteSubscriptionMutation.mutate(siteId);
};

const {getSiteColor, saveColor, selectedColors} = useSiteColors();
const { showDefederation } = useShowDefederation();
</script>
