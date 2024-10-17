<template>
  <v-app>
    <app-bar />
    <v-main min-height="100vh">
      <router-view />
    </v-main>
    <app-footer />
  </v-app>
</template>

<script setup lang="ts">
import { onKeyStroke } from '@vueuse/core';
import { ref, watchEffect } from 'vue';

import appBar from '/@/components/layout/appBar.vue';
import appFooter from '/@/components/layout/appFooter.vue';

import {useShowDefederation} from '/@/composables/showDefed';

const {showDefederation} = useShowDefederation();

const MAGIC_KEY = 'magicmagic';

const yetToType = ref(MAGIC_KEY);
onKeyStroke((e)=>{
  if (!yetToType.value.length) return;
  if (e.key === yetToType.value[0]) {
    yetToType.value = yetToType.value.slice(1);
  } else {
    yetToType.value = MAGIC_KEY;
  }
});
watchEffect(()=>{
  if (!yetToType.value.length) showDefederation.value = true;
});

</script>
