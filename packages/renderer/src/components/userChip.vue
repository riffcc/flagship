<template>
  <v-chip
    class="mx-2"
    label
  >
    <slot>Uploaded by:</slot> {{ displayName }}
  </v-chip>
</template>

<script setup lang="ts">
import {computed} from 'vue';

import {suivre as follow} from '@constl/vue';
import {useOrbiter} from '/@/plugins/flagship/utils';
import {selectTranslation} from '/@/utils';

const {orbiter} = useOrbiter();

const props = defineProps<{accountId: string}>();

const names = follow(orbiter.listenForNameChange.bind(orbiter), {accountId: computed(() => props.accountId)});

const displayName = computed(() => {
  return selectTranslation(names.value) || 'Anonymous';
});
</script>
