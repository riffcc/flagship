<template>
  <v-container
    fill-height
    class="d-flex flex-column ga-6 my-10"
  >
    <h1>Privacy Policy</h1>
    <p>{{ privacyPolicy.subtitle }}</p>
    <v-img :src="privacyPolicy.image"></v-img>
    <p
      v-for="(paragraph, index) in privacyPolicy.policy"
      :key="index"
    >
      {{ paragraph }}
    </p>
  </v-container>
</template>

<script setup lang="ts">
import { privacyPolicy } from '../constants/privacy-policty';

</script>
