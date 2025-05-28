<template>
  <v-container>
    <v-card>
      <v-card-title>Check Export File</v-card-title>
      <v-card-text>
        <v-file-input
          v-model="file"
          label="Select export JSON file"
          accept=".json"
          @change="analyzeFile"
        />
        
        <div v-if="analysis">
          <h3>Analysis Results:</h3>
          <p>Total releases: {{ analysis.totalReleases }}</p>
          <p>Releases with contentCID: {{ analysis.withCID }}</p>
          <p>Releases without contentCID: {{ analysis.withoutCID }}</p>
          
          <h4>Releases missing contentCID:</h4>
          <v-list density="compact">
            <v-list-item 
              v-for="release in analysis.missingCID" 
              :key="release.id"
            >
              <v-list-item-title>{{ release.name || 'Unnamed' }}</v-list-item-title>
              <v-list-item-subtitle>
                ID: {{ release.id }}
                <br>Category: {{ release.categoryId }}
                <br>Has Thumbnail: {{ !!release.thumbnailCID }}
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
          
          <h4>Releases with contentCID:</h4>
          <v-list density="compact">
            <v-list-item 
              v-for="release in analysis.hasCID" 
              :key="release.id"
            >
              <v-list-item-title>{{ release.name || 'Unnamed' }}</v-list-item-title>
              <v-list-item-subtitle>
                CID: {{ release.contentCID }}
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </div>
      </v-card-text>
    </v-card>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const file = ref<File[]>([]);
const analysis = ref<{
  totalReleases: number;
  withCID: number;
  withoutCID: number;
  missingCID: any[];
  hasCID: any[];
} | null>(null);

async function analyzeFile() {
  if (!file.value || file.value.length === 0) return;
  
  const fileContent = await file.value[0].text();
  const data = JSON.parse(fileContent);
  
  console.log('Export file data:', data);
  
  const releases = data.releases || [];
  const missingCID: any[] = [];
  const hasCID: any[] = [];
  
  releases.forEach((release: any) => {
    if (!release.contentCID) {
      missingCID.push({
        id: release.id,
        name: release.name,
        categoryId: release.categoryId,
        thumbnailCID: release.thumbnailCID,
      });
    } else {
      hasCID.push({
        id: release.id,
        name: release.name,
        contentCID: release.contentCID,
      });
    }
  });
  
  analysis.value = {
    totalReleases: releases.length,
    withCID: hasCID.length,
    withoutCID: missingCID.length,
    missingCID,
    hasCID,
  };
  
  console.log('Analysis complete:', analysis.value);
}</script>