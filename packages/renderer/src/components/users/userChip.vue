<template>
  <v-chip :prepend-avatar="userAvatar">
    {{ displayName }}
  </v-chip>
</template>
<script setup lang="ts">
import {computed} from 'vue';
import {selectTranslation} from '/@/utils';

import {suivre as follow} from '@constl/vue';
import {useUserProfilePhoto} from './utils';
import {useOrbiter} from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();

const props = defineProps<{accountId?: string}>();

// User name
const names = follow(orbiter.listenForNameChange.bind(orbiter), {accountId: computed(() => props.accountId)});

const displayName = computed(() => {
  return selectTranslation(names.value) || 'Anonymous';
});

// User avatar
const userAvatar = useUserProfilePhoto(props.accountId);
</script>
