<template>
  <v-container>
    <v-sheet class="px-6 py-4 mx-auto" max-width="1200px">
      <!-- Header -->
      <v-list-item class="px-0">
        <template #append>
          <v-dialog v-model="createStructureDialog" width="auto">
            <template #activator="{props: activatorProps}">
              <v-btn
                icon="$plus-circle"
                variant="text"
                density="comfortable"
                size="small"
                v-bind="activatorProps"
              ></v-btn>
            </template>
            <v-card width="600px" max-height="620px" class="pa-8 ma-auto">
              <structure-form
                :initial-type="currentStructureType"
                :parent-context="parentContext"
                @update:error="handleError"
                @update:success="handleSuccess"
              />
            </v-card>
          </v-dialog>
        </template>
        <h3>Manage Structures</h3>
      </v-list-item>
      <v-divider class="mt-2 mb-4"></v-divider>
      
      <!-- Breadcrumb Navigation -->
      <v-breadcrumbs v-if="breadcrumbs.length > 0" :items="breadcrumbs" class="pa-0 mb-4">
        <template #divider>
          <v-icon icon="$chevron-right"></v-icon>
        </template>
        <template #item="{ item }">
          <v-btn
            variant="text"
            size="small"
            @click="navigateToBreadcrumb(item)"
          >
            {{ item.title }}
          </v-btn>
        </template>
      </v-breadcrumbs>
      
      <!-- Loading state -->
      <v-progress-linear
        v-if="isLoading"
        indeterminate
        color="primary"
        class="mt-4"
      ></v-progress-linear>
      
      <!-- Main Categories View -->
      <div v-else-if="currentView === 'categories'">
        <v-row>
          <!-- TV Shows Category -->
          <v-col cols="12" md="6">
            <v-card @click="navigateToCategory('tv')">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$television" class="mr-2"></v-icon>
                TV Shows
                <v-spacer></v-spacer>
                <v-chip>{{ tvSeriesCount }}</v-chip>
              </v-card-title>
              <v-card-text>
                Manage TV series, seasons, and episodes
              </v-card-text>
            </v-card>
          </v-col>
          
          <!-- Music Category -->
          <v-col cols="12" md="6">
            <v-card @click="navigateToCategory('music')">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$music" class="mr-2"></v-icon>
                Music
                <v-spacer></v-spacer>
                <v-chip>{{ artistCount }}</v-chip>
              </v-card-title>
              <v-card-text>
                Manage artists, albums, and musical releases
              </v-card-text>
            </v-card>
          </v-col>
          
          <!-- Tags Category -->
          <v-col cols="12" md="6">
            <v-card @click="navigateToCategory('tags')">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$tag" class="mr-2"></v-icon>
                Tags
                <v-spacer></v-spacer>
                <v-chip>{{ tagCount }}</v-chip>
              </v-card-title>
              <v-card-text>
                Manage content tags and categories
              </v-card-text>
            </v-card>
          </v-col>
          
          <!-- Collections Category -->
          <v-col cols="12" md="6">
            <v-card @click="navigateToCategory('collections')">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$folder" class="mr-2"></v-icon>
                Collections
                <v-spacer></v-spacer>
                <v-chip>{{ collectionCount }}</v-chip>
              </v-card-title>
              <v-card-text>
                Manage custom collections and playlists
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </div>
      
      <!-- TV Shows List View -->
      <div v-else-if="currentView === 'tv-list'">
        <div class="d-flex align-center mb-4">
          <h4>TV Shows</h4>
          <v-spacer></v-spacer>
          <v-btn
            variant="outlined"
            @click="showAllEpisodes"
          >
            All Episodes
          </v-btn>
        </div>
        
        <v-list v-if="tvSeries.length > 0" lines="two">
          <v-list-item
            v-for="series in tvSeries"
            :key="series.id"
            @click="navigateToSeries(series)"
          >
            <template #prepend>
              <v-avatar>
                <v-img
                  v-if="series.thumbnailCID"
                  :src="parseUrlOrCid(series.thumbnailCID)"
                  cover
                ></v-img>
                <v-icon v-else icon="$television"></v-icon>
              </v-avatar>
            </template>
            <v-list-item-title>{{ series.name }}</v-list-item-title>
            <v-list-item-subtitle>
              {{ formatSeasonCount(getSeriesSeasonCount(series.id)) }} • 
              {{ formatEpisodeCount(getSeriesEpisodeCount(series.id)) }}
            </v-list-item-subtitle>
            <template #append>
              <v-btn
                icon="$pencil"
                variant="text"
                density="comfortable"
                size="small"
                @click.stop="editStructure(series)"
              ></v-btn>
              <v-btn
                icon="$delete"
                variant="text"
                density="comfortable"
                size="small"
                @click.stop="deleteStructure(series)"
              ></v-btn>
            </template>
          </v-list-item>
        </v-list>
        <v-alert v-else type="info" variant="tonal">
          No TV series yet. Series will appear automatically when you upload TV episodes with series information.
        </v-alert>
      </div>
      
      <!-- Series Detail View (Seasons) -->
      <div v-else-if="currentView === 'series-detail'">
        <div class="d-flex align-center mb-4">
          <h4>{{ currentSeries?.name }}</h4>
          <v-spacer></v-spacer>
        </div>
        
        <!-- Seasons List -->
        <h5 class="mb-2">Seasons</h5>
        <template v-if="currentSeasons.length > 0">
          <v-list lines="two">
            <v-list-item
              v-for="season in currentSeasons"
              :key="season.id"
              @click="navigateToSeason(season)"
            >
              <template #prepend>
                <v-avatar>
                  <v-img
                    v-if="season.thumbnailCID"
                    :src="parseUrlOrCid(season.thumbnailCID)"
                    cover
                  ></v-img>
                  <v-icon v-else icon="$numeric"></v-icon>
                </v-avatar>
              </template>
              <v-list-item-title>
                {{ season.name || `Season ${season.metadata?.seasonNumber}` }}
              </v-list-item-title>
              <v-list-item-subtitle>
                {{ getSeasonEpisodeCount(season.id, currentSeries?.id, season.metadata?.seasonNumber) }} Episodes
              </v-list-item-subtitle>
              <template #append>
                <v-btn
                  icon="$pencil"
                  variant="text"
                  density="comfortable"
                  size="small"
                  @click.stop="editStructure(season)"
                ></v-btn>
                <v-btn
                  icon="$delete"
                  variant="text"
                  density="comfortable"
                  size="small"
                  @click.stop="deleteStructure(season)"
                ></v-btn>
              </template>
            </v-list-item>
          </v-list>
        </template>
        <template v-else>
          <v-alert type="info" variant="tonal" class="mb-4">
            No seasons found. Seasons will be automatically created when you add or edit episodes with season numbers.
          </v-alert>
        </template>
        
        <!-- Episodes without seasons -->
        <template v-if="orphanEpisodes.length > 0">
          <h5 class="mt-4 mb-2">Uncategorized Episodes</h5>
          <v-list lines="two">
            <v-list-item
              v-for="episode in orphanEpisodes"
              :key="episode.id"
              @click="navigateToRelease(episode.id)"
            >
              <template #prepend>
                <v-avatar>
                  <v-img
                    v-if="episode.thumbnailCID"
                    :src="parseUrlOrCid(episode.thumbnailCID)"
                    cover
                  ></v-img>
                  <v-icon v-else icon="$play"></v-icon>
                </v-avatar>
              </template>
              <v-list-item-title>{{ episode.name }}</v-list-item-title>
              <v-list-item-subtitle>
                Episode {{ episode.metadata?.episodeNumber }}
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </template>
      </div>
      
      <!-- Season Detail View (Episodes) -->
      <div v-else-if="currentView === 'season-detail'">
        <div class="d-flex align-center mb-4">
          <h4>{{ currentSeason?.name || `Season ${currentSeason?.metadata?.seasonNumber}` }}</h4>
          <v-spacer></v-spacer>
        </div>
        
        <v-list lines="two">
          <v-list-item
            v-for="episode in currentEpisodes"
            :key="episode.id"
            @click="navigateToRelease(episode.id)"
          >
            <template #prepend>
              <v-avatar>
                <v-img
                  v-if="episode.thumbnailCID"
                  :src="parseUrlOrCid(episode.thumbnailCID)"
                  cover
                ></v-img>
                <v-icon v-else icon="$play"></v-icon>
              </v-avatar>
            </template>
            <v-list-item-title>{{ episode.name }}</v-list-item-title>
            <v-list-item-subtitle>
              Episode {{ episode.metadata?.episodeNumber }}
            </v-list-item-subtitle>
            <template #append>
              <v-btn
                icon="$open-in-new"
                variant="text"
                density="comfortable"
                size="small"
                @click.stop="navigateToRelease(episode.id)"
              ></v-btn>
            </template>
          </v-list-item>
        </v-list>
      </div>
      
      <!-- Music/Artists List View -->
      <div v-else-if="currentView === 'music-list'">
        <div class="d-flex align-center mb-4">
          <h4>Artists</h4>
          <v-spacer></v-spacer>
          <v-btn
            color="primary"
            prepend-icon="$plus"
            @click="createNewStructure('artist')"
          >
            New Artist
          </v-btn>
          <v-btn
            class="ml-2"
            variant="outlined"
            @click="showAllMusicReleases"
          >
            All Releases
          </v-btn>
        </div>
        
        <v-list lines="two">
          <v-list-item
            v-for="artist in artists"
            :key="artist.id"
            @click="navigateToArtist(artist)"
          >
            <template #prepend>
              <v-avatar>
                <v-img
                  v-if="artist.thumbnailCID"
                  :src="parseUrlOrCid(artist.thumbnailCID)"
                  cover
                ></v-img>
                <v-icon v-else icon="$account-music"></v-icon>
              </v-avatar>
            </template>
            <v-list-item-title>{{ artist.name }}</v-list-item-title>
            <v-list-item-subtitle>
              {{ artist.metadata?.genres?.join(', ') || 'No genres' }} • 
              {{ getArtistReleaseCount(artist.id) }} Releases
            </v-list-item-subtitle>
            <template #append>
              <v-btn
                icon="$pencil"
                variant="text"
                density="comfortable"
                size="small"
                @click.stop="editStructure(artist)"
              ></v-btn>
              <v-btn
                icon="$delete"
                variant="text"
                density="comfortable"
                size="small"
                @click.stop="deleteStructure(artist)"
              ></v-btn>
            </template>
          </v-list-item>
        </v-list>
      </div>
      
      <!-- Artist Detail View -->
      <div v-else-if="currentView === 'artist-detail'">
        <div class="d-flex align-center mb-4">
          <h4>{{ currentArtist?.name }}</h4>
          <v-spacer></v-spacer>
        </div>
        
        <v-list lines="two">
          <v-list-item
            v-for="release in currentArtistReleases"
            :key="release.id"
            @click="navigateToRelease(release.id)"
          >
            <template #prepend>
              <v-avatar>
                <v-img
                  v-if="release.thumbnailCID"
                  :src="parseUrlOrCid(release.thumbnailCID)"
                  cover
                ></v-img>
                <v-icon v-else icon="$music"></v-icon>
              </v-avatar>
            </template>
            <v-list-item-title>{{ release.name }}</v-list-item-title>
            <v-list-item-subtitle>
              {{ release.metadata?.releaseYear || 'Unknown year' }}
            </v-list-item-subtitle>
            <template #append>
              <v-btn
                icon="$open-in-new"
                variant="text"
                density="comfortable"
                size="small"
                @click.stop="navigateToRelease(release.id)"
              ></v-btn>
            </template>
          </v-list-item>
        </v-list>
      </div>
      
      <!-- Tags List View -->
      <div v-else-if="currentView === 'tags-list'">
        <div class="d-flex align-center mb-4">
          <h4>Tags</h4>
          <v-spacer></v-spacer>
          <v-btn
            color="primary"
            prepend-icon="$plus"
            @click="createNewStructure('tag')"
          >
            New Tag
          </v-btn>
        </div>
        
        <v-chip-group>
          <v-chip
            v-for="tag in tags"
            :key="tag.id"
            :color="tag.metadata?.color"
            closable
            @click="editStructure(tag)"
            @click:close="deleteStructure(tag)"
          >
            {{ tag.name }}
          </v-chip>
        </v-chip-group>
      </div>
      
      <!-- Collections List View -->
      <div v-else-if="currentView === 'collections-list'">
        <div class="d-flex align-center mb-4">
          <h4>Collections</h4>
          <v-spacer></v-spacer>
          <v-btn
            color="primary"
            prepend-icon="$plus"
            @click="createNewStructure('collection')"
          >
            New Collection
          </v-btn>
        </div>
        
        <v-list lines="two">
          <v-list-item
            v-for="collection in collections"
            :key="collection.id"
            @click="navigateToCollection(collection)"
          >
            <template #prepend>
              <v-avatar>
                <v-img
                  v-if="collection.thumbnailCID"
                  :src="parseUrlOrCid(collection.thumbnailCID)"
                  cover
                ></v-img>
                <v-icon v-else icon="$folder"></v-icon>
              </v-avatar>
            </template>
            <v-list-item-title>{{ collection.name }}</v-list-item-title>
            <v-list-item-subtitle>
              {{ collection.metadata?.collectionType || 'Custom' }} • 
              {{ collection.itemIds?.length || 0 }} items
            </v-list-item-subtitle>
            <template #append>
              <v-btn
                icon="$pencil"
                variant="text"
                density="comfortable"
                size="small"
                @click.stop="editStructure(collection)"
              ></v-btn>
              <v-btn
                icon="$delete"
                variant="text"
                density="comfortable"
                size="small"
                @click.stop="deleteStructure(collection)"
              ></v-btn>
            </template>
          </v-list-item>
        </v-list>
      </div>
      
      <!-- All Episodes View -->
      <div v-else-if="currentView === 'all-episodes'">
        <div class="d-flex align-center mb-4">
          <h4>All TV Episodes</h4>
          <v-spacer></v-spacer>
        </div>
        
        <v-data-table
          :headers="episodeTableHeaders"
          :items="allTVEpisodes"
          :items-per-page="25"
        >
          <template #item.name="{ item }">
            <div class="d-flex align-center">
              <v-avatar size="32" class="mr-2">
                <v-img
                  v-if="item.thumbnailCID"
                  :src="parseUrlOrCid(item.thumbnailCID)"
                ></v-img>
                <v-icon v-else size="small">$play</v-icon>
              </v-avatar>
              {{ item.name }}
            </div>
          </template>
          <template #item.series="{ item }">
            {{ item.metadata?.seriesName || 'Unknown' }}
          </template>
          <template #item.season="{ item }">
            S{{ String(item.metadata?.seasonNumber || 0).padStart(2, '0') }}
          </template>
          <template #item.episode="{ item }">
            E{{ String(item.metadata?.episodeNumber || 0).padStart(2, '0') }}
          </template>
          <template #item.actions="{ item }">
            <v-btn
              icon="$open-in-new"
              variant="text"
              density="comfortable"
              size="small"
              @click="navigateToRelease(item.id)"
            ></v-btn>
          </template>
        </v-data-table>
      </div>
      
      <!-- All Music Releases View -->
      <div v-else-if="currentView === 'all-music'">
        <div class="d-flex align-center mb-4">
          <h4>All Music Releases</h4>
          <v-spacer></v-spacer>
        </div>
        
        <v-data-table
          :headers="musicTableHeaders"
          :items="allMusicReleases"
          :items-per-page="25"
        >
          <template #item.name="{ item }">
            <div class="d-flex align-center">
              <v-avatar size="32" class="mr-2">
                <v-img
                  v-if="item.thumbnailCID"
                  :src="parseUrlOrCid(item.thumbnailCID)"
                ></v-img>
                <v-icon v-else size="small">$music</v-icon>
              </v-avatar>
              {{ item.name }}
            </div>
          </template>
          <template #item.artist="{ item }">
            {{ item.metadata?.author || 'Unknown' }}
          </template>
          <template #item.year="{ item }">
            {{ item.metadata?.releaseYear || '-' }}
          </template>
          <template #item.actions="{ item }">
            <v-btn
              icon="$open-in-new"
              variant="text"
              density="comfortable"
              size="small"
              @click="navigateToRelease(item.id)"
            ></v-btn>
          </template>
        </v-data-table>
      </div>
    </v-sheet>
  </v-container>
  
  <!-- Edit dialog -->
  <v-dialog v-model="editStructureDialog" max-width="600px">
    <v-card class="py-3">
      <v-card-title>
        <span class="text-h6 ml-2">Edit Structure</span>
      </v-card-title>
      <v-card-text>
        <structure-form
          :initial-data="editedStructure"
          mode="edit"
          @update:error="handleError"
          @update:success="handleSuccess"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          color="blue-darken-1"
          variant="text"
          @click="editStructureDialog = false"
        >
          Cancel
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
  
  <!-- Delete confirmation -->
  <confirmation-dialog
    message="Are you sure you want to delete this structure?"
    :dialog-open="confirmDeleteDialog"
    @close="() => {confirmDeleteDialog = false}"
    @confirm="confirmDelete"
  ></confirmation-dialog>
  
  <!-- Snackbar -->
  <v-snackbar
    v-model="showSnackbar"
    :color="snackbarMessage?.type ?? 'default'"
  >
    {{ snackbarMessage?.text }}
    <template #actions>
      <v-btn
        color="white"
        variant="text"
        @click="closeSnackbar"
      >
        Close
      </v-btn>
    </template>
  </v-snackbar>
