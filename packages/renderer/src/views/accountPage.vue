<template>
  <v-container class="text-center">
    <v-img
      :src="userAvatar"
      class="mx-auto"
      width="150"
    />
    {{ displayName }}
    <v-switch
      v-model="staticModeSwitch"
      :label="`Static mode ${staticModeSwitch ? 'on' : 'off'}`"
      :color="staticModeSwitch ? 'primary' : 'secondary'"
    />
  </v-container>
</template>
<script setup lang="ts">
import {computed, ref, watchEffect} from 'vue';
import {selectTranslation} from '/@/utils';

import {suivre as follow} from '@constl/vue';
import {useUserProfilePhoto} from '/@/components/users/utils';
import {useDevStatus} from '/@/composables/devStatus';
import {useOrbiter} from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();

const props = defineProps<{accountId?: string}>();

// User name
const names = follow(orbiter.listenForNameChange, {accountId: computed(() => props.accountId)});

const displayName = computed(() => {
  return selectTranslation(names.value) || 'Anonymous';
});

// User avatar
const userAvatar = useUserProfilePhoto(props.accountId);

// Dev static mode
const {status} = useDevStatus();
const staticModeSwitch = ref(status.value === 'static');
watchEffect(() => {
  status.value = staticModeSwitch.value ? 'static' : 'live';
});
watchEffect(() => {
  staticModeSwitch.value = status.value === 'static';
});
</script>
