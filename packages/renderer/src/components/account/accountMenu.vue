<template>
  <v-menu>
    <template #activator="{props: activatorProps}">
      <v-avatar
        v-bind="activatorProps"
        :image="userAvatar"
        border
        class="mr-2 d-none d-sm-block"
      />
    </template>
    <v-sheet
      border
      width="192px"
      class="mt-2"
    >
      <div class="px-4 py-2">
        <h4>{{ userData?.name }}</h4>
        <p class="text-caption mt-1">
          {{ userData?.email }}
        </p>
      </div>
      <v-divider></v-divider>
      <v-list>
        <v-list-item
          v-for="menuItem in menuItems"
          :key="menuItem.label"
          :title="menuItem.label"
          @click="menuItem.onClick"
        />
      </v-list>
    </v-sheet>
  </v-menu>
</template>

<script setup lang="ts">
import { ref, onScopeDispose } from 'vue';
import {useUserProfilePhoto} from '/@/components/users/utils';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import { useRouter } from 'vue-router';
import { useUserSession } from '/@/composables/userSession';

const {orbiter} = useOrbiter();

// User avatar
const accountId = ref<string | undefined>();

if (orbiter && typeof orbiter.listenForAccountId === 'function') {
  // Prefer a listening mechanism if available
  const unsubscribe = orbiter.listenForAccountId({
    f: (newAccountId: string | undefined) => {
      accountId.value = newAccountId;
    },
  });
  if (typeof unsubscribe === 'function') {
    onScopeDispose(unsubscribe);
  }
} else if (orbiter && typeof orbiter.getAccountId === 'function') {
  // Fallback to a one-time fetch if listenForAccountId is not available
  orbiter.getAccountId()
    .then(id => { accountId.value = id; })
    .catch(err => console.error("Failed to get account ID for account menu:", err));
}

const userAvatar = useUserProfilePhoto(accountId);

const router = useRouter();


const { userData } = useUserSession();
const menuItems = [
  { label: 'Account', onClick: () => router.push('/account')},
  { label: 'Settings', onClick: () => router.push('/account/settings')},
  { label: 'Disconnect', onClick: () => { userData.value = null; }},
];
</script>
