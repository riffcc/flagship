<template>
  <v-dialog
    :model-value="dialogOpen"
    max-width="512px"
  >
    <v-card class="pa-2">
      <v-card-title>{{ message }}</v-card-title>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          color="blue-darken-1"
          variant="text"
          @click="onClose"
        >
          Cancel
        </v-btn>
        <v-btn
          color="blue-darken-1"
          variant="text"
          @click="onConfirm"
        >
          OK
        </v-btn>
        <v-spacer></v-spacer>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
defineProps<{
  message: string;
  dialogOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
}>();
</script>
