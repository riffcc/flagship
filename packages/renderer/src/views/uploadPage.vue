<template>
  <v-container>
    <v-sheet
      width="480px"
      class="px-8 pb-16 pt-10 mx-auto"
    >
      <release-form
        v-if="canUpload"
        @update:success="handleSuccess"
        @update:error="handleError"
      />
      <v-alert
        v-else-if="canUpload === false"
        type="info"
        class="mt-4"
        color="black"
        text-color="white"
      >
        You aren't currently authorised to add releases to this instance of Riff.CC.
      </v-alert>
      <div
        v-else
      >
        <v-alert
          type="info"
          class="mt-4"
          color="black"
          text-color="white"
        >
          Loading authorisation data...
        </v-alert>
        <v-skeleton-loader
          type="list-item"
        />
      </div>
    </v-sheet>
  </v-container>
  <v-snackbar
    v-model="showSnackbar"
    :color="snackbarMessage?.type ?? 'default'"
  >
    {{ snackbarMessage?.text }}
    <template #actions>
      <v-btn
        color="white"
        variant="text"
        @click="closeSnackbar"
      >
        Close
      </v-btn>
    </template>
  </v-snackbar>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {useOrbiter} from '/@/plugins/flagship/utils';
import releaseForm from '/@/components/releases/releaseForm.vue';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';

const {orbiter} = useOrbiter();
const canUpload = computed(() => orbiter?.followCanUpload ? orbiter.followCanUpload() : undefined);
const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

function handleSuccess(message: string) {
  openSnackbar(message, 'success');
}

function handleError(message: string) {
  openSnackbar(message, 'error');
  console.error('Error:', message);
}
</script>