</template>

<script setup lang="ts">
import { ref, computed, watchEffect } from 'vue';
import { useRouter } from 'vue-router';
import { useGetStructuresQuery, useGetReleasesQuery, useDeleteStructureMutation, useContentCategoriesQuery } from '/@/plugins/lensService/hooks';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import { parseUrlOrCid } from '/@/utils';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import structureForm from '/@/components/admin/structureForm.vue';

const router = useRouter();
const { data: structures, isLoading: isStructuresLoading } = useGetStructuresQuery();
const { data: releases, isLoading: isReleasesLoading } = useGetReleasesQuery();
const { data: contentCategories } = useContentCategoriesQuery();
const deleteStructureMutation = useDeleteStructureMutation();

const isLoading = computed(() => isStructuresLoading.value || isReleasesLoading.value);

// View state
const currentView = ref<'categories' | 'tv-list' | 'series-detail' | 'season-detail' | 'music-list' | 'artist-detail' | 'tags-list' | 'collections-list' | 'all-episodes' | 'all-music'>('categories');
const currentSeries = ref<any>(null);
const currentSeason = ref<any>(null);
const currentArtist = ref<any>(null);
const currentStructureType = ref<string>('series');
const parentContext = ref<any>(null);

// Dialog states
const createStructureDialog = ref(false);
const editStructureDialog = ref(false);
const confirmDeleteDialog = ref(false);

