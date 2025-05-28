<template>
  <v-container>
    <v-card class="mb-4">
      <v-card-title>Federation Index Statistics</v-card-title>
      <v-card-text>
        <v-btn 
          color="primary" 
          :loading="isLoading"
          class="mb-4"
          @click="refreshStats"
        >
          Refresh Stats
        </v-btn>

        <div v-if="stats">
          <h3>Overview</h3>
          <p>Total Entries: {{ stats.totalEntries }}</p>
          
          <h3>Entries by Site</h3>
          <v-list density="compact">
            <v-list-item 
              v-for="[siteId, count] in Array.from(stats.entriesBySite.entries())" 
              :key="siteId"
            >
              <v-list-item-title>{{ siteId.slice(0, 16) }}...</v-list-item-title>
              <v-list-item-subtitle>{{ count }} entries</v-list-item-subtitle>
            </v-list-item>
          </v-list>
          
          <h3>Entries by Type</h3>
          <v-list density="compact">
            <v-list-item 
              v-for="[type, count] in Array.from(stats.entriesByType.entries())" 
              :key="type"
            >
              <v-list-item-title>{{ type }}</v-list-item-title>
              <v-list-item-subtitle>{{ count }} entries</v-list-item-subtitle>
            </v-list-item>
          </v-list>
          
          <div v-if="stats.oldestEntry">
            <h3>Oldest Entry</h3>
            <p>Title: {{ stats.oldestEntry.title }}</p>
            <p>Date: {{ new Date(stats.oldestEntry.timestamp).toLocaleString() }}</p>
          </div>
          
          <div v-if="stats.newestEntry">
            <h3>Newest Entry</h3>
            <p>Title: {{ stats.newestEntry.title }}</p>
            <p>Date: {{ new Date(stats.newestEntry.timestamp).toLocaleString() }}</p>
          </div>
        </div>
        
        <v-alert
          v-else-if="!isLoading"
          type="info"
        >
          No statistics available. Click "Refresh Stats" to load.
        </v-alert>
      </v-card-text>
    </v-card>

    <v-card>
      <v-card-title>Recent Entries</v-card-title>
      <v-card-text>
        <v-btn 
          color="secondary" 
          :loading="isLoadingRecent"
          class="mb-4"
          @click="loadRecentEntries"
        >
          Load Recent Entries
        </v-btn>

        <v-list
          v-if="recentEntries.length > 0"
          density="compact"
        >
          <v-list-item 
            v-for="entry in recentEntries" 
            :key="entry.id"
            class="mb-2"
          >
            <v-list-item-title>{{ entry.title }}</v-list-item-title>
            <v-list-item-subtitle>
              <div>Site: {{ entry.sourceSiteName }}</div>
              <div>Type: {{ entry.contentType }}</div>
              <div>Category: {{ entry.categoryId }}</div>
              <div>CID: {{ entry.contentCid }}</div>
              <div>Date: {{ new Date(entry.timestamp).toLocaleString() }}</div>
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>

        <v-alert
          v-else-if="!isLoadingRecent"
          type="info"
        >
          No recent entries found. Click "Load Recent Entries" to check.
        </v-alert>
      </v-card-text>
    </v-card>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useLensService } from '/@/plugins/lensService';
import type { IndexableFederationEntry } from '@riffcc/lens-sdk';

const { lensService } = useLensService();

const stats = ref<{
  totalEntries: number;
  entriesBySite: Map<string, number>;
  entriesByType: Map<string, number>;
  oldestEntry?: IndexableFederationEntry;
  newestEntry?: IndexableFederationEntry;
} | null>(null);

const recentEntries = ref<IndexableFederationEntry[]>([]);
const isLoading = ref(false);
const isLoadingRecent = ref(false);

async function refreshStats() {
  isLoading.value = true;
  try {
    const result = await lensService.getFederationIndexStats();
    stats.value = result;
    console.log('[FederationStats] Loaded stats:', result);
  } catch (error) {
    console.error('[FederationStats] Failed to load stats:', error);
  } finally {
    isLoading.value = false;
  }
}

async function loadRecentEntries() {
  isLoadingRecent.value = true;
  try {
    const entries = await lensService.getFederationIndexRecent(20);
    recentEntries.value = entries;
    console.log('[FederationStats] Loaded recent entries:', entries.length);
  } catch (error) {
    console.error('[FederationStats] Failed to load recent entries:', error);
  } finally {
    isLoadingRecent.value = false;
  }
}
</script>