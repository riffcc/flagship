<template>
  <v-container>
    <h2 class="text-h5 mb-6">Debug: Persistent Releases</h2>
    
    <v-card class="mb-6">
      <v-card-title>Current Subscriptions</v-card-title>
      <v-card-text>
        <div v-if="subscriptionsLoading">Loading subscriptions...</div>
        <div v-else-if="subscriptions && subscriptions.length > 0">
          <p class="mb-4">Found {{ subscriptions.length }} active subscriptions:</p>
          <v-list>
            <v-list-item
              v-for="sub in subscriptions"
              :key="sub.id"
            >
              <v-list-item-title>{{ sub.name }}</v-list-item-title>
              <v-list-item-subtitle>Site ID: {{ sub.siteId }}</v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </div>
        <div v-else>
          <v-alert type="info">No active subscriptions found</v-alert>
        </div>
      </v-card-text>
    </v-card>
    
    <v-card class="mb-6">
      <v-card-title>Current Releases</v-card-title>
      <v-card-text>
        <div v-if="releasesLoading">Loading releases...</div>
        <div v-else-if="releases && releases.length > 0">
          <p class="mb-4">Found {{ releases.length }} releases:</p>
          <v-list>
            <v-list-item
              v-for="release in releases"
              :key="release.id"
            >
              <v-list-item-title>{{ release.name }}</v-list-item-title>
              <v-list-item-subtitle>
                <div>ID: {{ release.id }}</div>
                <div>CID: {{ release.contentCID || 'NO CID' }}</div>
                <div v-if="(release as any).federatedFrom">
                  <strong>Federated From:</strong> {{ (release as any).federatedFrom }}
                </div>
                <div v-if="(release as any).author">
                  <strong>Author:</strong> {{ (release as any).author }}
                </div>
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </div>
        <div v-else>
          <v-alert type="info">No releases found</v-alert>
        </div>
      </v-card-text>
    </v-card>
    
    <v-card>
      <v-card-title>Actions</v-card-title>
      <v-card-text>
        <v-btn 
          color="primary" 
          :loading="refreshing"
          @click="refreshData"
        >
          Refresh Data
        </v-btn>
        
        <v-btn 
          color="error" 
          class="ml-4"
          :loading="deleting"
          @click="deleteAll"
        >
          Delete All Releases
        </v-btn>
        
        <v-btn 
          color="error" 
          variant="outlined"
          class="ml-4"
          :loading="clearing"
          @click="clearDatabase"
        >
          Clear Database & Index
        </v-btn>
      </v-card-text>
    </v-card>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { 
  useGetReleasesQuery, 
  useGetSubscriptionsQuery,
  useDeleteReleaseMutation,
  useClearAllReleasesMutation,
} from '/@/plugins/lensService/hooks';

const refreshing = ref(false);
const deleting = ref(false);
const clearing = ref(false);

// Queries
const { 
  data: releases, 
  isLoading: releasesLoading,
  refetch: refetchReleases, 
} = useGetReleasesQuery();

const { 
  data: subscriptions, 
  isLoading: subscriptionsLoading,
  refetch: refetchSubscriptions, 
} = useGetSubscriptionsQuery();

const deleteReleaseMutation = useDeleteReleaseMutation({
  onSuccess: () => {
    console.log('Release deleted successfully');
  },
  onError: (e) => {
    console.error('Failed to delete release:', e);
  },
});

const clearAllReleasesMutation = useClearAllReleasesMutation({
  onSuccess: () => {
    console.log('Database cleared successfully');
  },
  onError: (e) => {
    console.error('Failed to clear database:', e);
  },
});

const refreshData = async () => {
  refreshing.value = true;
  try {
    await Promise.all([
      refetchReleases(),
      refetchSubscriptions(),
    ]);
  } finally {
    refreshing.value = false;
  }
};

const deleteAll = async () => {
  if (!releases.value) return;
  
  deleting.value = true;
  try {
    console.log('[DEBUG] Starting to delete all releases...');
    
    for (const release of releases.value) {
      console.log('[DEBUG] Deleting release:', {
        id: release.id,
        name: release.name,
        contentCID: release.contentCID,
        federatedFrom: (release as any).federatedFrom,
        author: (release as any).author?.toString(),
      });
      
      const result = await deleteReleaseMutation.mutateAsync({ id: release.id });
      console.log('[DEBUG] Delete result:', result);
    }
    
    console.log('[DEBUG] All releases deleted, waiting 3 seconds...');
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    console.log('[DEBUG] Refreshing data to see if releases came back...');
    await refreshData();
    
  } finally {
    deleting.value = false;
  }
};

const clearDatabase = async () => {
  clearing.value = true;
  try {
    console.log('[DEBUG] Starting to clear entire database...');
    
    const result = await clearAllReleasesMutation.mutateAsync();
    console.log('[DEBUG] Clear database result:', result);
    
    console.log('[DEBUG] Database cleared, waiting 3 seconds...');
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    console.log('[DEBUG] Refreshing data to see if database is empty...');
    await refreshData();
    
  } finally {
    clearing.value = false;
  }
};
</script>