const editedStructure = ref<any>({});
const deleteTarget = ref<any>(null);

const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

// Computed properties for different structure types
const tvSeries = computed(() => {
  if (!structures.value) return [];
  // Show all series structures (empty ones will be cleaned up automatically)
  return structures.value.filter((s: any) => s.type === 'series');
});

// Get all season structures (for cleanup)
const allSeasons = computed(() => {
  if (!structures.value) return [];
  return structures.value.filter((s: any) => s.type === 'season');
});

// Store references to empty structures for manual cleanup
const emptyStructures = computed(() => {
  if (!releases.value || !structures.value || !contentCategories.value) return { series: [], seasons: [] };
  
  // Get all TV show category IDs (including federated)
  const tvCategoryIds = new Set<string>();
  contentCategories.value.forEach(cat => {
    if (cat.categoryId === 'tv-shows') {
      tvCategoryIds.add(cat.id);
      if (cat.allIds) {
        cat.allIds.forEach(id => tvCategoryIds.add(id));
      }
    }
  });
  
  // Find empty seasons (seasons with no episodes)
  const emptySeasons = allSeasons.value.filter(season => {
    const episodeCount = releases.value?.filter((r: any) => 
      tvCategoryIds.has(r.categoryId) && 
      r.metadata?.seriesId === season.parentId &&
      r.metadata?.seasonNumber === season.metadata?.seasonNumber
    ).length || 0;
    
    return episodeCount === 0;
  });
  
  // Find empty series (series with no episodes)
  const emptySeries = tvSeries.value.filter(series => {
    return getSeriesEpisodeCount(series.id) === 0;
  });
  
  return {
    series: emptySeries,
    seasons: emptySeasons
  };
});

