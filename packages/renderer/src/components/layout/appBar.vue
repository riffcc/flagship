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
    <div class="d-none d-sm-flex flex-1-0 align-center">
      <router-link
        to="/"
        class="nav-link"
      >
        Home
      </router-link>
      <router-link
        v-for="item in featuredContentCategories"
        :key="item.categoryId"
        :to="getCategoryRoute(item.categoryId)"
        class="nav-link"
      >
        {{ item.displayName === 'TV Shows' ? 'TV' : item.displayName }}
      </router-link>
      <router-link
        to="/books"
        class="nav-link"
      >
        Books
      </router-link>
    </div>
    <v-spacer></v-spacer>
    <div class="search-container d-none d-sm-flex mx-2" style="max-width: 400px; width: 100%;">
      <SearchBar />
    </div>
    <v-bottom-sheet
      inset
      close-on-content-click
    >
      <template #activator="{ props }">
        <v-btn
          v-bind="props"
          icon="$menu"
          class="d-sm-none"
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
          :title="item.displayName === 'TV Shows' ? 'TV' : item.displayName"
          active-class="text-primary-lighten-1"
          :active="route.path === item.categoryId"
          @click="router.push(getCategoryRoute(item.categoryId))"
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
    <account-menu v-if="userData"></account-menu>
  </v-app-bar>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useUserSession } from '/@/composables/userSession';
import { useAccountStatusQuery, useContentCategoriesQuery } from '/@/plugins/lensService/hooks';
import accountMenu from '/@/components/account/accountMenu.vue';
import SearchBar from '/@/components/search/SearchBar.vue';

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

// Map category slugs to clean routes
const categoryRouteMap: Record<string, string> = {
  'music': '/music',
  'movies': '/movies',
  'tv-shows': '/tv',
  'books': '/books',
  'audiobooks': '/audiobooks',
  'games': '/games',
};

const getCategoryRoute = (categoryId: string) => {
  return categoryRouteMap[categoryId] || `/featured/${categoryId}`;
};

</script>

<style scoped>
/* Beautiful styling ported from gamepadNavBar */
:deep(.v-toolbar) {
  background: rgba(0, 0, 0, 0.95) !important;
  backdrop-filter: blur(10px);
  border-bottom: none !important;
}

/* Animated breathing glow divider */
:deep(.v-toolbar)::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(138, 43, 226, 0.3) 20%,
    rgba(138, 43, 226, 0.6) 50%,
    rgba(138, 43, 226, 0.3) 80%,
    transparent
  );
  box-shadow:
    0 0 10px rgba(138, 43, 226, 0.5),
    0 0 20px rgba(138, 43, 226, 0.3),
    0 0 30px rgba(138, 43, 226, 0.1);
  animation: breathe 4s ease-in-out infinite;
}

@keyframes breathe {
  0%, 100% {
    opacity: 0.6;
    box-shadow:
      0 0 5px rgba(138, 43, 226, 0.3),
      0 0 10px rgba(138, 43, 226, 0.2);
  }
  50% {
    opacity: 1;
    box-shadow:
      0 0 15px rgba(138, 43, 226, 0.6),
      0 0 30px rgba(138, 43, 226, 0.4),
      0 0 45px rgba(138, 43, 226, 0.2);
  }
}

/* Nav link styling with active underline */
.nav-link {
  position: relative;
  color: rgba(255, 255, 255, 0.7);
  text-decoration: none;
  font-size: 16px;
  font-weight: 500;
  padding: 12px 16px;
  transition: all 0.3s ease;
}

.nav-link:hover {
  color: rgba(255, 255, 255, 0.95);
}

.nav-link::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 16px;
  right: 16px;
  height: 2px;
  background: #8a2be2;
  box-shadow: 0 0 10px rgba(138, 43, 226, 0.8);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.nav-link.router-link-active {
  color: #fff;
}

.nav-link.router-link-active::after {
  opacity: 1;
}

/* Logo hover effect */
:deep(.v-app-bar-title a) {
  transition: filter 0.3s ease;
}

:deep(.v-app-bar-title a:hover) {
  filter: brightness(1.2);
}
</style>
