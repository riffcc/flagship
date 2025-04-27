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
            :src="item.thumbnail"
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
      <template #item.contentCid="{item}">
        <span>{{
          lgAndUp ? item.contentCid : `${item.contentCid.slice(0, 6)}...${item.contentCid.slice(-6)}`
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
            prepend-icon="mdi-pencil"
            @click="editRelease(item.id)"
          >
            Edit
          </v-btn>
          <v-btn
            :prepend-icon="item.sourceSite === orbiter.siteId ? 'mdi-delete' : 'mdi-block-helper'"
            @click="deleteBlockRelease(item.id!, item.contentCid, item.sourceSite!)"
          >
            {{ item.sourceSite === orbiter.siteId ? 'Delete' : 'Block' }}
          </v-btn>
        </v-menu>
        <div
          v-else
          class="d-flex"
        >
          <v-btn
            icon="mdi-pencil"
            class="me-2"
            size="small"
            @click="editRelease(item.id)"
          ></v-btn>
          <v-btn
            :icon="item.sourceSite === orbiter.siteId ? 'mdi-delete' : 'mdi-block-helper'"
            size="small"
            @click="deleteBlockRelease(item.id!, item.contentCid, item.sourceSite!)"
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
import {useStaticReleases} from '/@/composables/staticReleases';
import {useOrbiter} from '/@/plugins/orbiter/utils';
// import { getStatusColor } from '/@/utils';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import ReleaseForm from '/@/components/releases/releaseForm.vue';
import { useStaticStatus } from '../../composables/staticStatus';
import type { PartialReleaseItem } from '/@/@types/release';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import { useReleasesStore } from '/@/stores/releases';
import { storeToRefs } from 'pinia';

const {staticStatus} = useStaticStatus();
const {lgAndUp, smAndDown} = useDisplay();
const {orbiter} = useOrbiter();
const {staticReleases} = useStaticReleases();
const releasesStore = useReleasesStore();
const {releases, isLoading } = storeToRefs(releasesStore);

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
  {title: 'Content CID', align: 'start', key: 'contentCid'},
  {title: 'Actions', key: 'actions', sortable: false},
];

const tableItems = computed(() => {
  if (staticStatus.value === 'static') {
    return staticReleases.value
    // .filter(r => r.status !== 'deleted')
    .map(r => ({
      id: r.id,
      thumbnail: r.thumbnail,
      name: r.name,
      category: r.category,
      contentCid: r.contentCID,
      sourceSite: r.sourceSite,
    }));
  } else {
    return releases.value;
  }
});

const editedRelease: Ref<PartialReleaseItem> = ref({
  name: '',
  contentCID: '',
  category: '',
  author: '',
  metadata: {},
});

const editReleaseDialog = ref(false);
const confirmDeleteBlockReleaseDialog = ref(false);
const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

function editRelease(id?: string) {
  if (!id) return;
  if (staticStatus.value === 'static') {
    const targetRelease = staticReleases.value.find(r => r.id === id);
    if (targetRelease) {
      editedRelease.value = {
        id: targetRelease.id,
        name: targetRelease.name,
        contentCID: targetRelease.contentCID,
        category: targetRelease.category,
        author: targetRelease.author,
        thumbnail: targetRelease.thumbnail,
        cover: targetRelease.cover,
        metadata: targetRelease.metadata ?? {},
      };
    }
  } else {
    const targetRelease = releases.value.find(r => r.id === id);
    if (targetRelease) {
      editedRelease.value = {
        id: targetRelease.id,
        name: targetRelease.name,
        contentCID: targetRelease.contentCID,
        category: targetRelease.category,
        author: targetRelease.author,
        thumbnail: targetRelease.thumbnail,
        cover: targetRelease.cover,
        metadata: targetRelease.metadata,
      };
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

function deleteBlockRelease(id: string, contentCID: string, sourceSite: string) {
  editedRelease.value = { id, contentCID, sourceSite };
  confirmDeleteBlockReleaseDialog.value = true;
}

async function confirmDeleteBlockRelease() {
  if (staticStatus.value === 'static') {
    const targetReleaseIndex = staticReleases.value.findIndex(r => r.id === editedRelease.value.id);
    if (targetReleaseIndex !== -1) {
      staticReleases.value.splice(targetReleaseIndex, 1);
    }
  } else {
    try {
      if (editedRelease.value.sourceSite === orbiter.siteId) {
        if (!editedRelease.value.id) throw Error('Target release id missing.');
        await orbiter.removeRelease(editedRelease.value.id!);
        openSnackbar('Release deleted successfully.', 'success');
      } else {
        if (!editedRelease.value.contentCID) throw Error('Target release content CID missing.');
        await orbiter.blockRelease({ cid: editedRelease.value.contentCID });
        openSnackbar('Release blocked successfully.', 'success');
      }
      resetEditedRelease();
    } catch (error) {
      if (editedRelease.value.sourceSite === orbiter.siteId) {
        console.error('Error deleting release:', error);
        openSnackbar('Error on deleting release.', 'error');
      } else {
        console.error('Error blocking release:', error);
        openSnackbar('Error on blocking release.', 'error');
      }

    }
  }
  confirmDeleteBlockReleaseDialog.value = false;
}

function resetEditedRelease() {
  editedRelease.value = {
    name: '',
    contentCID: '',
    category: '',
    author: '',
    metadata: {},
  };
};
</script>
