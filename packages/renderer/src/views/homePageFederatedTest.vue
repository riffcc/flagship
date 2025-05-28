<template>
  <v-container
    fluid
    class="pa-4"
  >
    <v-row>
      <v-col cols="12">
        <h1 class="text-h4 mb-4">Federation Index Test</h1>
        
        <!-- Debug Info -->
        <v-card class="mb-4">
          <v-card-title>Federation Index Status</v-card-title>
          <v-card-text>
            <p>Federation Index Enabled: {{ hasFederationIndex }}</p>
            <p>Total Entries: {{ stats?.totalEntries || 0 }}</p>
            <p>Entries by Site: {{ entriesBySiteCount }}</p>
            <p>Entries by Type: {{ entriesByTypeCount }}</p>
            
            <v-btn
              color="primary"
              class="mt-2"
              @click="populateTestData"
            >
              Populate Test Data
            </v-btn>
            <v-btn
              color="secondary"
              class="mt-2 ml-2"
              @click="refreshData"
            >
              Refresh
            </v-btn>
          </v-card-text>
        </v-card>

        <!-- Featured Content from Federation Index -->
        <v-card
          v-if="featuredData.length > 0"
          class="mb-4"
        >
          <v-card-title>Featured Content (Federation Index)</v-card-title>
          <v-card-text>
            <v-row>
              <v-col
                v-for="item in featuredData"
                :key="item.id"
                cols="12"
                md="4"
              >
                <v-card>
                  <v-img 
                    v-if="item.thumbnailCid"
                    :src="gatewayUrl(item.thumbnailCid)"
                    height="200"
                    cover
                  />
                  <v-card-title>{{ item.title }}</v-card-title>
                  <v-card-subtitle>
                    From: {{ item.sourceSiteName }}<br>
                    Type: {{ item.contentType }}<br>
                    CID: {{ item.contentCid?.substring(0, 20) }}...
                  </v-card-subtitle>
                  <v-card-actions>
                    <v-btn 
                      :href="gatewayUrl(item.contentCid)"
                      target="_blank"
                      color="primary"
                      text
                    >
                      View Content
                    </v-btn>
                  </v-card-actions>
                </v-card>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>

        <!-- All Entries -->
        <v-card>
          <v-card-title>All Federation Index Entries</v-card-title>
          <v-card-text>
            <v-data-table
              :headers="tableHeaders"
              :items="allEntries"
              :items-per-page="10"
            >
              <template #item.title="{ item }">
                <span>{{ item.title }}</span>
              </template>
              <template #item.sourceSiteName="{ item }">
                <v-chip small>{{ item.sourceSiteName }}</v-chip>
              </template>
              <template #item.contentType="{ item }">
                <v-chip
                  small
                  color="primary"
                >
                  {{ item.contentType }}
                </v-chip>
              </template>
              <template #item.actions="{ item }">
                <v-btn 
                  :href="gatewayUrl(item.contentCid)"
                  target="_blank"
                  icon
                  small
                >
                  <v-icon>mdi-open-in-new</v-icon>
                </v-btn>
              </template>
            </v-data-table>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useLensService } from '../plugins/lensService/hooks';
import { useQuery } from '@tanstack/vue-query';
import {
  RELEASE_NAME_PROPERTY,
  RELEASE_CONTENT_CID_PROPERTY,
  RELEASE_THUMBNAIL_CID_PROPERTY,
  RELEASE_CATEGORY_ID_PROPERTY,
  RELEASE_METADATA_PROPERTY,
} from '@riffcc/lens-sdk';

const { lensService } = useLensService();

// Check if federation index exists
const hasFederationIndex = ref(false);

// Stats
const stats = ref<any>(null);

const entriesBySiteCount = computed(() => {
  if (!stats.value?.entriesBySite) return 0;
  return Array.from(stats.value.entriesBySite.values()).reduce((a, b) => a + b, 0);
});

const entriesByTypeCount = computed(() => {
  if (!stats.value?.entriesByType) return 0;
  return Array.from(stats.value.entriesByType.entries())
    .map(([type, count]) => `${type}: ${count}`)
    .join(', ');
});

// Featured data
const { data: featuredData = [], refetch: refetchFeatured } = useQuery({
  queryKey: ['federationIndex', 'featured'],
  queryFn: async () => {
    try {
      return await lensService.getFederationIndexFeatured(20);
    } catch (error) {
      console.error('Failed to get featured:', error);
      return [];
    }
  },
  enabled: true,
});

// All entries
const allEntries = ref<any[]>([]);

const tableHeaders = [
  { text: 'Title', value: 'title' },
  { text: 'Source Site', value: 'sourceSiteName' },
  { text: 'Type', value: 'contentType' },
  { text: 'Category', value: 'categoryId' },
  { text: 'Actions', value: 'actions', sortable: false },
];

// Gateway URL helper
const gatewayUrl = (cid: string) => {
  return `https://gateway.pinata.cloud/ipfs/${cid}`;
};

// Populate test data
const populateTestData = async () => {
  try {
    console.log('Populating test data...');
    
    // Get site info
    const siteId = await lensService.getSiteId();
    const siteMetadata = await lensService.getSiteMetadata();
    const siteName = siteMetadata.name || 'Test Site';
    
    // Get existing releases
    const releases = await lensService.getReleases();
    console.log(`Found ${releases.length} releases to add to federation index`);
    
    if (releases.length === 0) {
      alert('No releases found. Please add some content first.');
      return;
    }
    
    // Get the federation index from the site program
    const { siteProgram } = lensService as any;
    if (!siteProgram?.federationIndex) {
      alert('Federation index not available');
      return;
    }
    
    // Add each release to the federation index as test data
    let added = 0;
    for (const release of releases) {
      try {
        const metadata = release[RELEASE_METADATA_PROPERTY] ? 
          (typeof release[RELEASE_METADATA_PROPERTY] === 'string' ? 
            JSON.parse(release[RELEASE_METADATA_PROPERTY]) : 
            release[RELEASE_METADATA_PROPERTY]) : {};
        
        const indexEntry = {
          contentCid: release[RELEASE_CONTENT_CID_PROPERTY],
          title: release[RELEASE_NAME_PROPERTY] || 'Untitled',
          sourceSiteId: siteId,
          sourceSiteName: siteName,
          contentType: metadata.contentType || 'video',
          categoryId: release[RELEASE_CATEGORY_ID_PROPERTY] || 'uncategorized',
          timestamp: Date.now(),
          description: metadata.description,
          thumbnailCid: release[RELEASE_THUMBNAIL_CID_PROPERTY],
          tags: metadata.tags || [],
        };
        
        await siteProgram.federationIndex.insertContent(indexEntry);
        added++;
      } catch (error) {
        console.error('Failed to add release to federation index:', error);
      }
    }
    
    alert(`Added ${added} entries to federation index`);
    await refreshData();
  } catch (error) {
    console.error('Failed to populate test data:', error);
    alert('Failed to populate test data: ' + error.message);
  }
};

// Refresh data
const refreshData = async () => {
  try {
    // Check federation index
    const { siteProgram } = lensService as any;
    hasFederationIndex.value = !!siteProgram?.federationIndex;
    
    if (hasFederationIndex.value) {
      // Get stats
      stats.value = await lensService.getFederationIndexStats();
      
      // Get all entries
      const entries = await siteProgram.federationIndex.getRecent(100);
      allEntries.value = entries;
    }
    
    // Refetch featured
    await refetchFeatured();
  } catch (error) {
    console.error('Failed to refresh data:', error);
  }
};

onMounted(() => {
  refreshData();
});
</script>