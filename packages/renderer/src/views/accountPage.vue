<template>
  <v-container>
    <v-sheet
      class="d-flex position-relative py-4 px-2 px-md-12"
      max-height="160"
    >
      <v-img
        :src="userData?.avatar"
        max-width="120"
      />
      <p class="ml-4 my-auto">
        Anonymous
      </p>
    </v-sheet>

    <v-card class="mt-4 text-center py-1">
      <v-card-title>
        Account info
      </v-card-title>
      <v-divider></v-divider>
      <v-skeleton-loader
        v-if="isLoading"
        max-width="90%"
        class="mx-auto"
        type="list-item-two-line@4"
      ></v-skeleton-loader>
      <v-list
        v-else
        lines="two"
      >
        <v-list-item
          v-if="publicKey"
          title="Public Key"
          :subtitle="isCopied(publicKey) ? 'Copied!' : publicKey"
          @click="copy(publicKey, publicKey)"
        >
        </v-list-item>
        <v-list-item
          v-if="peerId"
          title="Peer ID"
          :subtitle="isCopied(peerId) ? 'Copied!' : peerId"
          @click="copy(peerId, peerId)"
        >
        </v-list-item>
        <v-list-item
          v-if="accountStatus"
          title="Role"
        >
          <v-list-item-subtitle class="mt-1">
            <v-chip
              v-if="accountStatus.isAdmin"
              color="primary"
              class="font-weight-bold"
              size="small"
            >
              ADMINISTRATOR
            </v-chip>
            <v-chip
              v-for="role in accountStatus.roles"
              v-else
              :key="role"
              color="secondary"
              class="text-uppercase"
              size="small"
            >
              {{ role }}
            </v-chip>
          </v-list-item-subtitle>
        </v-list-item>
        <v-list-item
          v-if="accountStatus && accountStatus.permissions.length > 0"
          title="Key Permissions"
        >
          <v-list-item-subtitle class="text-wrap">
            {{ formattedPermissions }}
          </v-list-item-subtitle>
        </v-list-item>
      </v-list>
    </v-card>
  </v-container>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import { useUserSession } from '/@/composables/userSession';
import { useAccountStatusQuery, usePeerIdQuery, usePublicKeyQuery } from '/@/plugins/lensService/hooks';
import { useCopyToClipboard } from '../composables/copyToClipboard';
import { useLensInitialization } from '../composables/lensInitialization';

const { userData } = useUserSession();
const { copy, isCopied } = useCopyToClipboard();
const { isLensReady } = useLensInitialization();

const { data: publicKey, isLoading: publicKeyIsLoading } = usePublicKeyQuery({
  enabled: isLensReady,
});
const { data: peerId, isLoading: peerIdIsLoading } = usePeerIdQuery({
  enabled: isLensReady,
});
const { data: accountStatus, isLoading: accountStatusIsLoading } = useAccountStatusQuery({
  enabled: isLensReady,
});

const isLoading = computed(() => {
  return !publicKey.value || publicKeyIsLoading.value || !peerId.value || peerIdIsLoading.value || !accountStatus.value || accountStatusIsLoading.value;
});

const PERMISSION_LABELS: Record<string, string> = {
  // Release Permissions
  'release:create': 'Create Releases',
  'release:edit:own': 'Edit Own Releases',
  'release:edit:any': 'Edit Any Release',
  'release:delete': 'Delete Releases',

  // Content Management Permissions
  'featured:manage': 'Manage Featured Content',
  'category:manage': 'Manage Categories',
  'blocklist:manage': 'Manage Blocked Content',

  // Site Management Permissions
  'subscription:manage': 'Manage Site Subscriptions',

  // RBAC System Permissions (often implicit for admins, but good to have)
  'system:manage:admins': 'Manage Administrators',
  'system:manage:roles': 'Manage Roles',
  'system:manage:assignments': 'Manage User Roles',
};

function capitalize(str: string): string {
  if (!str) return '';
  return str.charAt(0).toUpperCase() + str.slice(1);
}

const formattedPermissions = computed(() => {
  if (!accountStatus.value) {
    return 'Loading permissions...';
  }

  if (accountStatus.value.isAdmin) {
    return 'Full control over all site content and user management.';
  }

  if (accountStatus.value.permissions.length === 0) {
    return 'View-only access to site content.';
  }

  const permissionTexts = accountStatus.value.permissions.map(p => {
    if (PERMISSION_LABELS[p]) {
      return PERMISSION_LABELS[p];
    }

    const parts = p.split(':');
    return parts.map(capitalize).join(' ');
  });

  return permissionTexts.join(', ');
});
</script>