const artists = computed(() => {
  if (!structures.value) return [];
  return structures.value.filter((s: any) => s.type === 'artist');
});

const tags = computed(() => {
  if (!structures.value) return [];
  return structures.value.filter((s: any) => s.type === 'tag');
});

const collections = computed(() => {
  if (!structures.value) return [];
  return structures.value.filter((s: any) => s.type === 'collection');
});

const currentSeasons = computed(() => {
  if (!structures.value || !currentSeries.value) return [];
  // Show all seasons for this series
  return structures.value
    .filter((s: any) => 
      s.type === 'season' && 
      s.parentId === currentSeries.value.id
    )
    .map((s: any) => {
      // Parse metadata if it's a string
      let metadata = s.metadata;
      if (typeof metadata === 'string') {
        try {
          metadata = JSON.parse(metadata);
        } catch (e) {
          console.error('Failed to parse season metadata:', e);
          metadata = {};
        }
      }
      return { ...s, metadata };
    })
    .sort((a: any, b: any) => (a.metadata?.seasonNumber || 0) - (b.metadata?.seasonNumber || 0));
});

const currentEpisodes = computed(() => {
  if (!releases.value || !currentSeason.value) return [];
  
  // Get all TV category hashes for federation support
  const tvCategoryIds = new Set<string>();
  contentCategories.value?.forEach(cat => {
    if (cat.categoryId === 'tv-shows') {
      tvCategoryIds.add(cat.id);
      if (cat.allIds) {
        cat.allIds.forEach(id => tvCategoryIds.add(id));
      }
    }
  });
  
  // Get the season number from metadata (handle both parsed and string)
  let seasonNumber = currentSeason.value?.metadata?.seasonNumber;
  if (seasonNumber === undefined && typeof currentSeason.value?.metadata === 'string') {
    try {
      const parsed = JSON.parse(currentSeason.value.metadata);
      seasonNumber = parsed.seasonNumber;
    } catch (e) {
      console.error('Failed to parse season metadata:', e);
    }
  }
  
  console.log('Looking for episodes with:', {
    seriesId: currentSeries.value?.id,
    seasonNumber,
    categoryIds: Array.from(tvCategoryIds)
  });
  
  // Episodes that belong to this season
  const episodes = releases.value
    .filter((r: any) => {
      const matches = tvCategoryIds.has(r.categoryId) && 
        r.metadata?.seriesId === currentSeries.value?.id &&
        r.metadata?.seasonNumber === seasonNumber;
      if (r.metadata?.seriesId === currentSeries.value?.id) {
        console.log('Episode check:', r.name, {
          hasCategory: tvCategoryIds.has(r.categoryId),
          seriesMatch: r.metadata?.seriesId === currentSeries.value?.id,
          seasonMatch: r.metadata?.seasonNumber === seasonNumber,
          episodeSeasonNumber: r.metadata?.seasonNumber,
          lookingForSeasonNumber: seasonNumber
        });
      }
      return matches;
    })
    .sort((a: any, b: any) => (a.metadata?.episodeNumber || 0) - (b.metadata?.episodeNumber || 0));
    
  console.log('Found episodes:', episodes.length);
  return episodes;
});

