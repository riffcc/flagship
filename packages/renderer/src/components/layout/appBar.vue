<template>
  <v-app-bar>
    <template #prepend>
      <v-app-bar-nav-icon v-show="false"></v-app-bar-nav-icon>
    </template>
    <v-app-bar-title>
      <router-link to="/">
        <v-img
          cover
          max-width="48px"
          aspect-ratio="1"
          src="/logo.svg"
        ></v-img>
      </router-link>
    </v-app-bar-title>
    <v-bottom-sheet
      inset
      close-on-content-click
    >
      <template #activator="{ props }">
        <v-btn
          v-bind="props"
          icon="$menu"
          class="d-sm-none mr-2"
        ></v-btn>
      </template>
      <v-list>
        <v-list-item
          title="Home"
          active-class="text-primary-lighten-1"
          :active="route.path === '/'"
          @click="router.push('/')"
        ></v-list-item>
        <v-list-item
          v-for="item in featuredContentCategories"
          :key="item.id"
          :title="item.displayName"
          active-class="text-primary-lighten-1"
          :active="route.path === item.categoryId"
          @click="router.push(`/featured/${item.categoryId}`)"
        ></v-list-item>
        <template v-if="userData">
          <v-divider class="my-1"></v-divider>
          <v-list-item
            v-if="canUpload"
            title="Upload"
            active-class="text-primary-lighten-1"
            :active="route.path === '/upload'"
            @click="router.push('/upload')"
          ></v-list-item>
          <v-list-item
            v-if="canAccessAdminPanel"
            title="Admin"
            active-class="text-primary-lighten-1"
            :active="route.path === '/admin'"
            @click="router.push('/admin')"
          ></v-list-item>
          <v-divider class="my-1"></v-divider>
          <v-list-item
            title="Account"
            active-class="text-primary-lighten-1"
            :active="route.path === '/account'"
            @click="router.push('/account')"
          ></v-list-item>
          <v-list-item
            title="Settings"
            active-class="text-primary-lighten-1"
            :active="route.path === '/settings'"
            @click="router.push('/settings')"
          ></v-list-item>
          <v-list-item
            title="Disconnect"
            active-class="text-primary-lighten-1"
            @click="handleOnDisconnect"
          ></v-list-item>
        </template>
      </v-list>
    </v-bottom-sheet>
    <div class="d-none d-sm-flex flex-1-0 align-center">
      <router-link
        to="/"
        class="text-decoration-none mx-2 text-subtitle-1 text-white"
        active-class="text-primary-lighten-1"
      >
        Home
      </router-link>
      <router-link
        v-for="item in featuredContentCategories"
        :key="item.categoryId"
        :to="`/featured/${item.categoryId}`"
        class="text-decoration-none mx-2 text-subtitle-1 text-white"
        active-class="text-primary-lighten-1"
      >
        {{ item.displayName }}
      </router-link>

      <template v-if="canUpload || canAccessAdminPanel">
        <v-divider
          vertical
          class="mx-4"
        ></v-divider>
        <router-link
          to="/upload"
          class="text-decoration-none mx-2 text-subtitle-1 text-white"
          active-class="text-primary-lighten-1"
        >
          Upload
        </router-link>
        <router-link
          v-if="canAccessAdminPanel"
          to="/admin"
          class="text-decoration-none mx-2 text-subtitle-1 text-white"
          active-class="text-primary-lighten-1"
        >
          Admin
        </router-link>
      </template>
    </div>
    <account-menu v-if="userData"></account-menu>
  </v-app-bar>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useUserSession } from '/@/composables/userSession';
import { useAccountStatusQuery, useContentCategoriesQuery } from '/@/plugins/lensService/hooks';
import accountMenu from '/@/components/account/accountMenu.vue';

const router = useRouter();
const route = useRoute();

const { data: contentCategories } = useContentCategoriesQuery();
const featuredContentCategories = computed(() => contentCategories.value?.filter(c => c.featured));

const { data: accountStatus } = useAccountStatusQuery();

const canUpload = computed(() =>
  accountStatus.value?.permissions.includes('release:create') ?? false,
);

const canAccessAdminPanel = computed(() => {
  // Always guard against the initial undefined state.
  if (!accountStatus.value) {
    return false;
  }

  // Check for the direct isAdmin flag first.
  if (accountStatus.value.isAdmin) {
    return true;
  }

  // FIX: Use .includes() or .some() to check for the presence of a role.
  // .includes() is cleaner if you only need to check for one role.
  if (accountStatus.value.roles.includes('moderator')) {
    return true;
  }

  // If none of the above, deny access.
  return false;
});
const { userData } = useUserSession();

function handleOnDisconnect() {
  userData.value = null;
};


</script>
