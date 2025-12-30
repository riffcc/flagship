<template>
  <v-container>
    <v-sheet class="px-6 py-4 mx-auto" max-width="1200px">
      <!-- Header -->
      <v-list-item class="px-0">
        <template #prepend>
          <v-avatar v-if="status.data.value" :color="status.data.value.status === 'healthy' ? 'success' : 'warning'" size="32">
            <v-icon size="small">$archive</v-icon>
          </v-avatar>
        </template>
        <h3>{{ currentViewTitle }}</h3>
        <template #append>
          <!-- Status bar: selection + job status in a unified card -->
          <v-card
            v-if="totalSelectionCount > 0 || runningJobCount > 0 || pendingJobCount > 0"
            class="status-bar d-flex align-center px-3 py-1"
            variant="tonal"
            color="surface-variant"
          >
            <!-- Selection info -->
            <template v-if="totalSelectionCount > 0">
              <v-btn
                icon="$close"
                variant="text"
                size="x-small"
                density="compact"
                @click="clearSelection"
              ></v-btn>
              <v-btn color="primary" size="small" variant="outlined" @click="currentView = 'selected'">
                {{ totalSelectionCount }} selected<span v-if="allInCollectionSelected">&nbsp;(all)</span>
              </v-btn>
            </template>

            <!-- Job status -->
            <v-btn
              v-if="runningJobCount > 0"
              color="success"
              size="small"
              variant="outlined"
              @click="currentView = 'queue'"
            >
              {{ runningJobCount }} running
            </v-btn>
            <v-btn
              v-if="pendingJobCount > 0"
              color="warning"
              size="small"
              variant="outlined"
              @click="currentView = 'queue'"
            >
              {{ pendingJobCount }} pending
            </v-btn>

            <!-- Action button -->
            <v-btn
              v-if="totalSelectionCount > 0"
              color="light-green-darken-1"
              size="small"
              @click="currentView = 'selected'"
            >
              Review & Import
            </v-btn>
          </v-card>
        </template>
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
        v-if="status.isLoading.value"
        indeterminate
        color="primary"
        class="mt-4"
      ></v-progress-linear>

      <!-- Connection Error -->
      <v-alert
        v-else-if="status.error.value"
        type="error"
        variant="tonal"
        class="mb-4"
      >
        Cannot connect to Librarian daemon at {{ apiUrl }}
        <template #append>
          <v-btn variant="text" size="small" @click="status.refetch()">Retry</v-btn>
        </template>
      </v-alert>

      <!-- ============================================================ -->
      <!-- MAIN VIEW: Category Cards -->
      <!-- ============================================================ -->
      <div v-else-if="currentView === 'main'">
        <v-row>
          <!-- Collections -->
          <v-col cols="12" md="6">
            <v-card class="nav-card" @click="currentView = 'collections'">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$folder-multiple" class="mr-2"></v-icon>
                Collections
                <v-spacer></v-spacer>
              </v-card-title>
              <v-card-text>
                Browse Archive.org collections (etree, GratefulDead, toucan, etc.)
              </v-card-text>
            </v-card>
          </v-col>

          <!-- Items -->
          <v-col cols="12" md="6">
            <v-card class="nav-card" @click="currentView = 'items'">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$playlist-music" class="mr-2"></v-icon>
                Items
                <v-spacer></v-spacer>
              </v-card-title>
              <v-card-text>
                Search and browse Archive.org items directly
              </v-card-text>
            </v-card>
          </v-col>

          <!-- Selected -->
          <v-col cols="12" md="6">
            <v-card class="nav-card" @click="currentView = 'selected'">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$playlist-check" class="mr-2"></v-icon>
                Selected
                <v-spacer></v-spacer>
                <v-chip v-if="totalSelectionCount > 0" color="primary">{{ totalSelectionCount }}</v-chip>
              </v-card-title>
              <v-card-text>
                Review selected items, edit metadata, and start imports
              </v-card-text>
            </v-card>
          </v-col>

          <!-- Queue -->
          <v-col cols="12" md="6">
            <v-card class="nav-card" @click="currentView = 'queue'">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$playlist-play" class="mr-2"></v-icon>
                Queue
                <v-spacer></v-spacer>
                <v-chip v-if="jobs.data.value?.length">{{ jobs.data.value.length }}</v-chip>
              </v-card-title>
              <v-card-text>
                Monitor import, transcode, and audit jobs
              </v-card-text>
            </v-card>
          </v-col>

          <!-- Quality -->
          <v-col cols="12" md="6">
            <v-card class="nav-card" @click="currentView = 'quality'">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$tune-vertical" class="mr-2"></v-icon>
                Quality
                <v-spacer></v-spacer>
              </v-card-title>
              <v-card-text>
                Quality ladder management and FLAC transcoding
              </v-card-text>
            </v-card>
          </v-col>

          <!-- HTTPS/S3/CID -->
          <v-col cols="12" md="6">
            <v-card class="nav-card" @click="currentView = 'sources'">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$cloud-download" class="mr-2"></v-icon>
                HTTPS/S3/CID
                <v-spacer></v-spacer>
              </v-card-title>
              <v-card-text>
                Import from URLs, S3 storage, or IPFS/Archivist CIDs
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </div>

      <!-- ============================================================ -->
      <!-- COLLECTIONS VIEW -->
      <!-- ============================================================ -->
      <div v-else-if="currentView === 'collections'">
        <!-- Search for collections -->
        <v-text-field
          v-model="collectionSearchQuery"
          label="Search for collections"
          placeholder="e.g. live music, grateful dead, jazz"
          prepend-inner-icon="$magnify"
          variant="outlined"
          density="compact"
          clearable
          hide-details
          class="mb-4"
          @keyup.enter="searchCollections"
        ></v-text-field>

        <div class="text-center text-grey mb-4">or enter a collection ID directly</div>

        <!-- Direct collection ID entry -->
        <div class="d-flex align-center mb-4">
          <v-text-field
            v-model="collectionId"
            label="Collection ID"
            placeholder="e.g. etree, GratefulDead, toucan, 78rpm"
            variant="outlined"
            density="compact"
            hide-details
            class="flex-grow-1 mr-4"
            @keyup.enter="loadCollection"
          ></v-text-field>
          <v-btn
            color="primary"
            :loading="collection.isLoading.value"
            @click="loadCollection"
          >
            Browse
          </v-btn>
        </div>

        <!-- Collection Search Results -->
        <div v-if="collectionSearchResults.data.value?.items?.length && !collectionIdQuery">
          <v-progress-linear
            v-if="collectionSearchResults.isLoading.value"
            indeterminate
            color="primary"
            class="mb-4"
          ></v-progress-linear>

          <div class="text-body-2 text-grey mb-3">
            Found {{ collectionSearchResults.data.value.total }} collections
          </div>

          <div class="items-grid">
            <v-card
              v-for="coll in collectionSearchResults.data.value.items"
              :key="coll.identifier"
              class="nav-card"
              @click="selectCollection(coll.identifier)"
            >
              <v-img
                :src="getThumbnailUrl(coll.identifier)"
                height="120"
                cover
                class="bg-grey-darken-3"
              >
                <template #placeholder>
                  <div class="d-flex align-center justify-center fill-height">
                    <v-progress-circular indeterminate size="24" color="grey"></v-progress-circular>
                  </div>
                </template>
                <template #error>
                  <div class="d-flex align-center justify-center fill-height bg-grey-darken-3">
                    <v-icon size="32" color="grey">$folder-multiple</v-icon>
                  </div>
                </template>
              </v-img>
              <v-card-text class="pa-3">
                <div class="text-subtitle-2 text-truncate">{{ coll.title || coll.identifier }}</div>
                <div class="text-caption text-grey text-truncate">{{ coll.identifier }}</div>
              </v-card-text>
            </v-card>
          </div>
        </div>

        <!-- Browsing a specific collection: Toolbar with View Mode Toggle + Select All -->
        <div v-if="accumulatedItems.length > 0" class="d-flex align-center mb-4">
          <!-- View Mode Toggle -->
          <v-btn-toggle v-model="viewMode" mandatory density="compact" variant="outlined" class="mr-4">
            <v-btn value="endless" size="small">
              <v-icon size="small" class="mr-1">$gesture-swipe-horizontal</v-icon>
              Endless
            </v-btn>
            <v-btn value="paged" size="small">
              <v-icon size="small" class="mr-1">$format-list-bulleted</v-icon>
              Paged
            </v-btn>
          </v-btn-toggle>

          <!-- Select All Dropdown -->
          <v-menu>
            <template #activator="{ props: menuProps }">
              <v-btn v-bind="menuProps" variant="outlined" size="small" prepend-icon="$checkbox-marked">
                Select
                <v-icon size="small" class="ml-1">$menu-down</v-icon>
              </v-btn>
            </template>
            <v-list density="compact">
              <v-list-item @click="selectAllOnPage">
                <v-list-item-title>All on this page ({{ visibleItems.length }})</v-list-item-title>
              </v-list-item>
              <v-list-item @click="selectAllInCollection">
                <v-list-item-title>All in collection ({{ totalResults }})</v-list-item-title>
              </v-list-item>
              <v-divider class="my-1" />
              <v-list-item @click="clearSelection" :disabled="totalSelectionCount === 0">
                <v-list-item-title class="text-error">None (clear selection)</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>

          <v-spacer></v-spacer>

          <!-- Stats -->
          <span class="text-body-2 text-grey">
            Showing {{ visibleItems.length }} of {{ totalResults }}
          </span>
        </div>

        <!-- Loading State -->
        <v-progress-linear
          v-if="collection.isLoading.value"
          indeterminate
          color="primary"
          class="mb-4"
        ></v-progress-linear>

        <!-- Results Grid (CSS Grid like infiniteReleaseList) -->
        <div v-if="accumulatedItems.length > 0" class="items-grid-container">
          <div class="items-grid">
            <v-card
              v-for="item in visibleItems"
              :key="item.identifier"
              class="item-card"
              :class="{ 'selected': isSelected(item.identifier) }"
              @click="toggleSelection(item)"
            >
              <v-img
                :src="getThumbnailUrl(item.identifier)"
                aspect-ratio="1"
                cover
                class="bg-grey-darken-3"
              >
                <template #placeholder>
                  <div class="d-flex align-center justify-center fill-height">
                    <v-progress-circular indeterminate size="24" color="grey"></v-progress-circular>
                  </div>
                </template>
                <template #error>
                  <div class="d-flex align-center justify-center fill-height bg-grey-darken-3">
                    <v-icon size="32" color="grey">$music</v-icon>
                  </div>
                </template>
                <!-- Selection indicator -->
                <v-icon
                  v-if="isSelected(item.identifier)"
                  class="selection-check"
                  color="primary"
                  size="24"
                >
                  $check-circle
                </v-icon>
              </v-img>
              <v-card-text class="pa-2">
                <div class="text-subtitle-2 text-truncate">{{ item.title || item.identifier }}</div>
                <div class="text-caption text-grey text-truncate">{{ item.creator || 'Unknown' }}</div>
              </v-card-text>
              <v-card-actions class="pa-2 pt-0">
                <v-btn
                  icon="$play"
                  variant="text"
                  size="x-small"
                  @click.stop="previewItem(item)"
                ></v-btn>
                <v-btn
                  icon="$information"
                  variant="text"
                  size="x-small"
                  @click.stop="viewItemDetails(item)"
                ></v-btn>
                <v-spacer></v-spacer>
                <v-chip size="x-small" variant="tonal">{{ item.mediatype }}</v-chip>
              </v-card-actions>
            </v-card>
          </div>

          <!-- Endless Scroll: Invisible trigger for loading more -->
          <v-sheet
            v-if="viewMode === 'endless' && hasMoreItems"
            v-intersect="onIntersect"
            height="100"
            class="d-flex align-center justify-center"
            color="transparent"
          >
            <v-progress-circular
              v-if="isLoadingMore"
              indeterminate
              size="32"
              color="primary"
            ></v-progress-circular>
          </v-sheet>

          <!-- Paged Mode: Pagination controls -->
          <div v-if="viewMode === 'paged'" class="d-flex align-center justify-center mt-4">
            <v-btn icon="$chevron-left" variant="text" :disabled="currentPage <= 1" @click="goToPreviousPage"></v-btn>
            <span class="mx-4">Page {{ currentPage }} of {{ totalPages }}</span>
            <v-btn icon="$chevron-right" variant="text" :disabled="currentPage >= totalPages" @click="goToNextPage"></v-btn>
          </div>
        </div>

        <!-- Empty State -->
        <v-sheet
          v-else-if="!collection.isLoading.value && !searchResults.isLoading.value"
          class="text-center pa-8"
          color="transparent"
        >
          <v-icon size="64" color="grey" class="mb-4">$archive</v-icon>
          <p class="text-body-1 text-grey">Enter a collection ID or search to browse Archive.org</p>
        </v-sheet>

      </div>

      <!-- ============================================================ -->
      <!-- ITEMS VIEW (Search/Browse Items Directly) -->
      <!-- ============================================================ -->
      <div v-else-if="currentView === 'items'">
        <v-text-field
          v-model="searchQuery"
          label="Search Archive.org items"
          placeholder="e.g. grateful dead 1977, live jazz, beethoven symphony"
          prepend-inner-icon="$magnify"
          variant="outlined"
          density="compact"
          clearable
          hide-details
          class="mb-4"
          @keyup.enter="executeSearch"
        ></v-text-field>

        <!-- Loading -->
        <v-progress-linear
          v-if="searchResults.isLoading.value"
          indeterminate
          color="primary"
          class="mb-4"
        ></v-progress-linear>

        <!-- Search Results -->
        <div v-if="searchResults.data.value?.items?.length" class="items-grid-container">
          <div class="text-body-2 text-grey mb-3">
            Found {{ searchResults.data.value.total }} items
          </div>
          <div class="items-grid">
            <v-card
              v-for="item in searchResults.data.value.items"
              :key="item.identifier"
              class="item-card"
              :class="{ 'selected': isSelected(item.identifier) }"
              @click="toggleSelection(item)"
            >
              <v-img
                :src="getThumbnailUrl(item.identifier)"
                aspect-ratio="1"
                cover
                class="bg-grey-darken-3"
              >
                <template #placeholder>
                  <div class="d-flex align-center justify-center fill-height">
                    <v-progress-circular indeterminate size="24" color="grey"></v-progress-circular>
                  </div>
                </template>
                <template #error>
                  <div class="d-flex align-center justify-center fill-height bg-grey-darken-3">
                    <v-icon size="32" color="grey">$music</v-icon>
                  </div>
                </template>
                <v-icon
                  v-if="isSelected(item.identifier)"
                  class="selection-check"
                  color="primary"
                  size="24"
                >
                  $check-circle
                </v-icon>
              </v-img>
              <v-card-text class="pa-2">
                <div class="text-subtitle-2 text-truncate">{{ item.title || item.identifier }}</div>
                <div class="text-caption text-grey text-truncate">{{ item.creator || 'Unknown' }}</div>
              </v-card-text>
              <v-card-actions class="pa-2 pt-0">
                <v-btn
                  icon="$play"
                  variant="text"
                  size="x-small"
                  @click.stop="previewItem(item)"
                ></v-btn>
                <v-btn
                  icon="$information"
                  variant="text"
                  size="x-small"
                  @click.stop="viewItemDetails(item)"
                ></v-btn>
                <v-spacer></v-spacer>
                <v-chip size="x-small" variant="tonal">{{ item.mediatype }}</v-chip>
              </v-card-actions>
            </v-card>
          </div>
        </div>

        <!-- Empty State -->
        <v-sheet
          v-else-if="!searchResults.isLoading.value"
          class="text-center pa-8"
          color="transparent"
        >
          <v-icon size="64" color="grey" class="mb-4">$magnify</v-icon>
          <p class="text-body-1 text-grey">Search Archive.org to find items</p>
        </v-sheet>

      </div>

      <!-- ============================================================ -->
      <!-- SELECTED VIEW (Items for Import) -->
      <!-- ============================================================ -->
      <div v-else-if="currentView === 'selected'">
        <div class="d-flex align-center mb-4">
          <h4>Selected Items</h4>
          <v-spacer></v-spacer>
          <v-btn variant="outlined" class="mr-2" @click="currentView = 'collections'">
            Browse Collections
          </v-btn>
          <v-btn variant="outlined" class="mr-2" @click="currentView = 'items'">
            Search Items
          </v-btn>
          <v-btn
            color="primary"
            :disabled="totalSelectionCount === 0"
            @click="startImport"
          >
            Import {{ totalSelectionCount }} Items
          </v-btn>
        </div>

        <v-alert v-if="totalSelectionCount === 0" type="info" variant="tonal">
          No items selected. Go to Collections to browse and select items.
        </v-alert>

        <!-- "All in Collection" summary -->
        <v-alert v-else-if="allInCollectionSelected" type="success" variant="tonal" class="mb-4">
          <div class="d-flex align-center">
            <v-icon class="mr-2">$check-circle</v-icon>
            <div>
              <strong>Entire collection selected:</strong> {{ allInCollectionCount }} items from "{{ allInCollectionId }}"
            </div>
          </div>
        </v-alert>

        <!-- Import Options -->
        <v-card v-if="totalSelectionCount > 0" class="mb-4">
          <v-card-title>Import Options</v-card-title>
          <v-card-text>
            <v-switch
              v-model="autoApprove"
              label="Auto-approve (skip moderation queue)"
              color="primary"
              hide-details
              class="mb-2"
            ></v-switch>
            <v-switch
              v-model="transcodeFlac"
              label="Transcode FLAC to quality ladder (Opus 192/160/128/96/64)"
              color="primary"
              hide-details
            ></v-switch>
          </v-card-text>
        </v-card>

        <!-- Show individually selected items (if any loaded) -->
        <v-list v-if="selectedItems.length > 0" lines="three">
          <v-list-subheader v-if="allInCollectionSelected">
            Preview ({{ selectedItems.length }} of {{ allInCollectionCount }} loaded)
          </v-list-subheader>
          <v-list-item
            v-for="item in selectedItems"
            :key="item.identifier"
          >
            <template #prepend>
              <v-avatar size="64" rounded>
                <v-img :src="getThumbnailUrl(item.identifier)"></v-img>
              </v-avatar>
            </template>
            <v-list-item-title>{{ item.title || item.identifier }}</v-list-item-title>
            <v-list-item-subtitle>{{ item.creator || 'Unknown artist' }}</v-list-item-subtitle>
            <v-list-item-subtitle>
              <v-chip size="x-small" variant="tonal" class="mr-1">{{ item.mediatype }}</v-chip>
              <v-chip v-if="item.date" size="x-small" variant="tonal">{{ item.date }}</v-chip>
            </v-list-item-subtitle>
            <template #append>
              <v-btn icon="$play" variant="text" size="small" @click="previewItem(item)"></v-btn>
              <v-btn v-if="!allInCollectionSelected" icon="$delete" variant="text" size="small" color="error" @click="removeFromSelection(item)"></v-btn>
            </template>
          </v-list-item>
        </v-list>
      </div>

      <!-- ============================================================ -->
      <!-- QUEUE VIEW -->
      <!-- ============================================================ -->
      <div v-else-if="currentView === 'queue'">
        <div class="d-flex align-center mb-4">
          <h4>Job Queue</h4>
          <v-chip
            v-if="archivedJobs.length > 0"
            size="small"
            variant="tonal"
            color="grey"
            class="ml-2"
          >
            {{ archivedJobs.length }} archived
          </v-chip>
          <v-spacer></v-spacer>
          <v-btn
            v-if="archivedJobs.length > 0"
            color="grey"
            variant="tonal"
            size="small"
            prepend-icon="$delete"
            class="mr-2"
            :loading="clearArchivedMutation.isPending.value"
            @click="clearArchivedJobs"
          >
            Clear Archived
          </v-btn>
          <v-btn color="primary" prepend-icon="$plus" @click="createJobDialog = true">
            Create Job
          </v-btn>
        </div>

        <v-progress-linear v-if="jobs.isLoading.value" indeterminate color="primary" class="mb-4"></v-progress-linear>

        <!-- Job List with Expandable Details -->
        <div v-if="jobs.data.value?.length" class="job-list">
          <v-card
            v-for="job in sortedJobs"
            :key="job.id"
            class="job-card mb-3"
            :class="{ 'job-running': job.status === 'Running' }"
          >
            <v-card-item class="clickable-card" @click="showJobDetails(job)">
              <!-- Status Icon -->
              <template #prepend>
                <v-avatar
                  :color="getJobStatusColor(job.status)"
                  size="40"
                  class="mr-3"
                >
                  <v-progress-circular
                    v-if="job.status === 'Running'"
                    indeterminate
                    size="24"
                    width="2"
                    color="white"
                  ></v-progress-circular>
                  <v-icon v-else-if="job.status === 'Pending'" color="white" size="20">$clock-outline</v-icon>
                  <v-icon v-else-if="job.status === 'Complete'" color="white" size="20">$check</v-icon>
                  <v-icon v-else-if="job.status === 'Failed'" color="white" size="20">$alert-circle</v-icon>
                </v-avatar>
              </template>

              <!-- Job Info -->
              <v-card-title class="text-body-1">
                <v-chip size="small" variant="outlined" class="mr-2">{{ job.job_type }}</v-chip>
                <span class="text-truncate">{{ formatJobTarget(job.target) }}</span>
              </v-card-title>
              <v-card-subtitle>
                {{ formatTimestamp(job.created_at) }}
                <span v-if="job.status === 'Running'" class="ml-2 text-success">
                  {{ job.progress ? `${Math.round(job.progress * 100)}%` : 'Starting...' }}
                </span>
              </v-card-subtitle>

              <!-- Action Buttons -->
              <template #append>
                <!-- Play button for pending jobs -->
                <v-btn
                  v-if="job.status === 'Pending'"
                  icon="$play"
                  variant="text"
                  size="small"
                  color="success"
                  @click.stop="startJob(job.id)"
                ></v-btn>
                <!-- Retry button for failed jobs -->
                <v-btn
                  v-if="job.status === 'Failed'"
                  icon="$refresh"
                  variant="text"
                  size="small"
                  color="warning"
                  @click.stop="retryJob(job.id)"
                ></v-btn>
                <!-- Archive button for completed/failed jobs -->
                <v-btn
                  v-if="job.status === 'Completed' || job.status === 'Failed'"
                  icon="$archive-outline"
                  variant="text"
                  size="small"
                  color="grey"
                  title="Archive job"
                  @click.stop="archiveJob(job.id)"
                ></v-btn>
                <!-- Stop button for running jobs -->
                <v-btn
                  v-if="job.status === 'Running'"
                  icon="$stop"
                  variant="text"
                  size="small"
                  color="error"
                  @click.stop="stopJob(job.id)"
                ></v-btn>
                <!-- Expand toggle -->
                <v-btn
                  :icon="expandedJobs.has(job.id) ? '$chevron-up' : '$chevron-down'"
                  variant="text"
                  size="small"
                  @click.stop="toggleJobExpand(job.id)"
                ></v-btn>
              </template>
            </v-card-item>

            <!-- Running job progress bar -->
            <v-progress-linear
              v-if="job.status === 'Running'"
              :model-value="(job.progress || 0) * 100"
              color="success"
              height="3"
            ></v-progress-linear>

            <!-- Expanded Details -->
            <v-expand-transition>
              <div v-if="expandedJobs.has(job.id)">
                <v-divider></v-divider>
                <v-card-text>
                  <!-- Job ID -->
                  <div class="mb-2">
                    <span class="text-caption text-grey">Job ID:</span>
                    <code class="ml-2 text-caption">{{ job.id }}</code>
                  </div>

                  <!-- Progress Message -->
                  <div v-if="job.progress_message" class="mb-2">
                    <span class="text-caption text-grey">Status:</span>
                    <span class="ml-2">{{ job.progress_message }}</span>
                  </div>

                  <!-- Result for completed jobs -->
                  <div v-if="job.result" class="mt-3">
                    <v-alert
                      v-if="getResultType(job.result) === 'Import'"
                      type="success"
                      variant="tonal"
                      density="compact"
                    >
                      Imported {{ getResultData(job.result).files_imported }} files
                    </v-alert>
                    <v-alert
                      v-else-if="getResultType(job.result) === 'Error'"
                      type="error"
                      variant="tonal"
                      density="compact"
                    >
                      {{ getResultData(job.result) }}
                    </v-alert>
                    <v-alert
                      v-else-if="getResultType(job.result) === 'Audit'"
                      type="info"
                      variant="tonal"
                      density="compact"
                    >
                      Audited {{ getResultData(job.result).total_releases }} releases
                      <span v-if="getResultData(job.result).releases_with_issues > 0">
                        · {{ getResultData(job.result).releases_with_issues }} with issues
                      </span>
                    </v-alert>
                    <v-alert
                      v-else-if="getResultType(job.result) === 'Transcode'"
                      type="success"
                      variant="tonal"
                      density="compact"
                    >
                      Created {{ getResultData(job.result).outputs?.length || 0 }} variants
                    </v-alert>
                    <v-alert
                      v-else-if="getResultType(job.result) === 'SourceImport'"
                      type="success"
                      variant="tonal"
                      density="compact"
                    >
                      Imported {{ formatSize(String(getResultData(job.result).size || 0)) }}
                      <span v-if="getResultData(job.result).new_cid"> · CID: {{ getResultData(job.result).new_cid?.slice(0, 12) }}...</span>
                    </v-alert>
                    <v-alert
                      v-else-if="getResultType(job.result) === 'NeedsInput'"
                      type="warning"
                      variant="tonal"
                      density="compact"
                    >
                      <div class="d-flex align-center">
                        <v-icon class="mr-2">$pencil</v-icon>
                        <span>Needs metadata input</span>
                        <v-btn
                          size="small"
                          color="warning"
                          variant="text"
                          class="ml-2"
                          @click.stop="openMetadataDialog(job)"
                        >
                          Provide Info
                        </v-btn>
                      </div>
                    </v-alert>
                  </div>
                </v-card-text>
              </div>
            </v-expand-transition>
          </v-card>
        </div>

        <v-sheet v-else class="text-center pa-8" color="transparent">
          <v-icon size="64" color="grey" class="mb-4">$playlist-check</v-icon>
          <p class="text-body-1 text-grey">No jobs in queue</p>
        </v-sheet>
      </div>

      <!-- ============================================================ -->
      <!-- QUALITY VIEW -->
      <!-- ============================================================ -->
      <div v-else-if="currentView === 'quality'">
        <div class="d-flex align-center mb-4">
          <h4>Quality Ladder</h4>
          <v-spacer></v-spacer>

          <!-- Sub-navigation tabs -->
          <v-btn-toggle v-model="qualitySubView" mandatory density="compact" variant="outlined">
            <v-btn value="overview" size="small">Overview</v-btn>
            <v-btn value="issues" size="small">
              Issues
              <v-chip
                v-if="auditResults?.releasesWithIssues"
                size="x-small"
                color="warning"
                class="ml-2"
              >
                {{ auditResults.releasesWithIssues }}
              </v-chip>
            </v-btn>
          </v-btn-toggle>
        </div>

        <!-- QUALITY OVERVIEW -->
        <div v-if="qualitySubView === 'overview'">
          <v-alert type="info" variant="tonal" class="mb-4">
            Quality ladder: 24-bit FLAC, 16-bit FLAC, Opus 192, Opus 160, Opus 128, Opus 96, Opus 64
          </v-alert>

          <v-row>
            <v-col cols="12" md="6">
              <v-card>
                <v-card-title>
                  <v-icon class="mr-2">$check-circle</v-icon>
                  Audit Releases
                </v-card-title>
                <v-card-text>
                  Scan all releases in Citadel Lens for quality issues
                  <div v-if="auditResults" class="mt-3">
                    <v-chip size="small" color="success" class="mr-2">
                      {{ auditResults.totalReleases - auditResults.releasesWithIssues }} healthy
                    </v-chip>
                    <v-chip size="small" color="warning">
                      {{ auditResults.releasesWithIssues }} with issues
                    </v-chip>
                  </div>
                </v-card-text>
                <v-card-actions>
                  <v-btn color="primary" @click="startAudit">Start Audit</v-btn>
                  <v-btn
                    v-if="auditResults?.releasesWithIssues"
                    variant="outlined"
                    @click="qualitySubView = 'issues'"
                  >
                    View Issues
                  </v-btn>
                </v-card-actions>
              </v-card>
            </v-col>
            <v-col cols="12" md="6">
              <v-card>
                <v-card-title>
                  <v-icon class="mr-2">$tune</v-icon>
                  Transcode Queue
                </v-card-title>
                <v-card-text>
                  FLAC files waiting to be transcoded to Opus variants
                </v-card-text>
                <v-card-actions>
                  <v-btn variant="outlined">View Queue</v-btn>
                </v-card-actions>
              </v-card>
            </v-col>
          </v-row>

          <!-- Issue Summary Cards (if audit results exist) -->
          <div v-if="auditResults?.issueCounts && Object.keys(auditResults.issueCounts).length > 0" class="mt-6">
            <h5 class="mb-3">Issue Summary</h5>
            <div class="issue-summary-grid">
              <v-card
                v-for="(count, issueType) in auditResults.issueCounts"
                :key="issueType"
                class="issue-summary-card"
                :color="issueColors[issueType] || 'grey'"
                variant="tonal"
                @click="issueFilter = issueType; qualitySubView = 'issues'"
              >
                <v-card-text class="text-center pa-3">
                  <div class="text-h5 font-weight-bold">{{ count }}</div>
                  <div class="text-caption">{{ issueLabels[issueType] || issueType }}</div>
                </v-card-text>
              </v-card>
            </div>
          </div>
        </div>

        <!-- QUALITY ISSUES -->
        <div v-else-if="qualitySubView === 'issues'">
          <!-- No audit results yet -->
          <v-alert v-if="!auditResults" type="info" variant="tonal" class="mb-4">
            No audit results yet. Run an audit to detect quality issues.
            <template #append>
              <v-btn color="primary" size="small" @click="startAudit">Start Audit</v-btn>
            </template>
          </v-alert>

          <!-- Audit results exist -->
          <template v-else>
            <!-- Filter bar and Fix All button -->
            <div class="d-flex align-center flex-wrap gap-2 mb-4">
              <v-chip
                :color="issueFilter === null ? 'primary' : 'default'"
                :variant="issueFilter === null ? 'flat' : 'outlined'"
                size="small"
                @click="filterByIssue(null)"
              >
                All ({{ auditResults.releasesWithIssues }})
              </v-chip>
              <v-chip
                v-for="(count, issueType) in auditResults.issueCounts"
                :key="issueType"
                :color="issueFilter === issueType ? issueColors[issueType] : 'default'"
                :variant="issueFilter === issueType ? 'flat' : 'outlined'"
                size="small"
                @click="filterByIssue(issueType)"
              >
                {{ issueLabels[issueType] || issueType }} ({{ count }})
              </v-chip>
              <v-spacer></v-spacer>
              <v-btn
                v-if="getTotalAutoFixableCount() > 0"
                color="primary"
                variant="flat"
                size="small"
                prepend-icon="$auto-fix"
                @click="fixAllReleases"
              >
                Fix all releases ({{ getTotalAutoFixableCount() }} issues)
              </v-btn>
            </div>

            <!-- Issues list -->
            <div v-if="filteredAudits.length > 0" class="issues-list">
              <v-card
                v-for="audit in filteredAudits"
                :key="audit.releaseId"
                class="issue-card mb-3"
              >
                <v-card-item @click="toggleAuditExpand(audit.releaseId)">
                  <template #prepend>
                    <v-avatar color="surface-variant" size="40">
                      <v-icon>$music</v-icon>
                    </v-avatar>
                  </template>

                  <v-card-title class="text-body-1">
                    {{ audit.title }}
                  </v-card-title>
                  <v-card-subtitle>
                    {{ audit.artist || 'Unknown artist' }}
                    <span v-if="audit.sourceQuality" class="ml-2">
                      · {{ audit.sourceQuality.toUpperCase() }}
                    </span>
                  </v-card-subtitle>

                  <template #append>
                    <!-- Issue chips summary -->
                    <div class="d-flex align-center gap-1 mr-2">
                      <v-chip
                        v-for="issue in audit.issues.slice(0, 3)"
                        :key="getIssueKey(issue)"
                        :color="getIssueColor(issue)"
                        size="x-small"
                        variant="tonal"
                      >
                        {{ getIssueLabel(issue) }}
                      </v-chip>
                      <v-chip
                        v-if="audit.issues.length > 3"
                        size="x-small"
                        variant="tonal"
                      >
                        +{{ audit.issues.length - 3 }}
                      </v-chip>
                    </div>
                    <v-btn
                      :icon="expandedAudits.has(audit.releaseId) ? '$chevron-up' : '$chevron-down'"
                      variant="text"
                      size="small"
                    ></v-btn>
                  </template>
                </v-card-item>

                <!-- Expanded details -->
                <v-expand-transition>
                  <div v-if="expandedAudits.has(audit.releaseId)">
                    <v-divider></v-divider>
                    <v-card-text>
                      <!-- All issues - clickable to fix -->
                      <div class="mb-3">
                        <div class="text-caption text-grey mb-2">Issues</div>
                        <div class="d-flex flex-wrap gap-2">
                          <v-chip
                            v-for="issue in audit.issues"
                            :key="getIssueKey(issue)"
                            :color="getIssueColor(issue)"
                            size="small"
                            variant="tonal"
                            class="issue-chip"
                            @click.stop="showIssueFix(issue, audit)"
                          >
                            {{ getIssueLabel(issue) }}
                            <span
                              v-if="getArchiveOrgIdentifier(issue)"
                              class="ml-1 text-caption"
                            >
                              ({{ getArchiveOrgIdentifier(issue) }})
                            </span>
                          </v-chip>
                        </div>
                      </div>

                      <!-- Quality tiers -->
                      <div v-if="audit.availableTiers.length > 0" class="mb-3">
                        <div class="text-caption text-grey mb-2">Available Quality Tiers</div>
                        <div class="d-flex flex-wrap gap-1">
                          <v-chip
                            v-for="tier in audit.availableTiers"
                            :key="tier"
                            size="small"
                            color="success"
                            variant="outlined"
                          >
                            {{ tier }}
                          </v-chip>
                        </div>
                      </div>

                      <div v-if="audit.missingTiers.length > 0" class="mb-3">
                        <div class="text-caption text-grey mb-2">Missing Quality Tiers</div>
                        <div class="d-flex flex-wrap gap-1">
                          <v-chip
                            v-for="tier in audit.missingTiers"
                            :key="tier"
                            size="small"
                            color="warning"
                            variant="outlined"
                          >
                            {{ tier }}
                          </v-chip>
                        </div>
                      </div>

                      <!-- Metadata -->
                      <div class="text-caption text-grey">
                        <span v-if="audit.contentCid">CID: {{ audit.contentCid?.slice(0, 16) }}...</span>
                        <span v-if="audit.archiveOrgId" class="ml-3">
                          Archive.org: {{ audit.archiveOrgId }}
                        </span>
                      </div>
                    </v-card-text>
                    <v-card-actions>
                      <!-- Fix all button for this release -->
                      <v-btn
                        v-if="getAutoFixableIssues(audit).length > 0"
                        color="primary"
                        variant="flat"
                        size="small"
                        prepend-icon="$wrench"
                        @click="fixAllForRelease(audit)"
                      >
                        Fix all ({{ getAutoFixableIssues(audit).length }})
                      </v-btn>
                      <v-spacer></v-spacer>
                    </v-card-actions>
                  </div>
                </v-expand-transition>
              </v-card>
            </div>

            <!-- Empty state -->
            <v-sheet v-else class="text-center pa-8" color="transparent">
              <v-icon size="64" color="success" class="mb-4">$check-circle</v-icon>
              <p class="text-body-1 text-grey">
                {{ issueFilter ? 'No releases with this issue type' : 'All releases are healthy!' }}
              </p>
            </v-sheet>
          </template>
        </div>
      </div>

      <!-- ============================================================ -->
      <!-- SOURCES VIEW (HTTPS/S3/CID) -->
      <!-- ============================================================ -->
      <div v-else-if="currentView === 'sources'">
        <div class="d-flex align-center mb-4">
          <h4>HTTPS / S3 / CID Sources</h4>
        </div>

        <v-row>
          <!-- Import from URL -->
          <v-col cols="12" md="6">
            <v-card>
              <v-card-title>
                <v-icon class="mr-2">$link</v-icon>
                Import from URL
              </v-card-title>
              <v-card-text>
                <v-text-field
                  v-model="directUrl"
                  label="Direct URL"
                  placeholder="https://example.com/album.zip"
                  variant="outlined"
                  density="compact"
                  hide-details
                ></v-text-field>
              </v-card-text>
              <v-card-actions>
                <v-btn
                  color="primary"
                  :disabled="!directUrl"
                  :loading="sourceImportLoading"
                  @click="importFromUrl"
                >
                  Import
                </v-btn>
              </v-card-actions>
            </v-card>
          </v-col>

          <!-- Import from CID -->
          <v-col cols="12" md="6">
            <v-card>
              <v-card-title>
                <v-icon class="mr-2">$content-copy</v-icon>
                Import from CID
              </v-card-title>
              <v-card-text>
                <v-text-field
                  v-model="importCid"
                  label="IPFS or Archivist CID"
                  placeholder="Qm... or bafy... or zD..."
                  variant="outlined"
                  density="compact"
                  :error-messages="cidError"
                  hide-details="auto"
                ></v-text-field>
                <v-alert v-if="existingReleaseByCid" type="info" variant="tonal" class="mt-3" density="compact">
                  <div class="d-flex align-center">
                    <v-icon class="mr-2">$information</v-icon>
                    <div>
                      CID exists as release: <strong>{{ existingReleaseByCid.name }}</strong>
                      <br><span class="text-caption">Will re-archive and update release</span>
                    </div>
                  </div>
                </v-alert>
              </v-card-text>
              <v-card-actions>
                <v-btn
                  color="primary"
                  :disabled="!isValidCid"
                  :loading="sourceImportLoading"
                  @click="importFromCid"
                >
                  {{ existingReleaseByCid ? 'Re-archive' : 'Import' }}
                </v-btn>
              </v-card-actions>
            </v-card>
          </v-col>

          <!-- S3 Configuration -->
          <v-col cols="12" md="6">
            <v-card>
              <v-card-title>
                <v-icon class="mr-2">$cloud</v-icon>
                S3 Configuration
              </v-card-title>
              <v-card-text>
                Configure S3-compatible storage for bulk imports
              </v-card-text>
              <v-card-actions>
                <v-btn variant="outlined">Configure</v-btn>
              </v-card-actions>
            </v-card>
          </v-col>
        </v-row>
      </div>

      <!-- ============================================================ -->
      <!-- DIALOGS -->
      <!-- ============================================================ -->

      <!-- Item Details Dialog -->
      <v-dialog v-model="detailsDialog" max-width="800px">
        <v-card v-if="detailItem">
          <v-card-title class="d-flex align-center">
            <v-avatar size="48" class="mr-3">
              <v-img :src="getThumbnailUrl(detailItem.identifier)"></v-img>
            </v-avatar>
            <div>
              <span>{{ detailItem.title || detailItem.identifier }}</span>
              <div class="text-subtitle-2 text-grey">{{ detailItem.creator }}</div>
            </div>
            <v-spacer></v-spacer>
            <v-btn icon="$close" variant="text" @click="detailsDialog = false"></v-btn>
          </v-card-title>
          <v-divider></v-divider>
          <v-card-text>
            <p v-if="detailItem.description" class="mb-4">{{ detailItem.description }}</p>

            <h4 class="mb-2">Files</h4>
            <v-progress-linear v-if="itemDetails.isLoading.value" indeterminate color="primary"></v-progress-linear>
            <v-list v-else-if="itemDetails.data.value?.files" lines="two" max-height="300" class="overflow-auto">
              <v-list-item v-for="file in sortedAudioFiles" :key="file.name">
                <template #prepend>
                  <v-avatar v-if="file.track" size="32" color="primary" variant="tonal" class="mr-2">
                    <span class="text-caption">{{ file.track }}</span>
                  </v-avatar>
                  <v-icon v-else>$file-music</v-icon>
                </template>
                <v-list-item-title>{{ unescapeMetadata(file.title) || file.name.replace(/\.[^.]+$/, '') }}</v-list-item-title>
                <v-list-item-subtitle>
                  <span v-if="file.artist">{{ unescapeMetadata(file.artist) }}</span>
                  <span v-if="file.artist && file.length"> · </span>
                  <span v-if="file.length">{{ formatDuration(file.length) }}</span>
                  <span v-if="(file.artist || file.length) && file.format"> · </span>
                  <span>{{ file.format }}</span>
                  <span v-if="file.size"> · {{ formatSize(file.size) }}</span>
                </v-list-item-subtitle>
                <template #append>
                  <v-btn icon="$play" variant="text" size="small" @click="playFile(file)"></v-btn>
                </template>
              </v-list-item>
            </v-list>
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn
              color="primary"
              :disabled="isSelected(detailItem.identifier)"
              @click="addToSelection(detailItem); detailsDialog = false"
            >
              {{ isSelected(detailItem.identifier) ? 'Already Selected' : 'Add to Selection' }}
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>

      <!-- Job Details Dialog -->
      <v-dialog v-model="jobDetailsDialog" max-width="600px">
        <v-card v-if="selectedJob">
          <v-card-title class="d-flex align-center">
            <v-avatar :color="getJobStatusColor(selectedJob.status)" size="32" class="mr-3">
              <v-progress-circular
                v-if="selectedJob.status === 'Running'"
                indeterminate
                size="18"
                width="2"
                color="white"
              ></v-progress-circular>
              <v-icon v-else-if="selectedJob.status === 'Pending'" color="white" size="16">$clock-outline</v-icon>
              <v-icon v-else-if="selectedJob.status === 'Complete'" color="white" size="16">$check</v-icon>
              <v-icon v-else-if="selectedJob.status === 'Failed'" color="white" size="16">$alert-circle</v-icon>
            </v-avatar>
            <div>
              <span>{{ formatJobTarget(selectedJob.target) }}</span>
              <div class="text-subtitle-2 text-grey">{{ selectedJob.job_type }} job</div>
            </div>
            <v-spacer></v-spacer>
            <v-btn icon="$close" variant="text" @click="jobDetailsDialog = false"></v-btn>
          </v-card-title>
          <v-divider></v-divider>
          <v-card-text>
            <v-list density="compact" class="bg-transparent">
              <v-list-item>
                <template #prepend><v-icon size="small" class="mr-2">$identifier</v-icon></template>
                <v-list-item-title class="text-caption">Job ID</v-list-item-title>
                <v-list-item-subtitle><code>{{ selectedJob.id }}</code></v-list-item-subtitle>
              </v-list-item>
              <v-list-item>
                <template #prepend><v-icon size="small" class="mr-2">$clock-outline</v-icon></template>
                <v-list-item-title class="text-caption">Created</v-list-item-title>
                <v-list-item-subtitle>{{ formatTimestamp(selectedJob.created_at) }}</v-list-item-subtitle>
              </v-list-item>
              <v-list-item>
                <template #prepend><v-icon size="small" class="mr-2">$account-group</v-icon></template>
                <v-list-item-title class="text-caption">Claims</v-list-item-title>
                <v-list-item-subtitle>{{ selectedJob.claim_count }} node(s)</v-list-item-subtitle>
              </v-list-item>
              <v-list-item v-if="selectedJob.executor">
                <template #prepend><v-icon size="small" class="mr-2">$account</v-icon></template>
                <v-list-item-title class="text-caption">Executor</v-list-item-title>
                <v-list-item-subtitle><code>{{ selectedJob.executor }}</code></v-list-item-subtitle>
              </v-list-item>
            </v-list>

            <!-- Result -->
            <div v-if="selectedJob.result" class="mt-4">
              <v-alert
                v-if="getResultType(selectedJob.result) === 'Import'"
                type="success"
                variant="tonal"
              >
                <v-alert-title>Import Complete</v-alert-title>
                Imported {{ getResultData(selectedJob.result).files_imported }} files successfully.
              </v-alert>
              <v-alert
                v-else-if="getResultType(selectedJob.result) === 'Error'"
                type="error"
                variant="tonal"
              >
                <v-alert-title>Job Failed</v-alert-title>
                {{ getResultData(selectedJob.result) }}
              </v-alert>
              <v-alert
                v-else-if="getResultType(selectedJob.result) === 'Audit'"
                type="info"
                variant="tonal"
              >
                <v-alert-title>Audit Complete</v-alert-title>
                Audited {{ getResultData(selectedJob.result).total_releases }} releases.
                <div v-if="getResultData(selectedJob.result).releases_with_issues > 0">
                  {{ getResultData(selectedJob.result).releases_with_issues }} releases with issues detected.
                </div>
              </v-alert>
              <v-alert
                v-else-if="getResultType(selectedJob.result) === 'Transcode'"
                type="success"
                variant="tonal"
              >
                <v-alert-title>Transcode Complete</v-alert-title>
                Created {{ getResultData(selectedJob.result).outputs?.length || 0 }} quality variants.
              </v-alert>
              <v-alert
                v-else-if="getResultType(selectedJob.result) === 'SourceImport'"
                type="success"
                variant="tonal"
              >
                <v-alert-title>Source Import Complete</v-alert-title>
                <div>Size: {{ formatSize(String(getResultData(selectedJob.result).size || 0)) }}</div>
                <div v-if="getResultData(selectedJob.result).new_cid">New CID: <code>{{ getResultData(selectedJob.result).new_cid }}</code></div>
                <div v-if="getResultData(selectedJob.result).content_type">Type: {{ getResultData(selectedJob.result).content_type }}</div>
              </v-alert>
            </div>
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn
              v-if="selectedJob.status === 'Pending'"
              color="success"
              prepend-icon="$play"
              @click="startJob(selectedJob.id); jobDetailsDialog = false"
            >
              Start
            </v-btn>
            <v-btn
              v-if="selectedJob.status === 'Failed'"
              color="warning"
              prepend-icon="$refresh"
              @click="retryJob(selectedJob.id); jobDetailsDialog = false"
            >
              Retry
            </v-btn>
            <v-btn
              v-if="selectedJob.status === 'Running'"
              color="error"
              prepend-icon="$stop"
              @click="stopJob(selectedJob.id); jobDetailsDialog = false"
            >
              Stop
            </v-btn>
            <v-btn
              v-if="selectedJob.status === 'Completed' || selectedJob.status === 'Failed'"
              color="grey"
              variant="tonal"
              prepend-icon="$archive-outline"
              @click="archiveJob(selectedJob.id); jobDetailsDialog = false"
            >
              Archive
            </v-btn>
            <v-btn variant="text" @click="jobDetailsDialog = false">Close</v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>

      <!-- Create Job Dialog -->
      <v-dialog v-model="createJobDialog" max-width="500px">
        <v-card>
          <v-card-title>Create Job</v-card-title>
          <v-card-text>
            <v-select
              v-model="newJob.job_type"
              :items="['audit', 'transcode', 'migrate', 'import']"
              label="Job Type"
              variant="outlined"
              class="mb-4"
            ></v-select>
            <v-text-field
              v-model="newJob.target"
              label="Target"
              variant="outlined"
              hint="all, release:id, category:name, archive.org:id"
              persistent-hint
            ></v-text-field>
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn variant="text" @click="createJobDialog = false">Cancel</v-btn>
            <v-btn color="primary" @click="submitJob">Create</v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>

      <!-- Issue Fix Dialog -->
      <v-dialog v-model="issueFixDialog" max-width="500px">
        <v-card v-if="selectedIssue">
          <v-card-title class="d-flex align-center">
            <v-chip
              :color="getIssueInfo(selectedIssue.key).color"
              size="small"
              variant="flat"
              class="mr-3"
            >
              {{ getIssueInfo(selectedIssue.key).label }}
            </v-chip>
            <v-spacer></v-spacer>
            <v-btn icon="$close" variant="text" size="small" @click="issueFixDialog = false"></v-btn>
          </v-card-title>
          <v-divider></v-divider>
          <v-card-text>
            <!-- Release info -->
            <div class="mb-4">
              <div class="text-subtitle-2">{{ selectedIssue.audit?.title }}</div>
              <div class="text-caption text-grey">{{ selectedIssue.audit?.artist || 'Unknown artist' }}</div>
            </div>

            <!-- Issue description -->
            <v-alert
              :color="getIssueInfo(selectedIssue.key).color"
              variant="tonal"
              density="compact"
              class="mb-4"
            >
              {{ getIssueInfo(selectedIssue.key).description }}
            </v-alert>

            <!-- Fix suggestions -->
            <div class="text-subtitle-2 mb-2">How to fix</div>
            <v-list density="compact" class="bg-transparent">
              <v-list-item
                v-for="(suggestion, idx) in getIssueInfo(selectedIssue.key).fixSuggestions"
                :key="idx"
                class="px-0"
              >
                <template #prepend>
                  <v-icon size="small" color="grey" class="mr-2">$circle-small</v-icon>
                </template>
                <v-list-item-title class="text-body-2">{{ suggestion }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-card-text>
          <v-divider></v-divider>
          <v-card-actions>
            <v-spacer></v-spacer>
            <!-- Auto-fix buttons based on issue type -->
            <v-btn
              v-if="(selectedIssue.key === 'missing_audio_quality' || selectedIssue.key === 'missing_track_metadata') && selectedIssue.audit?.contentCid"
              color="primary"
              variant="flat"
              prepend-icon="$magnify"
              @click="runIssueFix('reingest_archivist')"
            >
              Analyze content
            </v-btn>
            <v-btn
              v-if="selectedIssue.key === 'not_on_archivist' && selectedIssue.audit?.contentCid"
              color="primary"
              variant="flat"
              prepend-icon="$upload"
              @click="runIssueFix('reingest_archivist')"
            >
              Ingest into Archivist
            </v-btn>
            <v-btn
              v-if="(selectedIssue.key === 'can_refetch_from_archive' || selectedIssue.archiveOrgId || selectedIssue.audit?.archiveOrgId)"
              color="success"
              variant="flat"
              prepend-icon="$refresh"
              @click="runIssueFix('refetch_archive_org')"
            >
              Refetch from Archive.org
            </v-btn>
            <v-btn
              v-if="selectedIssue.key === 'missing_opus_encodes'"
              color="info"
              variant="flat"
              prepend-icon="$tune"
              @click="runIssueFix('generate_opus')"
            >
              Generate Opus ladder
            </v-btn>
            <v-btn
              v-if="!getIssueInfo(selectedIssue.key).canAutoFix"
              variant="outlined"
              @click="issueFixDialog = false"
            >
              Close
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>

      <!-- Metadata Input Dialog (for NeedsInput jobs) -->
      <v-dialog v-model="metadataDialog" max-width="500px">
        <v-card>
          <v-card-title>Provide Metadata</v-card-title>
          <v-card-text>
            <p v-if="metadataDetected?.trackCount" class="text-body-2 mb-4">
              Detected {{ metadataDetected.trackCount }} tracks
              <span v-if="metadataDetected.detectedFormat">({{ metadataDetected.detectedFormat }})</span>
            </p>

            <v-text-field
              v-model="metadataForm.artist"
              label="Artist"
              :hint="metadataDetected?.artist ? `Detected: ${metadataDetected.artist}` : undefined"
              persistent-hint
              class="mb-2"
            />

            <v-text-field
              v-model="metadataForm.album"
              label="Album"
              :hint="metadataDetected?.album ? `Detected: ${metadataDetected.album}` : undefined"
              persistent-hint
              class="mb-2"
            />

            <v-text-field
              v-model.number="metadataForm.year"
              label="Year"
              type="number"
              :hint="metadataDetected?.year ? `Detected: ${metadataDetected.year}` : undefined"
              persistent-hint
              class="mb-2"
            />

            <v-alert
              v-if="metadataDetected?.hasEmbeddedCover || metadataDetected?.hasDirectoryCover"
              type="success"
              variant="tonal"
              density="compact"
              class="mt-2"
            >
              Cover art detected
              <span v-if="metadataDetected?.hasEmbeddedCover">(embedded)</span>
              <span v-if="metadataDetected?.hasDirectoryCover">(directory)</span>
            </v-alert>

            <v-alert
              v-if="metadataDetected?.archiveOrgHint"
              type="info"
              variant="tonal"
              density="compact"
              class="mt-2"
            >
              Archive.org hint: {{ metadataDetected.archiveOrgHint }}
            </v-alert>
          </v-card-text>
          <v-card-actions>
            <v-spacer />
            <v-btn variant="text" @click="metadataDialog = false">Cancel</v-btn>
            <v-btn
              color="primary"
              variant="flat"
              :disabled="!metadataForm.artist || !metadataForm.album"
              :loading="provideMetadataMutation.isPending.value"
              @click="submitMetadata"
            >
              Submit & Retry
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>

      <!-- Snackbar -->
      <v-snackbar v-model="snackbar" :color="snackbarColor">
        {{ snackbarText }}
      </v-snackbar>
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import {
  useLibrarianStatus,
  useLibrarianJobs,
  useArchiveSearch,
  useArchiveCollection,
  useArchiveItem,
  useCreateJob,
  useStartJob,
  useStopJob,
  useRetryJob,
  useArchiveJob,
  useClearArchivedJobs,
  useCreateSourceImport,
  useProvideMetadataMutation,
  fetchAllCollectionItems,
  getThumbnailUrl,
  getStreamUrl,
  getLibrarianApiUrl,
  type SearchResultItem,
  type ItemFile,
  type Job,
  type ReleaseAudit,
  type AuditIssue,
} from '/@/composables/useLibrarian';
import { cid as isValidCidCheck } from 'is-ipfs';
import { useAudioAlbum, type AudioTrack } from '/@/composables/audioAlbum';

// ============================================================================
// View State
// ============================================================================

type ViewType = 'main' | 'collections' | 'items' | 'selected' | 'queue' | 'quality' | 'sources';
const currentView = ref<ViewType>('main');

// ============================================================================
// API State
// ============================================================================

const apiUrl = getLibrarianApiUrl();
const status = useLibrarianStatus();
const jobs = useLibrarianJobs();
const createJobMutation = useCreateJob();
const startJobMutation = useStartJob();
const stopJobMutation = useStopJob();
const retryJobMutation = useRetryJob();
const archiveJobMutation = useArchiveJob();
const clearArchivedMutation = useClearArchivedJobs();
const sourceImportMutation = useCreateSourceImport();
const provideMetadataMutation = useProvideMetadataMutation();

// Collection/Search
const collectionId = ref('');
const collectionIdQuery = ref('');
const collectionSearchQuery = ref('');
const collectionSearchExecuted = ref('');
const searchQuery = ref('');
const searchQueryExecuted = ref('');
const currentPage = ref(1);
const collectionSearchPage = ref(1);

// View mode: 'endless' (default) or 'paged'
const viewMode = ref<'endless' | 'paged'>('endless');

// Accumulated items for endless scroll
const accumulatedItems = ref<SearchResultItem[]>([]);
const isLoadingMore = ref(false);
const ITEMS_PER_PAGE = 48;

const collection = useArchiveCollection(collectionIdQuery, { page: currentPage, rows: ITEMS_PER_PAGE });
const searchResults = useArchiveSearch(searchQueryExecuted, { page: currentPage, rows: ITEMS_PER_PAGE });

// Collection search (searches for collections, not items)
const collectionSearchQueryWithFilter = computed(() =>
  collectionSearchExecuted.value ? `mediatype:collection AND (${collectionSearchExecuted.value})` : ''
);
const collectionSearchResults = useArchiveSearch(collectionSearchQueryWithFilter, { page: collectionSearchPage, rows: 24 });

// Item details
const detailItemId = ref('');
const itemDetails = useArchiveItem(detailItemId);

// ============================================================================
// Selection State
// ============================================================================

const selectedItems = ref<SearchResultItem[]>([]);
const autoApprove = ref(false);
const transcodeFlac = ref(true);

// "Select All in Collection" state - just track the flag and count, don't load all items
const allInCollectionSelected = ref(false);
const allInCollectionId = ref('');
const allInCollectionCount = ref(0);

// ============================================================================
// Dialog State
// ============================================================================

const detailsDialog = ref(false);
const detailItem = ref<SearchResultItem | null>(null);
const createJobDialog = ref(false);
const newJob = ref({ job_type: 'import', target: '' });

// ============================================================================
// Job Expansion State
// ============================================================================

const expandedJobs = ref<Set<string>>(new Set());
const jobDetailsDialog = ref(false);
const selectedJob = ref<Job | null>(null);

// ============================================================================
// Audit Results State
// ============================================================================

const auditResults = ref<{
  totalReleases: number;
  releasesWithIssues: number;
  audits: ReleaseAudit[];
  issueCounts: Record<string, number>;
} | null>(null);
const qualitySubView = ref<'overview' | 'issues'>('overview');
const expandedAudits = ref<Set<string>>(new Set());
const issueFilter = ref<string | null>(null);

// Issue fix dialog state
const issueFixDialog = ref(false);
const selectedIssue = ref<{
  key: string;
  audit: ReleaseAudit | null;
  archiveOrgId: string | null;
} | null>(null);

// Metadata input dialog state (for NeedsInput jobs)
const metadataDialog = ref(false);
const metadataJobId = ref<string | null>(null);
const metadataForm = ref({
  artist: '',
  album: '',
  year: null as number | null,
});
const metadataDetected = ref<{
  artist?: string;
  album?: string;
  year?: number;
  trackCount?: number;
  detectedFormat?: string;
  hasEmbeddedCover?: boolean;
  hasDirectoryCover?: boolean;
  archiveOrgHint?: string;
} | null>(null);

// ============================================================================
// Audio Player (global Flagship player)
// ============================================================================

const { albumFiles, handlePlay } = useAudioAlbum();

// ============================================================================
// HTTPS/S3/CID State
// ============================================================================

const directUrl = ref('');
const importCid = ref('');
const cidError = ref('');
const sourceImportLoading = ref(false);
const existingReleaseByCid = ref<{ id: string; name: string } | null>(null);

// CID validation
const isValidCid = computed(() => {
  if (!importCid.value) return false;
  const cid = importCid.value.trim();
  // Check for IPFS CIDs (Qm... or bafy...) or Archivist CIDs (zD... or zE...)
  if (cid.startsWith('Qm') && cid.length === 46) return true;
  if (cid.startsWith('bafy')) return true;
  if (cid.startsWith('zD') || cid.startsWith('zE')) return true;
  // Use is-ipfs for more comprehensive check
  return isValidCidCheck(cid);
});

// ============================================================================
// Snackbar
// ============================================================================

const snackbar = ref(false);
const snackbarText = ref('');
const snackbarColor = ref('success');

// ============================================================================
// Computed
// ============================================================================

const currentViewTitle = computed(() => {
  switch (currentView.value) {
    case 'main': return 'Librarian';
    case 'collections': return 'Collections';
    case 'items': return 'Items';
    case 'selected': return 'Selected';
    case 'queue': return 'Queue';
    case 'quality': return 'Quality';
    case 'sources': return 'HTTPS/S3/CID';
    default: return 'Librarian';
  }
});

const breadcrumbs = computed(() => {
  if (currentView.value === 'main') return [];
  return [
    { title: 'Librarian', value: 'main' },
    { title: currentViewTitle.value, value: currentView.value },
  ];
});

const pendingJobCount = computed(() =>
  jobs.data.value?.filter((j: any) => j.status === 'Pending').length ?? 0
);

const runningJobCount = computed(() =>
  jobs.data.value?.filter((j: any) => j.status === 'Running').length ?? 0
);

// Current page items (from API response)
const currentPageItems = computed(() => {
  if (collection.data.value?.items?.length) return collection.data.value.items;
  if (searchResults.data.value?.items?.length) return searchResults.data.value.items;
  return [];
});

// Total results count from API
const totalResults = computed(() => {
  if (collection.data.value?.total) return collection.data.value.total;
  if (searchResults.data.value?.total) return searchResults.data.value.total;
  return 0;
});

// Total pages for paged mode
const totalPages = computed(() => {
  return Math.ceil(totalResults.value / ITEMS_PER_PAGE);
});

// Has more items to load (for endless scroll)
const hasMoreItems = computed(() => {
  return accumulatedItems.value.length < totalResults.value;
});

// Total selection count (either individual items or "all in collection")
const totalSelectionCount = computed(() => {
  if (allInCollectionSelected.value) {
    return allInCollectionCount.value;
  }
  return selectedItems.value.length;
});

// Visible items based on view mode
const visibleItems = computed(() => {
  if (viewMode.value === 'endless') {
    return accumulatedItems.value;
  } else {
    // Paged mode shows only current page items
    return currentPageItems.value;
  }
});

const sortedJobs = computed(() => {
  if (!jobs.data.value) return [];
  return [...jobs.data.value]
    .filter(j => !j.archived)
    .sort((a, b) => {
      const statusOrder: Record<string, number> = { Running: 0, Pending: 1, Complete: 2, Failed: 3 };
      const aOrder = statusOrder[a.status] ?? 99;
      const bOrder = statusOrder[b.status] ?? 99;
      if (aOrder !== bOrder) return aOrder - bOrder;
      return b.created_at - a.created_at;
    });
});

const archivedJobs = computed(() => {
  if (!jobs.data.value) return [];
  return jobs.data.value.filter(j => j.archived);
});

// Filtered audits based on issue filter
const filteredAudits = computed(() => {
  if (!auditResults.value?.audits) return [];
  if (!issueFilter.value) return auditResults.value.audits;
  return auditResults.value.audits.filter(audit =>
    audit.issues.some(issue => getIssueKey(issue) === issueFilter.value)
  );
});

// Issue metadata: labels, colors, descriptions, and fix suggestions
const issueInfo: Record<string, {
  label: string;
  color: string;
  description: string;
  canAutoFix: boolean;
  fixSuggestions: string[];
}> = {
  missing_audio_quality: {
    label: 'No audio quality info',
    color: 'warning',
    description: 'The audio format, bitrate, and codec are unknown.',
    canAutoFix: true,
    fixSuggestions: [
      'Re-analyze the content to detect audio quality automatically',
    ],
  },
  missing_license: {
    label: 'No license',
    color: 'orange',
    description: 'No license information specified for this release.',
    canAutoFix: false,
    fixSuggestions: [
      'Add a Creative Commons or other license in Citadel Lens',
      'Mark as "All Rights Reserved" if applicable',
      'Check the original source for license info',
    ],
  },
  missing_source: {
    label: 'No source URL',
    color: 'orange',
    description: 'The original source of this content is not documented.',
    canAutoFix: false,
    fixSuggestions: [
      'Add the original source URL (Archive.org, Bandcamp, etc.)',
      'Mark as "Self" if you created this content',
      'Mark as "Unknown" if the source cannot be determined',
    ],
  },
  missing_opus_encodes: {
    label: 'No Opus ladder',
    color: 'info',
    description: 'This release has lossless audio but no Opus transcodes for streaming.',
    canAutoFix: true,
    fixSuggestions: [
      'Generate Opus quality ladder (192, 160, 128, 96, 64 kbps)',
      'This enables adaptive streaming for different bandwidths',
    ],
  },
  missing_cover_art: {
    label: 'No cover art',
    color: 'warning',
    description: 'This release has no thumbnail or cover image.',
    canAutoFix: true,
    fixSuggestions: [
      'Upload cover art in Citadel Lens',
      'Re-fetch from Archive.org if available there',
      'Extract embedded album art from audio files',
    ],
  },
  unused_track_art: {
    label: 'Unused track art',
    color: 'grey',
    description: 'Audio files contain embedded artwork that could be displayed.',
    canAutoFix: true,
    fixSuggestions: [
      'Extract and use embedded track artwork',
      'This is informational - not a critical issue',
    ],
  },
  missing_description: {
    label: 'No description',
    color: 'grey',
    description: 'This release has no description text.',
    canAutoFix: true,
    fixSuggestions: [
      'Add a description in Citadel Lens',
      'Re-fetch from Archive.org to get description',
      'Copy from the original source if available',
    ],
  },
  missing_year: {
    label: 'No release year',
    color: 'grey',
    description: 'The release year is not set.',
    canAutoFix: true,
    fixSuggestions: [
      'Set the release year in Citadel Lens',
      'Re-fetch from Archive.org to get date metadata',
      'Check audio file tags for year information',
    ],
  },
  missing_credits: {
    label: 'No credits',
    color: 'grey',
    description: 'No artist credits or attribution information.',
    canAutoFix: false,
    fixSuggestions: [
      'Add credits and attribution in Citadel Lens',
      'Required for proper licensing compliance',
    ],
  },
  no_audio_files: {
    label: 'No audio files',
    color: 'error',
    description: 'This release contains no playable audio files.',
    canAutoFix: false,
    fixSuggestions: [
      'Check if content was uploaded correctly',
      'Re-import the release from source',
      'This may be a broken or incomplete upload',
    ],
  },
  not_on_archivist: {
    label: 'Not on Archivist',
    color: 'error',
    description: 'Content is on IPFS but not archived in Archivist for permanent storage.',
    canAutoFix: true,
    fixSuggestions: [
      'Re-ingest into Archivist from the IPFS CID',
      'This ensures content is permanently preserved',
      'IPFS content may become unavailable without pinning',
    ],
  },
  invalid_content_cid: {
    label: 'Invalid content ID',
    color: 'error',
    description: 'The content CID is missing or invalid.',
    canAutoFix: false,
    fixSuggestions: [
      'Re-upload the content to get a valid CID',
      'Check if the release was created correctly',
      'This release may need to be recreated',
    ],
  },
  missing_track_metadata: {
    label: 'No track info',
    color: 'warning',
    description: 'Track titles and metadata are not available.',
    canAutoFix: true,
    fixSuggestions: [
      'Re-analyze content to extract track metadata from audio files',
    ],
  },
  can_refetch_from_archive: {
    label: 'Can refetch',
    color: 'success',
    description: 'This Archive.org import can be re-fetched for better metadata.',
    canAutoFix: true,
    fixSuggestions: [
      'Re-fetch from Archive.org to update metadata',
      'May get better cover art, description, or year',
      'Will not affect existing audio content',
    ],
  },
};

// Helper accessors for templates
const issueLabels = Object.fromEntries(
  Object.entries(issueInfo).map(([k, v]) => [k, v.label])
);
const issueColors = Object.fromEntries(
  Object.entries(issueInfo).map(([k, v]) => [k, v.color])
);

const itemAudioFiles = computed(() => {
  if (!itemDetails.data.value?.files) return [];
  return itemDetails.data.value.files.filter((f: ItemFile) => {
    const format = f.format?.toLowerCase() || '';
    return format.includes('flac') || format.includes('mp3') || format.includes('ogg') || format.includes('wav') || format.includes('opus');
  });
});

// Deduplicate by title (keep longest duration) and sort by track number
const sortedAudioFiles = computed(() => {
  const files = itemAudioFiles.value;

  // Group by normalized title (unescaped, or filename if no title)
  const byTitle = new Map<string, ItemFile>();
  for (const file of files) {
    const title = unescapeMetadata(file.title) || file.name.replace(/\.[^.]+$/, '');
    const existing = byTitle.get(title);

    if (!existing) {
      byTitle.set(title, file);
    } else {
      // Keep the one with longer duration
      const existingLength = existing.length ? parseFloat(existing.length) : 0;
      const newLength = file.length ? parseFloat(file.length) : 0;
      if (newLength > existingLength) {
        byTitle.set(title, file);
      }
    }
  }

  // Sort by track number, then by name
  return [...byTitle.values()].sort((a, b) => {
    const trackA = a.track ? parseInt(a.track, 10) : Infinity;
    const trackB = b.track ? parseInt(b.track, 10) : Infinity;
    if (trackA !== trackB) return trackA - trackB;
    return a.name.localeCompare(b.name);
  });
});


// ============================================================================
// Watchers: Accumulate items for endless scroll
// ============================================================================

// Watch for new data from API and accumulate items
watch(currentPageItems, (newItems) => {
  if (newItems.length > 0) {
    if (currentPage.value === 1) {
      // First page - replace accumulated items
      accumulatedItems.value = [...newItems];
    } else {
      // Subsequent pages - append items (avoiding duplicates)
      const existingIds = new Set(accumulatedItems.value.map(i => i.identifier));
      const uniqueNewItems = newItems.filter((i: SearchResultItem) => !existingIds.has(i.identifier));
      accumulatedItems.value = [...accumulatedItems.value, ...uniqueNewItems];
    }
    isLoadingMore.value = false;
  }
});

// ============================================================================
// Methods
// ============================================================================

function navigateToBreadcrumb(item: any) {
  if (item.value) {
    currentView.value = item.value as ViewType;
  }
}

function searchCollections() {
  if (collectionSearchQuery.value.trim()) {
    collectionSearchPage.value = 1;
    collectionSearchExecuted.value = collectionSearchQuery.value.trim();
    // Clear any loaded collection items
    accumulatedItems.value = [];
    collectionIdQuery.value = '';
  }
}

function loadCollection() {
  if (collectionId.value.trim()) {
    // Reset state for new collection
    accumulatedItems.value = [];
    currentPage.value = 1;
    collectionIdQuery.value = collectionId.value.trim();
    searchQueryExecuted.value = '';
    // Clear collection search
    collectionSearchExecuted.value = '';
  }
}

function selectCollection(collectionIdentifier: string) {
  collectionId.value = collectionIdentifier;
  loadCollection();
}

function executeSearch() {
  if (searchQuery.value.trim()) {
    // Reset state for new search
    accumulatedItems.value = [];
    currentPage.value = 1;
    searchQueryExecuted.value = searchQuery.value.trim();
    collectionIdQuery.value = '';
  }
}

function isSelected(identifier: string): boolean {
  return selectedItems.value.some(item => item.identifier === identifier);
}

function toggleSelection(item: SearchResultItem) {
  const index = selectedItems.value.findIndex(i => i.identifier === item.identifier);
  if (index === -1) {
    selectedItems.value.push(item);
  } else {
    selectedItems.value.splice(index, 1);
  }
}

function addToSelection(item: SearchResultItem) {
  if (!isSelected(item.identifier)) {
    selectedItems.value.push(item);
  }
}

function removeFromSelection(item: SearchResultItem) {
  const index = selectedItems.value.findIndex(i => i.identifier === item.identifier);
  if (index !== -1) {
    selectedItems.value.splice(index, 1);
  }
}

function selectAllOnPage() {
  // Select all items currently visible on the page
  for (const item of visibleItems.value) {
    if (!isSelected(item.identifier)) {
      selectedItems.value.push(item);
    }
  }
}

function selectAllInCollection() {
  // Just set the flag - don't load all items into DOM
  const currentCollectionId = collectionIdQuery.value;
  if (!currentCollectionId || totalResults.value === 0) {
    return;
  }

  // Mark all in collection as selected
  allInCollectionSelected.value = true;
  allInCollectionId.value = currentCollectionId;
  allInCollectionCount.value = totalResults.value;

  // Also mark any already-loaded items as selected (for visual feedback)
  for (const item of accumulatedItems.value) {
    if (!isSelected(item.identifier)) {
      selectedItems.value.push(item);
    }
  }
}

function clearSelection() {
  selectedItems.value = [];
  allInCollectionSelected.value = false;
  allInCollectionId.value = '';
  allInCollectionCount.value = 0;
}

// Endless scroll: intersection observer callback
function onIntersect(isIntersecting: boolean) {
  if (isIntersecting && hasMoreItems.value && !isLoadingMore.value) {
    loadMoreItems();
  }
}

function loadMoreItems() {
  isLoadingMore.value = true;
  currentPage.value++;
}

// Paged mode navigation
function goToPreviousPage() {
  if (currentPage.value > 1) {
    currentPage.value--;
  }
}

function goToNextPage() {
  if (currentPage.value < totalPages.value) {
    currentPage.value++;
  }
}

function viewItemDetails(item: SearchResultItem) {
  detailItem.value = item;
  detailItemId.value = item.identifier;
  detailsDialog.value = true;
}

// Track current preview to prevent race conditions
let currentPreviewId = '';

function previewItem(item: SearchResultItem) {
  // Set the current preview ID to track which item we're loading
  currentPreviewId = item.identifier;
  detailItemId.value = item.identifier;

  // Wait for item details to load, then set up tracks in global player
  const stopWatch = watch(() => itemDetails.data.value, (data) => {
    // Check if this is still the item we want to preview (prevents race condition)
    if (currentPreviewId !== item.identifier) {
      stopWatch();
      return;
    }

    if (data?.files && data.identifier === item.identifier) {
      const audioTracks = sortedAudioFiles.value;
      if (audioTracks.length > 0) {
        // Convert Archive.org files to AudioTrack format and play
        loadTracksAndPlay(item, audioTracks, 0);
        stopWatch(); // Unsubscribe after loading
      }
    }
  }, { immediate: true });
}

function playFile(file: ItemFile) {
  // Find the index of this file in sortedAudioFiles and play it
  const index = sortedAudioFiles.value.findIndex((f: ItemFile) => f.name === file.name);
  if (detailItem.value) {
    loadTracksAndPlay(detailItem.value, sortedAudioFiles.value, index >= 0 ? index : 0);
  }
}

function loadTracksAndPlay(item: SearchResultItem, files: ItemFile[], startIndex: number) {
  // Sort files by track number for proper playlist order
  const sortedFiles = [...files].sort((a, b) => {
    const trackA = a.track ? parseInt(a.track, 10) : Infinity;
    const trackB = b.track ? parseInt(b.track, 10) : Infinity;
    if (trackA !== trackB) return trackA - trackB;
    return a.name.localeCompare(b.name);
  });

  // Convert Archive.org files to AudioTrack format for global player
  // Use rich metadata from ID3/Vorbis tags when available
  // Unescape any backslash-escaped chars from Archive.org metadata
  const tracks: AudioTrack[] = sortedFiles.map((file, index) => ({
    index,
    cid: getStreamUrl(item.identifier, file.name), // URL passed as "cid" - parseUrlOrCid passes URLs through
    title: unescapeMetadata(file.title) || file.name.replace(/\.[^.]+$/, ''), // Use tag title or filename
    album: unescapeMetadata(file.album) || item.title || item.identifier,
    artist: unescapeMetadata(file.artist) || item.creator || 'Unknown',
    duration: file.length ? formatDuration(file.length) : undefined,
    size: file.size || undefined,
  }));

  // Set the global player's album files and start playback
  albumFiles.value = tracks;
  handlePlay(startIndex);
}

function startImport() {
  if (allInCollectionSelected.value) {
    // Create a single job for the entire collection - server handles enumeration
    createJobMutation.mutate({
      job_type: 'import',
      target: `archive.org:collection:${allInCollectionId.value}`,
    });
    snackbarText.value = `Created import job for collection "${allInCollectionId.value}" (${allInCollectionCount.value} items)`;
  } else {
    // Create import jobs for each individually selected item
    for (const item of selectedItems.value) {
      createJobMutation.mutate({
        job_type: 'import',
        target: `archive.org:${item.identifier}`,
      });
    }
    snackbarText.value = `Created ${selectedItems.value.length} import jobs`;
  }
  snackbarColor.value = 'success';
  snackbar.value = true;
  clearSelection();
  currentView.value = 'queue';
}

function startAudit() {
  createJobMutation.mutate({ job_type: 'audit', target: 'all' });
  snackbarText.value = 'Audit job created';
  snackbarColor.value = 'success';
  snackbar.value = true;
  currentView.value = 'queue';
}

// Watch for completed audit jobs and extract results
// Note: Rust serde uses externally-tagged enums: { "Audit": { ... } } not { "type": "Audit", ... }
watch(() => jobs.data.value, (jobList) => {
  if (!jobList) return;

  // Find the most recent completed audit job with results
  // Check for both externally-tagged { Audit: {...} } and internally-tagged { type: 'Audit', ... }
  const auditJob = jobList
    .filter(j => {
      if (j.job_type !== 'audit' || !j.result) return false;
      // Handle externally-tagged enum from Rust serde
      const auditData = (j.result as any).Audit || (j.result as any);
      return auditData?.audits !== undefined;
    })
    .sort((a, b) => b.created_at - a.created_at)[0];

  if (auditJob?.result) {
    // Extract data from externally-tagged format
    const auditData = (auditJob.result as any).Audit || auditJob.result;
    if (auditData?.audits) {
      auditResults.value = {
        totalReleases: auditData.total_releases || 0,
        releasesWithIssues: auditData.releases_with_issues || 0,
        audits: auditData.audits,
        issueCounts: auditData.issue_counts || {},
      };
    }
  }
}, { immediate: true });

async function submitJob() {
  if (!newJob.value.target.trim()) {
    snackbarText.value = 'Target is required';
    snackbarColor.value = 'error';
    snackbar.value = true;
    return;
  }

  try {
    await createJobMutation.mutateAsync(newJob.value);
    snackbarText.value = 'Job created';
    snackbarColor.value = 'success';
    snackbar.value = true;
    createJobDialog.value = false;
    newJob.value = { job_type: 'import', target: '' };
  } catch (error: any) {
    snackbarText.value = `Failed: ${error.message}`;
    snackbarColor.value = 'error';
    snackbar.value = true;
  }
}

function getJobStatusColor(status: string): string {
  switch (status) {
    case 'Running': return 'success';
    case 'Pending': return 'warning';
    case 'Complete': return 'info';
    case 'Failed': return 'error';
    default: return 'grey';
  }
}

function toggleJobExpand(jobId: string) {
  if (expandedJobs.value.has(jobId)) {
    expandedJobs.value.delete(jobId);
  } else {
    expandedJobs.value.add(jobId);
  }
  // Force reactivity
  expandedJobs.value = new Set(expandedJobs.value);
}

function formatJobTarget(target: string): string {
  // Pretty-print job targets
  if (target.startsWith('archive.org:')) {
    const identifier = target.replace('archive.org:', '');
    if (identifier.startsWith('collection:')) {
      return `Collection: ${identifier.replace('collection:', '')}`;
    }
    return `Archive.org: ${identifier}`;
  }
  if (target.startsWith('release:')) {
    return `Release: ${target.replace('release:', '')}`;
  }
  if (target.startsWith('category:')) {
    return `Category: ${target.replace('category:', '')}`;
  }
  if (target.startsWith('source:')) {
    const source = target.replace('source:', '');
    // Truncate long URLs/CIDs
    if (source.length > 40) {
      return `Source: ${source.slice(0, 37)}...`;
    }
    return `Source: ${source}`;
  }
  if (target === 'all') {
    return 'All releases';
  }
  return target;
}

function showJobDetails(job: Job) {
  selectedJob.value = job;
  jobDetailsDialog.value = true;
}

function startJob(jobId: string) {
  startJobMutation.mutate(jobId);
  snackbarText.value = 'Job started';
  snackbarColor.value = 'success';
  snackbar.value = true;
}

function retryJob(jobId: string) {
  retryJobMutation.mutate(jobId);
  snackbarText.value = 'Job retrying';
  snackbarColor.value = 'success';
  snackbar.value = true;
}

function stopJob(jobId: string) {
  stopJobMutation.mutate(jobId);
  snackbarText.value = 'Job cancelled';
  snackbarColor.value = 'warning';
  snackbar.value = true;
}

function archiveJob(jobId: string) {
  archiveJobMutation.mutate(jobId);
  snackbarText.value = 'Job archived';
  snackbarColor.value = 'info';
  snackbar.value = true;
}

function openMetadataDialog(job: Job) {
  metadataJobId.value = job.id;

  // Extract detected metadata from the NeedsInput result
  if (job.result && job.result.type === 'NeedsInput' && job.result.detected) {
    const detected = job.result.detected;

    // Pre-fill form with detected values
    metadataForm.value = {
      artist: detected.artist || '',
      album: detected.album || '',
      year: detected.year || null,
    };

    // Store detected info for display
    metadataDetected.value = {
      artist: detected.artist,
      album: detected.album,
      year: detected.year,
      trackCount: detected.trackCount,
      detectedFormat: detected.detectedFormat,
      hasEmbeddedCover: detected.hasEmbeddedCover,
      hasDirectoryCover: detected.hasDirectoryCover,
      archiveOrgHint: job.result.archive_org_hint,
    };
  } else {
    // Reset to empty state
    metadataForm.value = { artist: '', album: '', year: null };
    metadataDetected.value = null;
  }

  metadataDialog.value = true;
}

function submitMetadata() {
  if (!metadataJobId.value) return;

  provideMetadataMutation.mutate({
    jobId: metadataJobId.value,
    metadata: {
      artist: metadataForm.value.artist,
      album: metadataForm.value.album,
      year: metadataForm.value.year || undefined,
      trackTitles: [], // TODO: add track title editing
    },
  }, {
    onSuccess: () => {
      metadataDialog.value = false;
      metadataJobId.value = null;
      snackbarText.value = 'Metadata provided - job will retry';
      snackbarColor.value = 'success';
      snackbar.value = true;
    },
    onError: (error) => {
      snackbarText.value = `Failed to provide metadata: ${error.message}`;
      snackbarColor.value = 'error';
      snackbar.value = true;
    },
  });
}

// ============================================================================
// Audit Helper Functions
// ============================================================================

function getIssueKey(issue: AuditIssue): string {
  if (typeof issue === 'string') return issue;
  if ('can_refetch_from_archive' in issue) return 'can_refetch_from_archive';
  return 'unknown';
}

function getIssueLabel(issue: AuditIssue): string {
  const key = getIssueKey(issue);
  return issueLabels[key] || key;
}

function getIssueColor(issue: AuditIssue): string {
  const key = getIssueKey(issue);
  return issueColors[key] || 'grey';
}

function getArchiveOrgIdentifier(issue: AuditIssue): string | null {
  if (typeof issue === 'object' && 'can_refetch_from_archive' in issue) {
    return issue.can_refetch_from_archive.identifier;
  }
  return null;
}

// Check if CID is already on Archivist (zD or zE prefix)
function isArchivistCid(cid: string | null | undefined): boolean {
  if (!cid) return false;
  return cid.startsWith('zD') || cid.startsWith('zE');
}

function toggleAuditExpand(releaseId: string) {
  if (expandedAudits.value.has(releaseId)) {
    expandedAudits.value.delete(releaseId);
  } else {
    expandedAudits.value.add(releaseId);
  }
  expandedAudits.value = new Set(expandedAudits.value);
}

function filterByIssue(issueKey: string | null) {
  issueFilter.value = issueKey;
}

function showIssueFix(issue: AuditIssue, audit: ReleaseAudit) {
  const key = getIssueKey(issue);
  const archiveOrgId = getArchiveOrgIdentifier(issue);
  selectedIssue.value = { key, audit, archiveOrgId };
  issueFixDialog.value = true;
}

function getIssueInfo(key: string) {
  return issueInfo[key] || {
    label: key,
    color: 'grey',
    description: 'Unknown issue type.',
    canAutoFix: false,
    fixSuggestions: ['Contact support for assistance.'],
  };
}

// Get auto-fixable issues for a release
function getAutoFixableIssues(audit: ReleaseAudit): AuditIssue[] {
  return audit.issues.filter(issue => {
    const key = getIssueKey(issue);
    const info = issueInfo[key];
    if (!info?.canAutoFix) return false;

    // Check if we have the prerequisites to fix
    switch (key) {
      case 'missing_audio_quality':
      case 'missing_track_metadata':
      case 'not_on_archivist':
        return !!audit.contentCid;
      case 'can_refetch_from_archive':
      case 'missing_description':
      case 'missing_year':
      case 'missing_cover_art':
        return !!audit.archiveOrgId || !!getArchiveOrgIdentifier(issue);
      case 'missing_opus_encodes':
        return !!audit.releaseId;
      default:
        return false;
    }
  });
}

// Get total auto-fixable count across all releases
function getTotalAutoFixableCount(): number {
  if (!auditResults.value?.audits) return 0;
  return auditResults.value.audits.reduce((total, audit) => {
    return total + getAutoFixableIssues(audit).length;
  }, 0);
}

// Fix all auto-fixable issues for a single release
async function fixAllForRelease(audit: ReleaseAudit) {
  const fixable = getAutoFixableIssues(audit);
  let fixCount = 0;
  let didAnalyze = false;
  let didSourceImport = false;
  let didArchiveRefetch = false;

  for (const issue of fixable) {
    const key = getIssueKey(issue);

    // Metadata-only fixes: Use 'analyze' if content is already on Archivist (efficient, no re-upload)
    if ((key === 'missing_audio_quality' || key === 'missing_track_metadata')
        && audit.contentCid && isArchivistCid(audit.contentCid) && !didAnalyze) {
      createJobMutation.mutate({
        job_type: 'analyze',
        target: `source:${audit.contentCid}`,
      });
      fixCount++;
      didAnalyze = true; // Only one analyze per release
    }
    // Not on Archivist: Use 'source_import' to migrate from IPFS to Archivist
    else if (key === 'not_on_archivist' && audit.contentCid && !didSourceImport) {
      createJobMutation.mutate({
        job_type: 'source_import',
        target: `source:${audit.contentCid}`,
      });
      fixCount++;
      didSourceImport = true; // Only one source import per release
    }
    // Archive.org refetch fixes: can_refetch, missing_description, missing_year, missing_cover_art
    else if ((key === 'can_refetch_from_archive' || key === 'missing_description' ||
              key === 'missing_year' || key === 'missing_cover_art') &&
             (audit.archiveOrgId || getArchiveOrgIdentifier(issue)) && !didArchiveRefetch) {
      const archiveId = getArchiveOrgIdentifier(issue) || audit.archiveOrgId;
      createJobMutation.mutate({
        job_type: 'import',
        target: `archive.org:${archiveId}`,
      });
      fixCount++;
      didArchiveRefetch = true; // Only one refetch per release
    }
    // Transcode fixes: missing_opus_encodes
    else if (key === 'missing_opus_encodes' && audit.releaseId) {
      createJobMutation.mutate({
        job_type: 'transcode',
        target: `release:${audit.releaseId}`,
      });
      fixCount++;
    }
  }

  if (fixCount > 0) {
    snackbarText.value = `Created ${fixCount} fix job(s) for "${audit.title}"`;
    snackbarColor.value = 'success';
    snackbar.value = true;
  }
}

// Fix all auto-fixable issues across all releases
async function fixAllReleases() {
  if (!auditResults.value?.audits) return;

  let totalJobs = 0;
  const processedAnalyzeCids = new Set<string>(); // For analyze jobs (Archivist CIDs)
  const processedSourceCids = new Set<string>(); // For source_import jobs (IPFS CIDs)
  const processedArchiveIds = new Set<string>();
  const processedTranscodes = new Set<string>();

  for (const audit of auditResults.value.audits) {
    const fixable = getAutoFixableIssues(audit);

    for (const issue of fixable) {
      const key = getIssueKey(issue);

      // Metadata-only fixes: Use 'analyze' for Archivist CIDs (efficient, no re-upload)
      if ((key === 'missing_audio_quality' || key === 'missing_track_metadata')
          && audit.contentCid && isArchivistCid(audit.contentCid)
          && !processedAnalyzeCids.has(audit.contentCid)) {
        processedAnalyzeCids.add(audit.contentCid);
        createJobMutation.mutate({
          job_type: 'analyze',
          target: `source:${audit.contentCid}`,
        });
        totalJobs++;
      }
      // Not on Archivist: Use 'source_import' to migrate from IPFS (dedupe by CID)
      else if (key === 'not_on_archivist' && audit.contentCid
               && !processedSourceCids.has(audit.contentCid)) {
        processedSourceCids.add(audit.contentCid);
        createJobMutation.mutate({
          job_type: 'source_import',
          target: `source:${audit.contentCid}`,
        });
        totalJobs++;
      }
      // Archive.org refetch fixes (dedupe by archive ID)
      else if ((key === 'can_refetch_from_archive' || key === 'missing_description' ||
                key === 'missing_year' || key === 'missing_cover_art')) {
        const archiveId = getArchiveOrgIdentifier(issue) || audit.archiveOrgId;
        if (archiveId && !processedArchiveIds.has(archiveId)) {
          processedArchiveIds.add(archiveId);
          createJobMutation.mutate({
            job_type: 'import',
            target: `archive.org:${archiveId}`,
          });
          totalJobs++;
        }
      }
      // Transcode fixes (dedupe by release ID)
      else if (key === 'missing_opus_encodes' && audit.releaseId && !processedTranscodes.has(audit.releaseId)) {
        processedTranscodes.add(audit.releaseId);
        createJobMutation.mutate({
          job_type: 'transcode',
          target: `release:${audit.releaseId}`,
        });
        totalJobs++;
      }
    }
  }

  if (totalJobs > 0) {
    snackbarText.value = `Created ${totalJobs} fix jobs for all releases`;
    snackbarColor.value = 'success';
    snackbar.value = true;
  }
}

async function runIssueFix(action: string) {
  if (!selectedIssue.value) return;

  const { key, audit, archiveOrgId } = selectedIssue.value;

  switch (action) {
    case 'reingest_archivist':
      // Use 'analyze' for metadata-only fixes if already on Archivist (efficient, no re-upload)
      // Use 'source_import' if content needs to be migrated from IPFS
      if (audit?.contentCid) {
        const isMetadataOnly = key === 'missing_audio_quality' || key === 'missing_track_metadata';
        const alreadyOnArchivist = isArchivistCid(audit.contentCid);

        if (isMetadataOnly && alreadyOnArchivist) {
          // Efficient: Just analyze, don't re-upload
          createJobMutation.mutate({
            job_type: 'analyze',
            target: `source:${audit.contentCid}`,
            existing_release_id: audit.releaseId,
          });
          snackbarText.value = `Analyzing ${audit.title}...`;
        } else {
          // Need to migrate/re-upload
          createJobMutation.mutate({
            job_type: 'source_import',
            target: `source:${audit.contentCid}`,
            existing_release_id: audit.releaseId,
          });
          snackbarText.value = `Re-ingesting ${audit.title}...`;
        }
        snackbarColor.value = 'success';
        snackbar.value = true;
        issueFixDialog.value = false;
      }
      break;

    case 'refetch_archive_org':
      // Re-import from Archive.org
      const archiveId = archiveOrgId || audit?.archiveOrgId;
      if (archiveId) {
        createJobMutation.mutate({
          job_type: 'import',
          target: `archive.org:${archiveId}`,
          existing_release_id: audit?.releaseId,
        });
        snackbarText.value = `Re-fetching ${audit?.title} from Archive.org...`;
        snackbarColor.value = 'success';
        snackbar.value = true;
        issueFixDialog.value = false;
      }
      break;

    case 'generate_opus':
      // Create transcode job
      if (audit?.releaseId) {
        createJobMutation.mutate({
          job_type: 'transcode',
          target: `release:${audit.releaseId}`,
        });
        snackbarText.value = `Generating Opus ladder for ${audit.title}...`;
        snackbarColor.value = 'success';
        snackbar.value = true;
        issueFixDialog.value = false;
      }
      break;
  }
}

// Helper to get job result type (handles Rust's externally-tagged enum format)
// Rust serde: { "Import": {...} } vs internally-tagged: { "type": "Import", ... }
function getResultType(result: any): string | null {
  if (!result) return null;
  // Check for externally-tagged (Rust serde default)
  const keys = Object.keys(result);
  if (keys.length === 1 && ['Import', 'Audit', 'Transcode', 'Migrate', 'SourceImport', 'Analyze', 'Error', 'NeedsInput'].includes(keys[0])) {
    return keys[0];
  }
  // Check for internally-tagged
  return result.type || null;
}

// Helper to get job result data (unwraps externally-tagged format)
function getResultData(result: any): any {
  if (!result) return null;
  const type = getResultType(result);
  if (type && result[type]) {
    return result[type];
  }
  return result;
}

function clearArchivedJobs() {
  clearArchivedMutation.mutate(undefined, {
    onSuccess: (data) => {
      snackbarText.value = `Cleared ${data.deleted} archived job${data.deleted === 1 ? '' : 's'}`;
      snackbarColor.value = 'success';
      snackbar.value = true;
    },
  });
}

function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleString();
}

