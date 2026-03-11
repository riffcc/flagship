<template>
  <v-container
    fill-height
    class="d-flex flex-column ga-4 my-10"
  >
    <h1>Credits & Licensing</h1>
    <p class="text-body-1">
      Riff.CC is built with open source software and uses assets from generous contributors.
      This page acknowledges their work.
    </p>

    <!-- Asset Credits -->
    <section class="mt-6">
      <h2>Assets</h2>
      <v-list bg-color="transparent" class="mt-2">
        <v-list-item
          v-for="(asset, index) in assetCredits"
          :key="index"
          class="px-0"
        >
          <template #prepend>
            <v-icon color="primary" class="mr-3">$image</v-icon>
          </template>
          <v-list-item-title class="font-weight-medium">
            {{ asset.name }}
          </v-list-item-title>
          <v-list-item-subtitle>
            {{ asset.description }}
          </v-list-item-subtitle>
          <div class="text-caption mt-1">
            By
            <a
              v-if="asset.authorUrl"
              :href="asset.authorUrl"
              target="_blank"
              rel="noopener"
              class="text-primary"
            >{{ asset.author }}</a>
            <span v-else>{{ asset.author }}</span>
            &bull;
            <a
              :href="asset.licenseUrl"
              target="_blank"
              rel="noopener"
              class="text-primary"
            >{{ asset.license }}</a>
            &bull;
            <a
              :href="asset.sourceUrl"
              target="_blank"
              rel="noopener"
              class="text-primary"
            >Source</a>
          </div>
        </v-list-item>
      </v-list>
    </section>

    <!-- Software Credits -->
    <section class="mt-6">
      <h2>Open Source Software</h2>
      <p class="text-body-2 text-grey mt-2">
        Flagship is built on the shoulders of these open source projects.
      </p>
      <v-list bg-color="transparent" class="mt-2">
        <v-list-item
          v-for="(software, index) in softwareCredits"
          :key="index"
          class="px-0"
        >
          <template #prepend>
            <v-icon color="primary" class="mr-3">$code-tags</v-icon>
          </template>
          <v-list-item-title class="font-weight-medium">
            <a
              :href="software.projectUrl"
              target="_blank"
              rel="noopener"
              class="text-primary"
            >{{ software.name }}</a>
          </v-list-item-title>
          <v-list-item-subtitle>
            {{ software.description }}
          </v-list-item-subtitle>
          <div class="text-caption mt-1">
            License: {{ software.license }}
          </div>
        </v-list-item>
      </v-list>
    </section>

    <!-- Content Licensing -->
    <section class="mt-6">
      <h2>Content Licensing</h2>
      <p class="text-body-2 mt-2">
        {{ contentLicenses.description }}
      </p>
      <v-list bg-color="transparent" class="mt-2">
        <v-list-item
          v-for="(license, index) in contentLicenses.licenses"
          :key="index"
          class="px-0"
        >
          <template #prepend>
            <v-icon color="primary" class="mr-3">$scale-balance</v-icon>
          </template>
          <v-list-item-title class="font-weight-medium">
            <a
              :href="license.url"
              target="_blank"
              rel="noopener"
              class="text-primary"
            >{{ license.name }}</a>
          </v-list-item-title>
          <v-list-item-subtitle>
            {{ license.description }}
          </v-list-item-subtitle>
        </v-list-item>
      </v-list>
    </section>
  </v-container>
</template>

<script setup lang="ts">
import { assetCredits, softwareCredits, contentLicenses } from '../constants/credits';
</script>
