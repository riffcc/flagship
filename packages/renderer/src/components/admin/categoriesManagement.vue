<template>
  <v-container>
    <v-sheet
      class="px-6 py-4 mx-auto"
      max-width="448px"
    >
      <v-list-item
        class="px-0"
      >
        <template #append>
          <v-dialog
            v-model="createCategoryDialog"
            width="auto"
          >
            <template #activator="{props: activatorProps}">
              <v-btn
                icon="mdi-plus-circle"
                variant="text"
                density="comfortable"
                size="small"
                v-bind="activatorProps"
              ></v-btn>
            </template>
            <v-sheet
              width="480px"
              max-height="620px"
              class="pa-8 ma-auto"
            >
              <content-category-form
                @update:error="handleError"
                @update:success="handleSuccess"
              />
            </v-sheet>
          </v-dialog>
        </template>
        <h3>Edit Categories</h3>
      </v-list-item>
      <v-divider class="mt-2"></v-divider>
      <div v-if="contentCategories">
        <v-list-item
          v-for="category in contentCategories"
          :key="category.id"
          :title="category.contentCategory.displayName"
          lines="two"
        >
          <template #append>
            <v-btn
              icon="mdi-pencil"
              variant="text"
              density="comfortable"
              size="x-small"
              @click="() => editCategory(category.id)"
            ></v-btn>
            <v-btn
              icon="mdi-delete"
              variant="text"
              density="comfortable"
              size="x-small"
              @click="() => deleteCategory(category.id)"
            ></v-btn>
          </template>
        </v-list-item>
      </div>
    </v-sheet>
  </v-container>
  <v-dialog
    v-model="editCategoryDialog"
    max-width="500px"
  >
    <v-card class="py-3">
      <v-card-title>
        <span class="text-h6 ml-2">Edit Category</span>
      </v-card-title>

      <v-card-text>
        <content-category-form
          :initial-data="editedContentCategory"
          mode="edit"
          @update:error="handleError"
          @update:success="handleSuccess"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          color="blue-darken-1"
          variant="text"
          @click="editCategoryDialog = false"
        >
          Cancel
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
  <confirmation-dialog
    message="Are you sure you want to delete this category?"
    :dialog-open="confirmDeleteCategoryDialog"
    @close="() => {confirmDeleteCategoryDialog = false}"
    @confirm="confirmDeleteCategory"
  ></confirmation-dialog>
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
import { ref } from 'vue';
import { useOrbiter } from '/@/plugins/orbiter/utils';
import { suivre as follow } from '@constl/vue';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import type { types as orbiterTypes } from '@riffcc/orbiter';
import ContentCategoryForm from '/@/components/releases/contentCategoryForm.vue';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';

const createCategoryDialog = ref(false);
const editCategoryDialog = ref(false);
const confirmDeleteCategoryDialog = ref(false);

const editedContentCategory = ref<orbiterTypes.ContentCategoryWithId<orbiterTypes.ContentCategoryMetadataField>>({
  id: '',
  contentCategory: {
    categoryId: '',
    displayName: '',
    metadataSchema: {},
  },
});
const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();
const {orbiter} = useOrbiter();
const contentCategories = follow(orbiter.listenForContentCategories.bind(orbiter));

function editCategory (id?: string) {
  if (!id) return;
  const targetItem = contentCategories.value?.find(item => item.id === id);
  if (targetItem) {
    editedContentCategory.value = {
      id: targetItem.id,
      contentCategory: {
        ...targetItem.contentCategory,
        metadataSchema: JSON.parse(targetItem.contentCategory.metadataSchema),
      },
    };
  }
  editCategoryDialog.value = true;
};
function handleSuccess(message: string) {
  openSnackbar(message, 'success');
  editCategoryDialog.value = false;
}

function handleError(message: string) {
  openSnackbar(message, 'error');
  console.error('Error:', message);
}

function deleteCategory(id?: string) {
  if (!id) return;
  editedContentCategory.value = {
    id,
    contentCategory: {
      categoryId: '',
      displayName: '',
      metadataSchema: {},
    },
  };
  confirmDeleteCategoryDialog.value = true;
}

async function confirmDeleteCategory() {
  try {
      await orbiter.removeCategory(editedContentCategory.value.id);
    } catch (error) {
      console.error('Error deleting release:', error);
    }
  confirmDeleteCategoryDialog.value = false;
}
</script>
