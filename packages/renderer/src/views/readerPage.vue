<template>
  <EpubReader
    v-if="book"
    :book-id="id"
    :content-cid="book.contentCID"
    :title="book.name"
  />
  <v-sheet
    v-else
    color="transparent"
    class="d-flex w-100 fill-height align-center justify-center"
  >
    <v-progress-circular
      indeterminate
      color="primary"
      size="64"
    ></v-progress-circular>
  </v-sheet>
</template>

<script setup lang="ts">
import { useGetReleaseQuery } from '/@/plugins/lensService/hooks';
import EpubReader from '/@/components/reader/EpubReader.vue';

const props = defineProps<{
  id: string;
}>();

// Fetch the book data
const { data: book } = useGetReleaseQuery(props.id);
</script>
