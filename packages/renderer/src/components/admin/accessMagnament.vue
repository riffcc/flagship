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
          <h6 class="text-h6 font-weight-bold mb-4">New Admin</h6>
          <v-form @submit="handleOnSubmit">
            <v-text-field
              v-model="newAdmin.id"
              label="ID"
              :rules="rules"
            ></v-text-field>
            <v-switch
              v-model="newAdmin.super"
              color="primary"
              label="Super"
            ></v-switch>
            <v-btn
              color="primary"
              type="submit"
              text="Add"
              block
            >
            </v-btn>
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
          <h6 class="text-h6 font-weight-bold mb-4">Admins</h6>
          <v-list v-if="adminList.length > 0">
            <v-list-item
              v-for="admin, i in adminList"
              :key="i"
              class="px-0"
              :title="admin.id"
            >
              <template #prepend>
                <v-icon
                  :icon="admin.super ? 'mdi-account-supervisor' : 'mdi-account'"
                  size="small"
                ></v-icon>
              </template>
              <template #append>
                <v-btn
                  icon="mdi-delete"
                  size="small"
                  @click="confirmDeleteAdminDialog = true"
                >
                </v-btn>
              </template>
              <confirmation-dialog
                message="Are you sure you want to delete this admin?"
                :dialog-open="confirmDeleteAdminDialog"
                @close="confirmDeleteAdminDialog = false"
                @confirm="() => confirmDeleteAdmin(admin.id)"
              ></confirmation-dialog>
            </v-list-item>
          </v-list>
          <div
            v-else
            class="d-flex h-75"
          >
            <span class="ma-auto text-body-2 text-medium-emphasis">No Admins found.</span>
          </div>
        </v-sheet>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { type Ref, ref } from 'vue';
import type { SubmitEventPromise } from 'vuetify';
import { isValidEmail } from '/@/utils';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
type Admin = {
  id: string;
  super: boolean;
}

const adminList: Ref<Admin[]> = ref([
  { id: 'admin@test.com', super: true},
]);

const newAdmin: Ref<Admin> = ref({
  id: '',
  super: false,
});

const rules = [
  (v: string) => Boolean(v) || 'Required field.',
  (v: string) => isValidEmail(v) || 'Please enter a valid email.',
  (v: string) => !adminList.value.map(a => a.id).includes(v) || 'Email already registered as Admin.',
];

function handleOnSubmit(e: SubmitEventPromise){
  e.preventDefault();
  e.then(result => {
    if (result.valid) {
      adminList.value.push(newAdmin.value);
      newAdmin.value = {
        id: '',
        super: false,
      };
    }
  });
}
const confirmDeleteAdminDialog = ref(false);

function confirmDeleteAdmin(id: string){
  adminList.value = adminList.value.filter(a => a.id !== id);
  confirmDeleteAdminDialog.value = false;
};
</script>
