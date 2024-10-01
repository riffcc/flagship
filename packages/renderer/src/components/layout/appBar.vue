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
          src="src/assets/logo.svg"
        ></v-img>
      </router-link>
    </v-app-bar-title>

    <div class="d-none d-md-flex flex-1-0">
      <v-btn
        v-for="route in routes"
        :key="route.label"
        :text="route.label"
        :class="$router.currentRoute.value.path === route.path ? 'text-primary-lighten-1 text-none' : 'text-none'"
        @click="() => $router.push(`${route.path}`)"
      ></v-btn>
      <v-divider
        vertical
        class="mx-4"
      ></v-divider>
      <v-btn
        v-for="route in authenticatedRoutes"
        :key="route.label"
        :text="route.label"
        :class="$router.currentRoute.value.path === route.path ? 'text-primary-lighten-1 text-none' : 'text-none'"
        @click="() => $router.push(`${route.path}`)"
      ></v-btn>
    </div>
    <!-- <template #append>
      <AccountMenu />
      <SettingsDialog />
    </template> -->
  </v-app-bar>
</template>

<script setup lang="ts">
// import AccountMenu from '/@/components/account/accountMenu.vue';
// import SettingsDialog from '/@/components/settings/settingsDialog.vue';

type Route = {
  label: string;
  path: string
}

const routes: Array<Route> = [
  { label: 'Home', path: '/' },
  { label: 'Music', path: '/music' },
  { label: 'Movies', path: '/movies' },
  { label: 'TV Shows', path: '/tv-shows' },
];

const authenticatedRoutes: Array<Route> = [
  { label: 'Upload', path: '/upload' },
  { label: 'Admin', path: '/admin' },
];
</script>
