<template>
  <div class="d-flex align-center justify-space-between w-100 mt-12 mb-4">
    <p class="text-h6 text-sm-h5 font-weight-bold">{{ title }}</p>
    <div v-if="pagination">
      <v-btn
        icon="$menu-left"
        density="compact"
        variant="text"
        @click="onPrevious"
      ></v-btn>
      <v-btn
        icon="$menu-right"
        density="compact"
        variant="text"
        @click="onNext"
      ></v-btn>
    </div>
    <v-btn
      v-else-if="onNavigate"
      variant="text"
      slim
      density="comfortable"
      text="View All"
      class="text-none"
      @click="onNavigate"
    ></v-btn>
  </div>
  <v-row dense>
    <slot></slot>
  </v-row>
</template>

<script setup lang="ts">
defineProps<{
  title: string;
  pagination?: boolean;
  onNavigate?: () => void;
  onPrevious?: () => void;
  onNext?: () => void;
}>();
</script>