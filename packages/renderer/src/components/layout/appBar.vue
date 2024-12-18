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

    <div class="d-none d-md-flex flex-1-0 align-center">
      <router-link
        v-for="item in navigationMap.appBar.normal"
        :key="item.label"
        :to="item.path"
        class="text-decoration-none mx-2 text-subtitle-1 text-white"
        active-class="text-primary-lighten-1"
      >
        {{ item.label }}
      </router-link>
      <v-divider
        vertical
        class="mx-4"
      ></v-divider>
      <router-link
        v-for="item in navigationMap.appBar.authenticated"
        :key="item.label"
        :to="item.path"
        class="text-decoration-none mx-2 text-subtitle-1 text-white"
        active-class="text-primary-lighten-1"
      >
        {{ item.label }}
      </router-link>
      <div v-if="isAdmin">
        <router-link
          v-for="item in navigationMap.appBar.admin"
          :key="item.label"
          :to="item.path"
          class="text-decoration-none mx-2 text-subtitle-1 text-white"
          active-class="text-primary-lighten-1"
        >
          {{ item.label }}
        </router-link>
      </div>
    </div>
  </v-app-bar>
</template>

<script setup lang="ts">
import { suivre as follow } from '@constl/vue';
import {navigationMap} from '/@/constants/navigation';
import { useOrbiter } from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();


const isAdmin = follow(orbiter.listenToIsModerator);
</script>
