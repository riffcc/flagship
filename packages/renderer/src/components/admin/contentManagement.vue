<!-- eslint-disable vue/valid-v-slot -->
<template>
  <v-container>
    <v-data-table
      :headers="smAndDown ? smTableHeaders : tableHeaders"
      :items="tableItems"
      :loading="isLoading"
      hide-default-header
    >
      <template #item.thumbnail="{item}">
        <v-card
          class="my-2"
          elevation="2"
          rounded
        >
          <v-img
            :src="parseUrlOrCid(item.thumbnailCID)"
            height="64"
            width="113"
          ></v-img>
        </v-card>
      </template>
      <template #item.name="{item}">
        <v-container :max-width="smAndDown ? '128px' : '256px'">
          <span>{{ item.name }}</span>
        </v-container>
      </template>
      <template #item.contentCID="{item}">
        <span>{{
          lgAndUp ? item.contentCID : `${item.contentCID.slice(0, 6)}...${item.contentCID.slice(-6)}`
        }}</span>
      </template>
      <template #item.actions="{item}">
        <v-menu v-if="smAndDown">
          <template #activator="{props}">
            <v-btn
              icon="$dots-vertical"
              variant="text"
              v-bind="props"
            ></v-btn>
          </template>
          <v-btn
            prepend-icon="$clipboard-multiple-outline"
            @click="copy(item.id!, item.id!)"
          >
            Copy ID
          </v-btn>
          <v-btn
            prepend-icon="$pencil"
            @click="targetReleaseToEdit = item"
          >
            Edit
          </v-btn>
          <v-btn
            prepend-icon="$star-plus-outline"
            @click="requestFeatureRelease(item.id)"
          >
            Feature
          </v-btn>
          <v-btn
            prepend-icon="$delete"
            @click="targetReleaseToDelete = item"
          >
            Delete
          </v-btn>
        </v-menu>
        <div
          v-else
          class="d-flex"
        >
          <v-tooltip
            text="Copy Release ID"
            location="bottom"
          >
            <template #activator="{ props: tooltipProps }">
              <v-btn
                v-bind="tooltipProps"
                :icon="getIcon(item.id!)"
                :color="getColor(item.id!)"
                class="me-2"
                size="small"
                variant="text"
                @click="copy(item.id!, item.id!)"
              ></v-btn>
            </template>
          </v-tooltip>
          <v-btn
            icon="$pencil"
            class="me-2"
            size="small"
            @click="targetReleaseToEdit = item"
          ></v-btn>
          <v-tooltip
            text="Feature Release"
            location="bottom"
          >
            <template #activator="{ props: tooltipProps }">
              <v-btn
                v-bind="tooltipProps"
                icon="$star-plus-outline"
                class="me-2"
                size="small"
                @click="requestFeatureRelease(item.id)"
              ></v-btn>
            </template>
          </v-tooltip>
          <v-btn
            icon="$delete"
            size="small"
            @click="targetReleaseToDelete = item"
          ></v-btn>
        </div>
      </template>
      <!-- <template #item.status="{item}">
        <v-chip
          :color="getStatusColor(item.status)"
          class="text-uppercase text-caption"
        >
          {{ item.status }}
        </v-chip>
      </template> -->
    </v-data-table>
    <v-dialog
      :model-value="Boolean(targetReleaseToEdit)"
      max-width="500px"
    >
      <v-card class="py-3">
        <v-card-title>
          <span class="text-h6 ml-2">Edit Release</span>
        </v-card-title>

        <v-card-text>
          <release-form
            v-if="targetReleaseToEdit"
            :initial-data="targetReleaseToEdit"
            mode="edit"
            @update:success="handleSuccess"
            @update:error="handleError"
          />
        </v-card-text>

        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn
            color="blue-darken-1"
            variant="text"
            @click="targetReleaseToEdit = null"
          >
            Cancel
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <confirmation-dialog
      :message="`Are you sure you want to delete this release?`"
      :dialog-open="Boolean(targetReleaseToDelete)"
      @close="() => { targetReleaseToDelete = null }"
      @confirm="confirmDeleteBlockRelease"
    ></confirmation-dialog>
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
import {computed, ref, type Ref} from 'vue';
import {useDisplay} from 'vuetify';
import ReleaseForm from '/@/components/releases/releaseForm.vue';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import {useStaticData} from '../../composables/staticData';
import { useStaticStatus } from '/@/composables/staticStatus';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import { useCopyToClipboard } from '/@/composables/copyToClipboard';
import type { ReleaseItem } from '/@/types';
import {
  parseUrlOrCid,
  // getStatusColor,
 } from '/@/utils';
import type { AnyObject } from '@riffcc/lens-sdk';
import { useDeleteReleaseMutation, useGetReleasesQuery } from '/@/plugins/lensService/hooks';


const {staticStatus} = useStaticStatus();
const {lgAndUp, smAndDown} = useDisplay();

const {staticReleases} = useStaticData();
const { data: releases, isLoading } = useGetReleasesQuery();
const deleteReleaseMutation = useDeleteReleaseMutation({
  onSuccess: () => {
    openSnackbar('Release deleted successfully.', 'success');
    targetReleaseToDelete.value = null;
  },
  onError: (e) => {
    console.error('Error blocking release:', e);
    openSnackbar('Error on blocking release.', 'error');
  },
});

const { copy, getIcon, getColor } = useCopyToClipboard();
const emit = defineEmits<{
  'feature-release': [id: string]
}>();

type Header = {
  title: string;
  align?: 'start' | 'end' | 'center';
  sortable?: boolean;
  key: string;
};
const smTableHeaders: Header[] = [
  {title: 'ID', align: 'start', key: 'id'},
  {title: 'Name', align: 'start', key: 'name'},
  {title: 'Actions', key: 'actions', sortable: false},
];
const tableHeaders: Header[] = [
  {title: 'ID', align: 'start', key: 'id'},
  {
    title: 'Thumbnail',
    align: 'start',
    key: 'thumbnail',
  },
  {title: 'Name', align: 'start', key: 'name'},
  {title: 'Category', align: 'start', key: 'category'},
  {title: 'Content CID', align: 'start', key: 'contentCID'},
  {title: 'Actions', key: 'actions', sortable: false},
];

const tableItems = computed(() => {
  if (staticStatus.value) {
    return staticReleases.value;
  } else {
    return releases.value;
  }
});

const targetReleaseToEdit = ref<ReleaseItem<AnyObject> | null>(null);
const targetReleaseToDelete = ref<ReleaseItem<AnyObject> | null>(null);

const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

function handleSuccess(message: string) {
  openSnackbar(message, 'success');
  targetReleaseToEdit.value = null;
}

function handleError(message: string) {
  openSnackbar(message, 'error');
  console.error('Error:', message);
}

async function confirmDeleteBlockRelease() {
  if (!targetReleaseToDelete.value) return;
  deleteReleaseMutation.mutate({ id: targetReleaseToDelete.value.id });
}

function requestFeatureRelease(releaseId: string | undefined) {
  if (releaseId) {
    emit('feature-release', releaseId);
  } else {
    console.warn('Attempted to feature a release with no ID.');
  }
}
</script>