function formatSize(size: string | null): string {
  if (!size) return '';
  const bytes = parseInt(size, 10);
  if (isNaN(bytes)) return size;
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatDuration(seconds: string | null): string {
  if (!seconds) return '';
  const secs = parseFloat(seconds);
  if (isNaN(secs)) return seconds;
  const mins = Math.floor(secs / 60);
  const remainingSecs = Math.floor(secs % 60);
  return `${mins}:${remainingSecs.toString().padStart(2, '0')}`;
}

// Unescape backslash-escaped characters from Archive.org metadata
// Returns null if input is null/undefined so callers can use || fallback properly
function unescapeMetadata(str: string | null | undefined): string | null {
  if (str === null || str === undefined) return null;
  // Archive.org sometimes escapes parentheses and other chars with backslashes
  return str.replace(/\\([()[\]{}])/g, '$1');
}

// ============================================================================
// Source Import Methods (URL/CID)
// ============================================================================

async function importFromUrl() {
  if (!directUrl.value.trim()) return;

  sourceImportLoading.value = true;
  cidError.value = '';

  try {
    await sourceImportMutation.mutateAsync({
      source: directUrl.value.trim(),
    });

    snackbarText.value = 'Import job created';
    snackbarColor.value = 'success';
    snackbar.value = true;

    // Clear input and navigate to queue
    directUrl.value = '';
    currentView.value = 'queue';
  } catch (error: any) {
    snackbarText.value = `Import failed: ${error.message}`;
    snackbarColor.value = 'error';
    snackbar.value = true;
  } finally {
    sourceImportLoading.value = false;
  }
}

async function importFromCid() {
  if (!isValidCid.value) return;

  sourceImportLoading.value = true;
  cidError.value = '';

  try {
    await sourceImportMutation.mutateAsync({
      source: importCid.value.trim(),
      existing_release_id: existingReleaseByCid.value?.id,
    });

    const action = existingReleaseByCid.value ? 'Re-archive' : 'Import';
    snackbarText.value = `${action} job created`;
    snackbarColor.value = 'success';
    snackbar.value = true;

    // Clear input and navigate to queue
    importCid.value = '';
    existingReleaseByCid.value = null;
    currentView.value = 'queue';
  } catch (error: any) {
    cidError.value = error.message;
    snackbarText.value = `Import failed: ${error.message}`;
    snackbarColor.value = 'error';
    snackbar.value = true;
  } finally {
    sourceImportLoading.value = false;
  }
}

// No cleanup needed - global audio player manages its own lifecycle
</script>

<style scoped>
.nav-card {
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.nav-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.item-card {
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.item-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.item-card.selected {
  border: 2px solid rgb(var(--v-theme-primary));
}

.selection-check {
  position: absolute;
  top: 8px;
  right: 8px;
  background: white;
  border-radius: 50%;
}

.status-bar {
  border-radius: 8px;
  gap: 4px;
}

/* CSS Grid for items (like infiniteReleaseList) */
.items-grid-container {
  width: 100%;
  max-width: 100%;
}

.items-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 1rem;
  width: 100%;
}

/* Job queue cards */
.job-card {
  transition: box-shadow 0.2s;
}

.job-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.job-card.job-running {
  border-left: 3px solid rgb(var(--v-theme-success));
}

.job-list code {
  font-size: 0.75rem;
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
}

.clickable-card {
  cursor: pointer;
}

/* Issue summary grid */
.issue-summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 1rem;
}

.issue-summary-card {
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.issue-summary-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

/* Issues list */
.issues-list .issue-card {
  cursor: pointer;
  transition: box-shadow 0.2s;
}

.issues-list .issue-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

/* Gap utility for older browsers */
.gap-1 {
  gap: 4px;
}

.gap-2 {
  gap: 8px;
}

/* Clickable issue chips */
.issue-chip {
  cursor: pointer;
  transition: transform 0.1s, box-shadow 0.1s;
}

.issue-chip:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}
</style>
