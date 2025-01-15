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
          icon="mdi-menu"
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
          v-for="item in contentCategories.filter((cc: ContentCategory) => cc.featured)"
          :key="item.id"
          :title="pluralize(startCase(item.id))"
          active-class="text-primary-lighten-1"
          :active="route.path === item.id"
          @click="router.push(`/featured/${item.id}`)"
        ></v-list-item>
        <template v-if="userData">
          <v-divider class="my-1"></v-divider>
          <v-list-item
            title="Upload"
            active-class="text-primary-lighten-1"
            :active="route.path === '/upload'"
            @click="router.push('/upload')"
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
          <v-list-item
            v-if="isAdmin"
            title="Admin"
            active-class="text-primary-lighten-1"
            :active="route.path === '/admin'"
            @click="router.push('/admin')"
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
        v-for="item in contentCategories.filter((cc: ContentCategory) => cc.featured)"
        :key="item.id"
        :to="`/featured/${item.id}`"
        class="text-decoration-none mx-2 text-subtitle-1 text-white"
        active-class="text-primary-lighten-1"
      >
        {{ pluralize(startCase(item.id)) }}
      </router-link>

      <template v-if="userData">
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
          v-if="isAdmin"
          to="Admin"
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
import { useRoute, useRouter } from 'vue-router';
import { suivre as follow } from '@constl/vue';
import {startCase} from 'lodash';
import pluralize from 'pluralize-esm';
import { useOrbiter } from '/@/plugins/orbiter/utils';
import { type ContentCategory, useSiteSettings } from '/@/composables/siteSettings';
import { useUserSession } from '/@/composables/userSession';
import accountMenu from '/@/components/account/accountMenu.vue';

const {orbiter} = useOrbiter();
const router = useRouter();
const route = useRoute();
const isAdmin = follow(orbiter.listenToIsModerator);
const { contentCategories } = useSiteSettings();
const { userData } = useUserSession();

function handleOnDisconnect(){
  userData.value = null;
};
</script>
