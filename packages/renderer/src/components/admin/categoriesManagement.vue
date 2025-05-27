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
                icon="$plus-circle"
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
          :title="category.displayName"
          lines="two"
        >
          <template #prepend>
            <v-sheet width="24">
              <v-icon
                v-if="category.featured"
                :icon="'$star'"
                variant="text"
                color="yellow"
                size="small"
                class="mr-1 mb-1"
              ></v-icon>
            </v-sheet>
          </template>
          <template #append>
            <v-btn
              icon="$pencil"
              variant="text"
              density="comfortable"
              size="x-small"
              @click="() => editCategory(category.id)"
            ></v-btn>
            <v-btn
              icon="$delete"
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
import { useSnackbarMessage } from '/@/composables/snackbarMessage';

import ContentCategoryForm from '/@/components/releases/contentCategoryForm.vue';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import type { ContentCategoryData, ContentCategoryMetadata } from '@riffcc/lens-sdk';
import { useContentCategoriesQuery } from '../../plugins/lensService/hooks';


const { data: contentCategories } = useContentCategoriesQuery();

const createCategoryDialog = ref(false);
const editCategoryDialog = ref(false);
const confirmDeleteCategoryDialog = ref(false);

const editedContentCategory = ref<ContentCategoryData<ContentCategoryMetadata>>({
  id: '',
  displayName: '',
  featured: false,
  metadataSchema: {},
});
const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();



function editCategory (id?: string) {
  if (!id) return;
  const targetItem = contentCategories.value?.find(item => item.id === id);
  if (targetItem) {
    editedContentCategory.value = targetItem;
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
    displayName: '',
    featured: false,
    metadataSchema: {},
  };
  confirmDeleteCategoryDialog.value = true;
}

async function confirmDeleteCategory() {
  try {
      // await orbiter.removeCategory(editedContentCategory.value.id);
    } catch (error) {
      console.error('Error deleting release:', error);
    }
  confirmDeleteCategoryDialog.value = false;
}
</script>
