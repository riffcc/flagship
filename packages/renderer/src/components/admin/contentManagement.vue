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
              icon="mdi-dots-vertical"
              variant="text"
              v-bind="props"
            ></v-btn>
          </template>
          <v-btn
            prepend-icon="mdi-clipboard-multiple-outline"
            @click="copy(item.id!, item.id!)"
          >
            Copy ID
          </v-btn>
          <v-btn
            prepend-icon="mdi-pencil"
            @click="editRelease(item.id)"
          >
            Edit
          </v-btn>
          <v-btn
            prepend-icon="mdi-star-plus-outline"
            @click="requestFeatureRelease(item.id)"
          >
            Feature
          </v-btn>
          <!-- <v-btn
            :prepend-icon="item.sourceSite === orbiter.siteId ? 'mdi-delete' : 'mdi-block-helper'"
            @click="deleteBlockRelease(item.id!, item.contentCID, item.sourceSite!)"
          >
            {{ item.sourceSite === orbiter.siteId ? 'Delete' : 'Block' }}
          </v-btn> -->
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
            icon="mdi-pencil"
            class="me-2"
            size="small"
            @click="editRelease(item.id)"
          ></v-btn>
          <v-tooltip
            text="Feature Release"
            location="bottom"
          >
            <template #activator="{ props: tooltipProps }">
              <v-btn
                v-bind="tooltipProps"
                icon="mdi-star-plus-outline"
                class="me-2"
                size="small"
                @click="requestFeatureRelease(item.id)"
              ></v-btn>
            </template>
          </v-tooltip>
          <!-- <v-btn
            :icon="item.sourceSite === orbiter.siteId ? 'mdi-delete' : 'mdi-block-helper'"
            size="small"
            @click="deleteBlockRelease(item.id!, item.contentCID, item.sourceSite!)"
          ></v-btn> -->
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
      v-model="editReleaseDialog"
      max-width="500px"
    >
      <v-card class="py-3">
        <v-card-title>
          <span class="text-h6 ml-2">Edit Release</span>
        </v-card-title>

        <v-card-text>
          <release-form
            :initial-data="editedRelease"
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
            @click="editReleaseDialog = false"
          >
            Cancel
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <confirmation-dialog
      :message="`Are you sure you want to delete/block this release?`"
      :dialog-open="confirmDeleteBlockReleaseDialog"
      @close="() => {confirmDeleteBlockReleaseDialog = false}"
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
import {useStaticReleases} from '/@/composables/staticReleases';
import { useStaticStatus } from '/@/composables/staticStatus';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import { useCopyToClipboard } from '/@/composables/copyToClipboard';
import type { ReleaseItem, PartialReleaseItem } from '/@/types';
import {
  parseUrlOrCid,
  // getStatusColor,
 } from '/@/utils';
import type { AnyObject } from '@riffcc/lens-sdk';
import { useQuery } from '@tanstack/vue-query';


const {staticStatus} = useStaticStatus();
const {lgAndUp, smAndDown} = useDisplay();

const {staticReleases} = useStaticReleases();
const { data: releases, isLoading } = useQuery<ReleaseItem<AnyObject>[]>({
  queryKey: ['releases'],
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

const editedRelease: Ref<PartialReleaseItem<AnyObject>> = ref({
  id: '',
  name: '',
  categoryId: '',
  contentCID: '',
  metadata: {},
});

const editReleaseDialog = ref(false);
const confirmDeleteBlockReleaseDialog = ref(false);
const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

function editRelease(id?: string) {
  if (!id) return;
  if (staticStatus.value) {
    const targetRelease = staticReleases.value.find(r => r.id === id);
    if (targetRelease) {
      editedRelease.value = targetRelease;
    }
  } else {
    const targetRelease = releases.value?.find(r => r.id === id);
    if (targetRelease) {
      editedRelease.value = targetRelease;
    }
  }
  editReleaseDialog.value = true;
}

function handleSuccess(message: string) {
  openSnackbar(message, 'success');
  editReleaseDialog.value = false;
  resetEditedRelease();
}

function handleError(message: string) {
  openSnackbar(message, 'error');
  console.error('Error:', message);
}

// function deleteBlockRelease(id: string, contentCID: string) {
//   editedRelease.value = { id, contentCID };
//   confirmDeleteBlockReleaseDialog.value = true;
// }

async function confirmDeleteBlockRelease() {
  if (!editedRelease.value.id) return;
  if (staticStatus.value) {
    const targetReleaseIndex = releases.value?.findIndex(r => r.id === editedRelease.value.id);
    if (targetReleaseIndex && targetReleaseIndex !== -1) {
      releases.value?.splice(targetReleaseIndex, 1);
    }
  } else {
    // try {
    //   if (editedRelease.value.sourceSite === orbiter.siteId) {
    //     await orbiter.removeRelease(editedRelease.value.id);
    //     openSnackbar('Release deleted successfully.', 'success');
    //   } else {
    //     if (!editedRelease.value.contentCID) throw Error('Target release content CID missing.');
    //     await orbiter.blockRelease({ cid: editedRelease.value.contentCID });
    //     openSnackbar('Release blocked successfully.', 'success');
    //   }
    //   resetEditedRelease();
    // } catch (error) {
    //   if (editedRelease.value.sourceSite === orbiter.siteId) {
    //     console.error('Error deleting release:', error);
    //     openSnackbar('Error on deleting release.', 'error');
    //   } else {
    //     console.error('Error blocking release:', error);
    //     openSnackbar('Error on blocking release.', 'error');
    //   }
    // }
    openSnackbar('Not implemented.', 'error');

  }
  confirmDeleteBlockReleaseDialog.value = false;
  resetEditedRelease();
}

function resetEditedRelease() {
  editedRelease.value = {
    name: '',
    categoryId: '',
    contentCID: '',
    metadata: {},
  };
};

function requestFeatureRelease(releaseId: string | undefined) {
  if (releaseId) {
    emit('feature-release', releaseId);
  } else {
    console.warn('Attempted to feature a release with no ID.');
  }
}
</script>