const orphanEpisodes = computed(() => {
  if (!releases.value || !currentSeries.value) return [];
  
  // Get all TV category hashes for federation support
  const tvCategoryIds = new Set<string>();
  contentCategories.value?.forEach(cat => {
    if (cat.categoryId === 'tv-shows') {
      tvCategoryIds.add(cat.id);
      if (cat.allIds) {
        cat.allIds.forEach(id => tvCategoryIds.add(id));
      }
    }
  });
  
  // Episodes that belong to this series but have no season
  return releases.value
    .filter((r: any) => 
      tvCategoryIds.has(r.categoryId) && 
      r.metadata?.seriesId === currentSeries.value.id &&
      !r.metadata?.seasonNumber
    );
});

const currentArtistReleases = computed(() => {
  if (!releases.value || !currentArtist.value) return [];
  return releases.value
    .filter((r: any) => 
      r.categorySlug === 'music' && 
      r.metadata?.artistId === currentArtist.value.id
    );
});

const allTVEpisodes = computed(() => {
  if (!releases.value || !contentCategories.value) return [];
  
  // Get all TV show category IDs (including federated)
  const tvCategoryIds = new Set<string>();
  contentCategories.value.forEach(cat => {
    if (cat.categoryId === 'tv-shows') {
      tvCategoryIds.add(cat.id);
      if (cat.allIds) {
        cat.allIds.forEach(id => tvCategoryIds.add(id));
      }
    }
  });
  
  return releases.value
    .filter((r: any) => tvCategoryIds.has(r.categoryId))
    .map((episode: any) => {
      // Find the series structure to get the name
      const seriesId = episode.metadata?.seriesId;
      const series = seriesId ? structures.value?.find((s: any) => s.id === seriesId && s.type === 'series') : null;
      
      // Enrich the episode with series name
      return {
        ...episode,
        metadata: {
          ...episode.metadata,
          seriesName: series?.name || 'Unknown'
        }
      };
    });
});

