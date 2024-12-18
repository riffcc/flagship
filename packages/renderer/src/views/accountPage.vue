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
    <p>
      Account id: {{ accountId }}
    </p>
    <p>
      Device id: {{ deviceId }}
    </p>
    <p>
      Account status: {{ accountStatus }} ({{ statusExplanation }})
    </p>
  </v-container>
</template>
<script setup lang="ts">
import {computed, ref, watchEffect} from 'vue';
import {selectTranslation} from '/@/utils';

import {suivre as follow, obt} from '@constl/vue';
import {useUserProfilePhoto} from '/@/components/users/utils';
import {useDevStatus} from '/@/composables/devStatus';
import {useOrbiter} from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();

// User name
const names = follow(orbiter.listenForNameChange);

const displayName = computed(() => {
  return selectTranslation(names.value) || 'Anonymous';
});

// Account id
const accountId = follow(orbiter.constellation.suivreIdCompte);
const deviceId = obt(orbiter.constellation.obtIdDispositif);

// User avatar
const userAvatar = useUserProfilePhoto(accountId.value);

// Account status
const moderator = follow(orbiter.listenToIsModerator);
const canUpload = follow(orbiter.followCanUpload);
const accountStatus = computed(()=>{
  return moderator.value || (canUpload.value ? 'MEMBER' : 'GUEST');
});
const statusExplanation = computed(()=>{
  switch (accountStatus.value) {
    case 'ADMIN':
      return 'Can moderate content and invite other moderators or administrators.';
    case 'MODERATOR':
      return 'Can moderate content.';
    case 'MEMBER':
      return 'Can add content.';
    case 'GUEST':
      return 'View-only access to site.';
    default:
      return 'Unknown role';
  }
});

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
