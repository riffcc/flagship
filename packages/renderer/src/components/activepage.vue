<template>
  <v-container class="pa-4">
    <div class="d-flex justify-center">
      <v-switch v-model="model" color="primary" :label="model ? 'Hide network connections' : 'View network connections'"
        hide-details inset />
    </div>
    <div v-if="model" class="d-flex justify-center align-center fill-height">
      <v-list>
        <userlistitem v-for="(user) in users" :key="user.infoMembre.idCompte" :userid="user.infoMembre.idCompte"
          :lastActive="user.vuÀ"></userlistitem>
      </v-list>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import userlistitem from './userlistitem.vue';
import { suivre as follow, obt } from '@constl/vue';
import { useConstellation } from '../utils';

const model = ref(false);
const { constl } = useConstellation()
const users = follow(constl.réseau.suivreConnexionsMembres)

</script>