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

    <div class="d-none d-md-flex flex-1-0">
      <v-btn
        v-for="item in navigationMap.appBar.normal"
        :key="item.label"
        :text="item.label"
        :class="$router.currentRoute.value.path === item.path ? 'text-primary-lighten-1 text-none' : 'text-none'"
        @click="() => $router.push(`${item.path}`)"
      ></v-btn>
      <v-divider
        vertical
        class="mx-4"
      ></v-divider>
      <v-btn
        v-for="item in navigationMap.appBar.authenticated"
        :key="item.label"
        :text="item.label"
        :class="$router.currentRoute.value.path === item.path ? 'text-primary-lighten-1 text-none' : 'text-none'"
        @click="() => $router.push(`${item.path}`)"
      ></v-btn>
    </div>
  </v-app-bar>
</template>

<script setup lang="ts">
import { navigationMap } from '/@/constants/navigation';

</script>