const allMusicReleases = computed(() => {
  if (!releases.value || !contentCategories.value) return [];
  
  // Get all music category IDs (including federated)
  const musicCategoryIds = new Set<string>();
  contentCategories.value.forEach(cat => {
    if (cat.categoryId === 'music') {
      musicCategoryIds.add(cat.id);
      if (cat.allIds) {
        cat.allIds.forEach(id => musicCategoryIds.add(id));
      }
    }
  });
  
  return releases.value.filter((r: any) => musicCategoryIds.has(r.categoryId));
});

// Counts
const tvSeriesCount = computed(() => tvSeries.value.length);
const artistCount = computed(() => artists.value.length);
const tagCount = computed(() => tags.value.length);
const collectionCount = computed(() => collections.value.length);

// Breadcrumbs
const breadcrumbs = computed(() => {
  const items = [];
  
  if (currentView.value !== 'categories') {
    items.push({ title: 'Categories', value: 'categories' });
  }
  
  if (currentView.value === 'tv-list' || currentView.value === 'series-detail' || currentView.value === 'season-detail' || currentView.value === 'all-episodes') {
    items.push({ title: 'TV Shows', value: 'tv-list' });
  }
  
  if (currentView.value === 'series-detail' || currentView.value === 'season-detail') {
    items.push({ title: currentSeries.value?.name || 'Series', value: 'series-detail' });
  }
  
  if (currentView.value === 'season-detail') {
    items.push({ 
      title: currentSeason.value?.name || `Season ${currentSeason.value?.metadata?.seasonNumber}`, 
      value: 'season-detail' 
    });
  }
  
  if (currentView.value === 'music-list' || currentView.value === 'artist-detail' || currentView.value === 'all-music') {
    items.push({ title: 'Music', value: 'music-list' });
  }
  
  if (currentView.value === 'artist-detail') {
    items.push({ title: currentArtist.value?.name || 'Artist', value: 'artist-detail' });
  }
  
  if (currentView.value === 'tags-list') {
    items.push({ title: 'Tags', value: 'tags-list' });
  }
  
  if (currentView.value === 'collections-list') {
    items.push({ title: 'Collections', value: 'collections-list' });
  }
  
  if (currentView.value === 'all-episodes') {
    items.push({ title: 'All Episodes', value: 'all-episodes' });
  }
  
  if (currentView.value === 'all-music') {
    items.push({ title: 'All Releases', value: 'all-music' });
  }
  
  return items;
});

