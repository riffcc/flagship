<template>
  <v-container>
    <v-row>
      <v-col cols="12">
        <h1>Hybrid Network Service Test</h1>
        <p>This component tests the new UnifiedNetworkService with both Peerbit and Citadel support.</p>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12" md="6">
        <v-card title="Network Configuration">
          <v-card-text>
            <pre>{{ networkConfig }}</pre>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="6">
        <v-card title="Network Status">
          <v-card-text>
            <v-list density="compact">
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :color="isInitialized ? 'success' : 'warning'">
                    {{ isInitialized ? '$check-circle' : '$alert-circle' }}
                  </v-icon>
                </template>
                <v-list-item-title>Service Initialized: {{ isInitialized }}</v-list-item-title>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :color="isLoading ? 'info' : 'grey'">
                    {{ isLoading ? '$loading' : '$circle' }}
                  </v-icon>
                </template>
                <v-list-item-title>Loading: {{ isLoading }}</v-list-item-title>
              </v-list-item>

              <v-list-item v-if="error">
                <template v-slot:prepend>
                  <v-icon color="error">$alert</v-icon>
                </template>
                <v-list-item-title>Error: {{ error.message }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12">
        <v-card title="Network Health Check">
          <v-card-text>
            <v-btn
              @click="checkHealth"
              :loading="checkingHealth"
              color="primary"
              class="mb-4"
            >
              Check Network Health
            </v-btn>

            <v-data-table
              v-if="healthResults"
              :items="healthResults"
              :headers="healthHeaders"
              density="compact"
            ></v-data-table>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12">
        <v-card title="Data Fetching Test">
          <v-card-text>
            <v-btn
              @click="fetchReleases"
              :loading="fetchingReleases"
              color="primary"
              class="mb-4"
            >
              Fetch Releases (Hybrid)
            </v-btn>

            <v-btn
              @click="fetchRelease('sample-release-1')"
              :loading="fetchingRelease"
              color="secondary"
              class="mb-4 ml-2"
            >
              Fetch Specific Release
            </v-btn>

            <v-alert v-if="fetchError" type="error" class="mb-4">
              {{ fetchError }}
            </v-alert>

            <v-data-table
              v-if="releases.length > 0"
              :items="releases"
              :headers="releaseHeaders"
              density="compact"
            ></v-data-table>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12">
        <v-card title="Network Telemetry">
          <v-card-text>
            <v-btn
              @click="getTelemetry"
              :loading="fetchingTelemetry"
              color="info"
              class="mb-4"
            >
              Get Telemetry Data
            </v-btn>

            <v-data-table
              v-if="telemetryStats"
              :items="telemetryStats.byAdapter"
              :headers="telemetryHeaders"
              density="compact"
            ></v-data-table>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted } from 'vue';
import { useNetworkService } from '@/composables/useNetworkService';

const {
  networkService,
  isInitialized,
  isLoading,
  error,
  getReleases,
  getRelease,
  getNetworkStats,
  checkHealth
} = useNetworkService();

const networkConfig = ref<any>(null);
const healthResults = ref<any[]>([]);
const releases = ref<any[]>([]);
const telemetryStats = ref<any>(null);
const fetchError = ref<string | null>(null);

const checkingHealth = ref(false);
const fetchingReleases = ref(false);
const fetchingRelease = ref(false);
const fetchingTelemetry = ref(false);

const healthHeaders = [
  { title: 'Adapter', key: 'adapter' },
  { title: 'Healthy', key: 'isHealthy' },
  { title: 'Response Time (ms)', key: 'responseTime' },
  { title: 'Error', key: 'error' }
];

const releaseHeaders = [
  { title: 'ID', key: 'id' },
  { title: 'Title', key: 'title' },
  { title: 'Category', key: 'categorySlug' },
  { title: 'Source', key: 'source' }
];

const telemetryHeaders = [
  { title: 'Adapter', key: 'adapter' },
  { title: 'Total Requests', key: 'total' },
  { title: 'Success Rate', key: 'successRate' },
  { title: 'Avg Duration (ms)', key: 'avgDuration' }
];

onMounted(() => {
  if (networkService) {
    networkConfig.value = networkService.getConfig();
  }
});

async function checkHealth() {
  try {
    checkingHealth.value = true;
    const health = await networkService.checkHealth();

    healthResults.value = Object.entries(health).map(([adapter, data]) => ({
      adapter,
      isHealthy: data.isHealthy,
      responseTime: data.responseTime,
      error: data.error ? data.error.message : ''
    }));
  } catch (err) {
    console.error('Health check failed:', err);
    fetchError.value = err instanceof Error ? err.message : String(err);
  } finally {
    checkingHealth.value = false;
  }
}

async function fetchReleases() {
  try {
    fetchingReleases.value = true;
    fetchError.value = null;

    const result = await getReleases();
    releases.value = result.map(release => ({
      ...release,
      source: 'hybrid'
    }));
  } catch (err) {
    console.error('Failed to fetch releases:', err);
    fetchError.value = err instanceof Error ? err.message : String(err);
  } finally {
    fetchingReleases.value = false;
  }
}

async function fetchRelease(id: string) {
  try {
    fetchingRelease.value = true;
    fetchError.value = null;

    const release = await getRelease(id);
    releases.value = [{
      ...release,
      source: 'hybrid'
    }];
  } catch (err) {
    console.error(`Failed to fetch release ${id}:`, err);
    fetchError.value = err instanceof Error ? err.message : String(err);
  } finally {
    fetchingRelease.value = false;
  }
}

async function getTelemetry() {
  try {
    fetchingTelemetry.value = true;
    const telemetry = networkService.getTelemetry();
    const stats = telemetry.getStats();

    telemetryStats.value = stats;
  } catch (err) {
    console.error('Failed to get telemetry:', err);
    fetchError.value = err instanceof Error ? err.message : String(err);
  } finally {
    fetchingTelemetry.value = false;
  }
}
</script>

<style scoped>
pre {
  background: #f5f5f5;
  padding: 12px;
  border-radius: 4px;
  overflow-x: auto;
}
</style>
