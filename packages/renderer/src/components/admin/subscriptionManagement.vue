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
              v-model="newSubcription.to"
              label="Site Address"
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
              :title="`${s.to.slice(0, 17)}...${s.to.slice(-10)}`"
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
                      :color="getSiteColor(s.to)"
                    />
                  </template>
                  <v-color-picker
                    v-model="selectedColors[s.to]"
                    @update:model-value="saveColor(s.to, $event)"
                  />
                </v-menu>
              </template>
              <template #append>
                <v-btn
                  icon="$delete"
                  density="comfortable"
                  size="small"
                  @click="() => unsubscribe({ id: s.id })"
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
import { computed, ref } from 'vue';
import { useGetSubscriptionsQuery, useAddSubscriptionMutation, useDeleteSubscriptionMutation } from '/@/plugins/lensService';
import { type SubscriptionData } from '@riffcc/lens-sdk';
import { useSiteColors } from '/@/composables/siteColors';
import { useShowDefederation } from '/@/composables/showDefed';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';

const formRef = ref();
const { openSnackbar } = useSnackbarMessage();

const newSubcription = ref<SubscriptionData>({
  to: '',
});

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidSiteAddress: (v: string) =>
    /^[1-9A-HJ-NP-Za-km-z]+$/.test(v) || 'Please enter a valid site address (e.g., `zb2rhdS7GgY88eLJe1WptwXa9Zmibh1xTc5WCkSCox2sTDwuX`).',
};

// Query subscriptions using our new hook
const { data: subscriptions, isLoading } = useGetSubscriptionsQuery();

// Add subscription mutation using our new hook
const addSubscriptionMutation = useAddSubscriptionMutation({
  onSuccess: (result) => {
    if (result.success) {
      openSnackbar('Subscription added successfully', 'success');
      newSubcription.value = {
        to: '',
      };
    } else {
      openSnackbar(result.error || 'Failed to add subscription', 'error');
    }
  },
  onError: (error) => {
    openSnackbar(`Error: ${error.message}`, 'error');
  },
});

// Delete subscription mutation using our new hook
const deleteSubscriptionMutation = useDeleteSubscriptionMutation({
  onSuccess: (result) => {
    if (result.success) {
      openSnackbar('Subscription removed', 'success');
    } else {
      openSnackbar(result.error || 'Failed to remove subscription', 'error');
    }
  },
  onError: (error) => {
    openSnackbar(`Error: ${error.message}`, 'error');
  },
});

const readyToSave = computed(() => {
  if (newSubcription.value && newSubcription.value.to !== '' && formRef.value?.isValid) {
    return {
      to: newSubcription.value.to,
    };
  } else return undefined;
});

const loading = computed(() => addSubscriptionMutation.isPending.value);

const handleOnSubmit = async () => {
  if (!readyToSave.value) return;

  await addSubscriptionMutation.mutateAsync(readyToSave.value);
};

const unsubscribe = ({ id }: { id: string }) => {
  deleteSubscriptionMutation.mutate({ id: id });
};

const {getSiteColor, saveColor, selectedColors} = useSiteColors();
const { showDefederation } = useShowDefederation();
</script>