// Table headers
const episodeTableHeaders = [
  { title: 'Episode', key: 'name' },
  { title: 'Series', key: 'series' },
  { title: 'Season', key: 'season' },
  { title: 'Episode', key: 'episode' },
  { title: 'Actions', key: 'actions', sortable: false }
];

const musicTableHeaders = [
  { title: 'Title', key: 'name' },
  { title: 'Artist', key: 'artist' },
  { title: 'Year', key: 'year' },
  { title: 'Actions', key: 'actions', sortable: false }
];

// Helper functions
function getSeriesEpisodeCount(seriesId: string): number {
  if (!releases.value || !contentCategories.value) return 0;
  
  // Get all TV show category IDs (including federated)
  const tvCategoryIds = new Set<string>();
  contentCategories.value.forEach(cat => {
    if (cat.categoryId === 'tv-shows') {
      tvCategoryIds.add(cat.id);
      if (cat.allIds) {
        cat.allIds.forEach(id => tvCategoryIds.add(id));
      }
    }
  });
  
  // Count episodes that belong to this series
  return releases.value.filter((r: any) => 
    tvCategoryIds.has(r.categoryId) && 
    r.metadata?.seriesId === seriesId
  ).length;
}

function getSeriesSeasonCount(seriesId: string): number {
  if (!releases.value || !contentCategories.value) return 0;
  
  // Get all TV show category IDs (including federated)
  const tvCategoryIds = new Set<string>();
  contentCategories.value.forEach(cat => {
    if (cat.categoryId === 'tv-shows') {
      tvCategoryIds.add(cat.id);
      if (cat.allIds) {
        cat.allIds.forEach(id => tvCategoryIds.add(id));
      }
    }
  });
  
  // Get unique season numbers from episodes of this series
  const seasonNumbers = new Set(
    releases.value
      .filter((r: any) => 
        tvCategoryIds.has(r.categoryId) && 
        r.metadata?.seriesId === seriesId && 
        r.metadata?.seasonNumber !== undefined
      )
      .map((r: any) => r.metadata.seasonNumber)
  );
  
  return seasonNumbers.size;
}

