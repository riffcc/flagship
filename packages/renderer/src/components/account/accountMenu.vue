<template>
  <v-menu open-on-hover>
    <template #activator="{props: activatorProps}">
      <v-avatar
        v-bind="activatorProps"
        :image="userAvatar"
      />
    </template>
    <v-list>
      <v-list-item
        title="Manage account"
        prepend-icon="mdi-account"
      />
      <v-list-item
        title="Site info"
        prepend-icon="mdi-cog"
      />
      <v-list-item
        title="About Riff.CC"
        prepend-icon="mdi-information-outline"
      />
    </v-list>
  </v-menu>
</template>

<script setup lang="ts">
import {suivre as follow} from '@constl/vue';
import {useUserProfilePhoto} from '/@/components/users/utils';
import {useOrbiter} from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();

// User avatar
const accountId = follow(orbiter.listenForAccountId);

const userAvatar = useUserProfilePhoto(accountId);
</script>
