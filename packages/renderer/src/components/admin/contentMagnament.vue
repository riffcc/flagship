<!-- eslint-disable vue/valid-v-slot -->
<template>
  <v-container>
    <v-data-table
      :headers="smAndDown ? smTableHeaders : tableHeaders"
      :items="tableItems"
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
            prepend-icon="mdi-delete"
            @click="deleteRelease(item.id)"
          >
            Delete
          </v-btn>
        </v-menu>
        <div
          v-else
          class="d-flex"
        >
          <v-icon
            class="me-2"
            size="small"
            @click="editRelease(item.id)"
          >
            mdi-pencil
          </v-icon>
          <v-icon
            size="small"
            @click="deleteRelease(item.id)"
          >
            mdi-delete
          </v-icon>
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
          <v-text-field
            v-model="editedRelease.name"
            label="Name"
          ></v-text-field>
          <v-select
            v-model="editedRelease.category"
            label="Category"
            :items="consts.CONTENT_CATEGORIES"
            variant="solo"
          ></v-select>
          <v-text-field
            v-model="editedRelease.contentCid"
            label="Content CID"
          ></v-text-field>
          <!-- <v-select
            v-model="editedRelease.status"
            label="Status"
            :items="orbiter.statusType.filter(s => s !== 'deleted')"
            variant="solo"
          ></v-select> -->
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
          <v-btn
            color="blue-darken-1"
            variant="text"
            @click="saveRelease"
          >
            Save
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <confirmation-dialog
      message="Are you sure you want to delete this release?"
      :dialog-open="confirmDeleteReleaseDialog"
      @close="() => {confirmDeleteReleaseDialog = false}"
      @confirm="confirmDeleteRelease"
    ></confirmation-dialog>
  </v-container>
</template>
<script setup lang="ts">
import {computed, ref, type Ref} from 'vue';
import {suivre as follow} from '@constl/vue';
import {consts} from '@riffcc/orbiter';
import {useDisplay} from 'vuetify';
import {useStaticReleases} from '/@/composables/staticReleases';
import {useOrbiter} from '/@/plugins/orbiter/utils';
// import { getStatusColor } from '/@/utils';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import { useDevStatus } from '/@/composables/devStatus';

const {status} = useDevStatus();
const {lgAndUp, smAndDown} = useDisplay();
const {orbiter} = useOrbiter();
const {staticReleases} = useStaticReleases();
const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));


type Header = {
  title: string;
  align?: 'start' | 'end' | 'center';
  sortable?: boolean;
  key: string;
};
const smTableHeaders: Header[] = [
  {title: 'ID', align: 'start', key: 'id'},
  {title: 'Name', align: 'start', key: 'name'},
  {title: 'Status', align: 'start', key: 'status'},
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
  // {title: 'Status', align: 'start', key: 'status'},
  {title: 'Actions', key: 'actions', sortable: false},
];

const tableItems = computed(() => {
  if (status.value === 'static') {
    return staticReleases.value
    // .filter(r => r.status !== 'deleted')
    .map(r => ({
      id: r.id,
      thumbnail: r.thumbnail,
      name: r.name,
      category: r.category,
      contentCid: r.contentCID,
      // status: r.status,
    }));
  } else {
    return (orbiterReleases.value || [])
    // .filter(r => r.release.release.status !== 'deleted')
    .map(r => ({
      id: r.release.id,
      thumbnail: r.release.release.thumbnail,
      name: r.release.release.contentName,
      category: r.release.release.category,
      contentCid: r.release.release.file,
      // status: r.release.release.status as ItemStatus,
    }));
  }
});

const editedRelease: Ref<{
  id: string;
  name: string;
  category: string;
  contentCid: string;
  // status: ItemStatus;
}> = ref({
  id: '',
  name: '',
  category: '',
  contentCid: '',
  // status: 'pending',
});
const editReleaseDialog = ref(false);
const confirmDeleteReleaseDialog = ref(false);

const isSavingLoading = ref(false);
isSavingLoading.value = true;


async function saveRelease() {
  if (status.value === 'static') {
    const targetReleaseIndex = staticReleases.value.findIndex(
      r => r.id == editedRelease.value.id,
    );
    if (targetReleaseIndex !== -1) {
      staticReleases.value.splice(targetReleaseIndex, 1, {
        ...staticReleases.value[targetReleaseIndex],
        ...editedRelease.value,
      });
    }
    editReleaseDialog.value = false;
  } else {
    try {
      await orbiter.editRelease({
        releaseId: editedRelease.value.id,
        release: {
          contentName: editedRelease.value.name,
          category: editedRelease.value.category,
          file: editedRelease.value.contentCid,
          // status: editedRelease.value.status,
        },
      });
      editReleaseDialog.value = false;
    } catch (error) {
      console.log('error on saveRelease', error);
    } finally {
      isSavingLoading.value = false;
    }
  }
}

function deleteRelease(id: string) {
  editedRelease.value = {
    id,
    name: '',
    category: '',
    contentCid: '',
    // status: 'deleted',
  };
  confirmDeleteReleaseDialog.value = true;
}

async function confirmDeleteRelease() {
  if (status.value === 'static') {
    const targetReleaseIndex = staticReleases.value.findIndex(
      r => r.id == editedRelease.value.id,
    );
    if (targetReleaseIndex !== -1) {
      staticReleases.value.splice(targetReleaseIndex, 1, {
        ...staticReleases.value[targetReleaseIndex],
        ...editedRelease.value,
      });
    }
    confirmDeleteReleaseDialog.value = false;
  } else {
    try {
      await orbiter.editRelease({
        releaseId: editedRelease.value.id,
        release: {
          contentName: editedRelease.value.name,
          category: editedRelease.value.category,
          file: editedRelease.value.contentCid,
          // status: editedRelease.value.status,
        },
      });
      editReleaseDialog.value = false;
    } catch (error) {
      console.log('error on confirmDeleteRelease', error);
    } finally {
      isSavingLoading.value = false;
    }
  }

}

function editRelease(id: string) {
  if (status.value === 'static') {
    const targetRelease =  staticReleases.value.find(r => r.id == id);
    if (targetRelease) {
      editedRelease.value = {
        id: targetRelease.id,
        name: targetRelease.name,
        category: targetRelease.category,
        contentCid: targetRelease.contentCID,
        // status: targetRelease.status,
      };
    }
  } else {
    const targetRelease = orbiterReleases.value?.find((r => r.release.id == id));
    if (targetRelease) {
      editedRelease.value = {
        id: targetRelease.release.id,
        name: targetRelease.release.release.contentName,
        category: targetRelease.release.release.category,
        contentCid: targetRelease.release.release.file,
        // status: targetRelease.release.release.status as ItemStatus,
      };
    }
  }
  editReleaseDialog.value = true;
}
</script>