function formatSeasonCount(count: number): string {
  return count === 1 ? '1 Season' : `${count} Seasons`;
}

function formatEpisodeCount(count: number): string {
  return count === 1 ? '1 Episode' : `${count} Episodes`;
}

function getSeasonEpisodeCount(seasonId: string, seriesId?: string, seasonNumber?: number): number {
  if (!releases.value) return 0;
  
  // Get all TV category hashes for federation support
  const tvCategoryIds = new Set<string>();
  contentCategories.value?.forEach(cat => {
    if (cat.categoryId === 'tv-shows') {
      tvCategoryIds.add(cat.id);
      if (cat.allIds) {
        cat.allIds.forEach(id => tvCategoryIds.add(id));
      }
    }
  });
  
  // If we have series and season info, count actual episodes
  if (seriesId && seasonNumber !== undefined) {
    return releases.value.filter((r: any) => 
      tvCategoryIds.has(r.categoryId) && 
      r.metadata?.seriesId === seriesId &&
      r.metadata?.seasonNumber === seasonNumber
    ).length;
  }
  
  // Fallback to checking season's itemIds
  const season = structures.value?.find((s: any) => s.id === seasonId);
  return season?.itemIds?.length || 0;
}

function getArtistReleaseCount(artistId: string): number {
  if (!releases.value) return 0;
  return releases.value.filter((r: any) => 
    r.categorySlug === 'music' && r.metadata?.artistId === artistId
  ).length;
}

// Navigation functions
function navigateToCategory(category: string) {
  switch (category) {
    case 'tv':
      currentView.value = 'tv-list';
      break;
    case 'music':
      currentView.value = 'music-list';
      break;
    case 'tags':
      currentView.value = 'tags-list';
      break;
    case 'collections':
      currentView.value = 'collections-list';
      break;
  }
}

function navigateToSeries(series: any) {
  // Store the full series structure (not filtered)
  currentSeries.value = structures.value?.find((s: any) => s.id === series.id) || series;
  currentView.value = 'series-detail';
}

function navigateToSeason(season: any) {
  currentSeason.value = season;
  currentView.value = 'season-detail';
}

function navigateToArtist(artist: any) {
  currentArtist.value = artist;
  currentView.value = 'artist-detail';
}

function navigateToCollection(collection: any) {
  // TODO: Implement collection detail view
  openSnackbar('Collection detail view coming soon', 'info');
}

function navigateToRelease(releaseId: string) {
  router.push(`/release/${releaseId}`);
}

function navigateToBreadcrumb(item: any) {
  currentView.value = item.value;
  
  // Reset context when navigating up
  if (item.value === 'categories' || item.value === 'tv-list' || item.value === 'music-list') {
    currentSeries.value = null;
    currentSeason.value = null;
    currentArtist.value = null;
  }
  
  if (item.value === 'series-detail') {
    currentSeason.value = null;
  }
}

function showAllEpisodes() {
  currentView.value = 'all-episodes';
}

function showAllMusicReleases() {
  currentView.value = 'all-music';
}

function createNewStructure(type: string) {
  currentStructureType.value = type;
  
  // Set parent context if applicable
  if (type === 'season' && currentSeries.value) {
    parentContext.value = {
      parentId: currentSeries.value.id,
      parentName: currentSeries.value.name
    };
  } else {
    parentContext.value = null;
  }
  
  createStructureDialog.value = true;
}

// CRUD operations
function handleSuccess(message: string) {
  openSnackbar(message, 'success');
  createStructureDialog.value = false;
  editStructureDialog.value = false;
}

function handleError(message: string) {
  openSnackbar(message, 'error');
}

function editStructure(structure: any) {
  editedStructure.value = { ...structure };
  editStructureDialog.value = true;
}

function deleteStructure(structure: any) {
  deleteTarget.value = structure;
  confirmDeleteDialog.value = true;
}

async function confirmDelete() {
  try {
    if (deleteTarget.value) {
      await deleteStructureMutation.mutateAsync(deleteTarget.value.id);
      openSnackbar('Structure deleted successfully', 'success');
    }
  } catch (error) {
    handleError('Failed to delete structure');
  }
  confirmDeleteDialog.value = false;
}
</script>

<style scoped>
.v-card {
  cursor: pointer;
  transition: transform 0.2s;
}

.v-card:hover {
  transform: translateY(-2px);
}
</style>