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
            <v-card width="600px" max-height="620px" class="pa-8 ma-auto" color="black">
              <artist-form
                v-if="currentStructureType === 'artist'"
                mode="create"
                @update:error="handleError"
                @update:success="handleSuccess"
              />
              <structure-form
                v-else
                :initial-type="currentStructureType"
                :parent-context="parentContext"
                @update:error="handleError"
                @update:success="handleSuccess"
              />
            </v-card>
          </v-dialog>
        </template>
        <h3>{{ currentViewTitle }}</h3>
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

      <!-- Categories View -->
      <div v-else-if="currentView === 'categories'">
        <v-row>
          <!-- Music/Artists -->
          <v-col cols="12" md="6">
            <v-card @click="navigateToCategory('music')">
              <v-card-title class="d-flex align-center">
                <v-icon icon="$music" class="mr-2"></v-icon>
                Artists
                <v-spacer></v-spacer>
                <v-chip>{{ artistCount }}</v-chip>
              </v-card-title>
              <v-card-text>
                Manage artists, albums, and musical releases
              </v-card-text>
            </v-card>
          </v-col>

          <!-- TV Shows -->
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
            @click="navigateToAlbumEdit(release)"
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
                icon="$pencil"
                variant="text"
                density="comfortable"
                size="small"
                title="Edit album"
                @click.stop="navigateToAlbumEdit(release)"
              ></v-btn>
              <v-btn
                icon="$play"
                variant="text"
                density="comfortable"
                size="small"
                title="Play album"
                @click.stop="navigateToRelease(release.id)"
              ></v-btn>
            </template>
          </v-list-item>
        </v-list>
      </div>

      <!-- Album Detail View -->
      <div v-else-if="currentView === 'album-detail' && currentAlbum">
        <div class="d-flex align-center mb-4">
          <v-avatar size="64" class="mr-4">
            <v-img
              v-if="currentAlbum.thumbnailCID"
              :src="parseUrlOrCid(currentAlbum.thumbnailCID)"
              cover
            ></v-img>
            <v-icon v-else size="32">$album</v-icon>
          </v-avatar>
          <div>
            <h4>{{ currentAlbum.name }}</h4>
            <p class="text-body-2 text-grey">{{ currentAlbum.metadata?.releaseYear || 'Unknown year' }}</p>
          </div>
          <v-spacer></v-spacer>
          <v-btn
            color="primary"
            prepend-icon="$content-save"
            @click="saveAlbumDetails"
          >
            Save Changes
          </v-btn>
        </div>

        <v-divider class="my-4"></v-divider>

        <!-- Album Covers Section -->
        <h5 class="text-h6 mb-3">Album Artwork</h5>
        <v-row class="mb-6">
          <v-col cols="12" sm="6" md="4">
            <v-card>
              <v-card-subtitle class="pt-3">Front Cover</v-card-subtitle>
              <v-img
                :src="parseUrlOrCid(currentAlbum.thumbnailCID)"
                aspect-ratio="1"
                cover
                max-height="200"
              >
                <template #placeholder>
                  <v-sheet color="grey-darken-3" class="d-flex align-center justify-center fill-height">
                    <v-icon size="48" color="grey">$album</v-icon>
                  </v-sheet>
                </template>
              </v-img>
              <v-card-text>
                <v-text-field
                  :model-value="currentAlbum.thumbnailCID"
                  label="Front Cover URL/CID"
                  density="compact"
                  hide-details
                  @update:model-value="currentAlbum.thumbnailCID = $event"
                ></v-text-field>
              </v-card-text>
            </v-card>
          </v-col>
          <v-col cols="12" sm="6" md="4">
            <v-card>
              <v-card-subtitle class="pt-3">Back Cover</v-card-subtitle>
              <v-img
                :src="parseUrlOrCid(albumBackCover) || undefined"
                aspect-ratio="1"
                cover
                max-height="200"
              >
                <template #placeholder>
                  <v-sheet color="grey-darken-3" class="d-flex align-center justify-center fill-height">
                    <v-icon size="48" color="grey">$album</v-icon>
                    <span class="text-caption ml-2">No back cover</span>
                  </v-sheet>
                </template>
              </v-img>
              <v-card-text>
                <v-text-field
                  v-model="albumBackCover"
                  label="Back Cover URL/CID"
                  density="compact"
                  hide-details
                  placeholder="Enter URL or CID"
                ></v-text-field>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>

        <v-divider class="my-4"></v-divider>

        <!-- Track List Section -->
        <div class="d-flex align-center mb-3">
          <h5 class="text-h6">Track List</h5>
          <v-spacer></v-spacer>
          <v-btn
            size="small"
            prepend-icon="$plus"
            variant="tonal"
            @click="addTrack"
          >
            Add Track
          </v-btn>
        </div>

        <v-card v-if="albumTracks.length > 0">
          <v-list>
            <v-list-item
              v-for="(track, index) in albumTracks"
              :key="index"
              class="py-3"
            >
              <template #prepend>
                <div class="d-flex align-center mr-3">
                  <span class="text-body-2 text-grey mr-3" style="min-width: 24px;">
                    {{ (index + 1).toString().padStart(2, '0') }}
                  </span>
                  <v-avatar size="48" rounded="lg" class="track-artwork-preview">
                    <v-img
                      v-if="track.artwork"
                      :src="parseUrlOrCid(track.artwork)"
                      cover
                    ></v-img>
                    <v-icon v-else size="24" color="grey">$music</v-icon>
                  </v-avatar>
                </div>
              </template>

              <v-row dense align="center">
                <v-col cols="12" sm="4">
                  <v-text-field
                    v-model="track.title"
                    label="Track Title"
                    density="compact"
                    hide-details
                  ></v-text-field>
                </v-col>
                <v-col cols="12" sm="3">
                  <v-text-field
                    v-model="track.artist"
                    label="Artist (optional)"
                    density="compact"
                    hide-details
                    :placeholder="currentArtist?.name || 'Same as album'"
                  ></v-text-field>
                </v-col>
                <v-col cols="12" sm="4">
                  <v-text-field
                    v-model="track.artwork"
                    label="Track Artwork URL/CID"
                    density="compact"
                    hide-details
                    placeholder="Leave empty for album art"
                  ></v-text-field>
                </v-col>
                <v-col cols="12" sm="1" class="text-right">
                  <v-btn
                    icon="$delete"
                    variant="text"
                    size="small"
                    color="error"
                    @click="removeTrack(index)"
                  ></v-btn>
                </v-col>
              </v-row>
            </v-list-item>
          </v-list>
        </v-card>

        <v-sheet v-else color="transparent" class="text-center pa-8">
          <v-icon size="48" color="grey" class="mb-2">$music-note</v-icon>
          <p class="text-body-2 text-grey">No tracks yet. Click "Add Track" to get started.</p>
        </v-sheet>
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
              <v-avatar
                size="32"
                class="mr-2 clickable-thumbnail"
                @click="navigateToAlbum(item.id)"
              >
                <v-img
                  v-if="item.thumbnailCID"
                  :src="parseUrlOrCid(item.thumbnailCID)"
                ></v-img>
                <v-icon v-else size="small">$music</v-icon>
              </v-avatar>
              <div class="editable-cell flex-grow-1" @click="startInlineEdit(item, 'name')">
                <template v-if="inlineEditTarget?.id === item.id && inlineEditField === 'name'">
                  <v-text-field
                    v-model="inlineEditValue"
                    density="compact"
                    hide-details
                    autofocus
                    @keyup.enter="saveInlineEdit(item)"
                    @keyup.escape="cancelInlineEdit"
                    @blur="saveInlineEdit(item)"
                  ></v-text-field>
                </template>
                <template v-else>
                  {{ item.name }}
                  <v-icon size="x-small" class="ml-1 edit-hint">$pencil</v-icon>
                </template>
              </div>
            </div>
          </template>
          <template #item.artist="{ item }">
            <div class="editable-cell" @click="startInlineEdit(item, 'artist')">
              <template v-if="inlineEditTarget?.id === item.id && inlineEditField === 'artist'">
                <v-autocomplete
                  v-model="inlineEditValue"
                  :items="artists"
                  item-title="name"
                  item-value="id"
                  density="compact"
                  hide-details
                  autofocus
                  @update:model-value="saveInlineEdit(item)"
                  @update:search="inlineArtistSearch = $event"
                >
                  <template #append-item>
                    <v-list-item
                      v-if="shouldShowCreateArtistInline"
                      class="text-primary"
                      @click="createArtistAndAssociateMatching(inlineArtistSearch)"
                    >
                      <v-list-item-title>
                        <v-icon start>$plus</v-icon>
                        Create "{{ inlineArtistSearch }}" and associate matching
                      </v-list-item-title>
                    </v-list-item>
                    <v-list-item
                      v-if="matchingExistingArtist"
                      class="text-primary"
                      @click="associateMatchingWithExistingArtist(matchingExistingArtist)"
                    >
                      <v-list-item-title>
                        <v-icon start>$link</v-icon>
                        Associate matching with "{{ matchingExistingArtist.name }}"
                      </v-list-item-title>
                    </v-list-item>
                  </template>
                </v-autocomplete>
              </template>
              <template v-else>
                <span :class="{ 'text-purple': hasArtistStructure(item.metadata?.artistId) }">
                  {{ getArtistName(item.metadata?.artistId) || item.metadata?.author || 'Unknown' }}
                </span>
                <v-icon size="x-small" class="ml-1 edit-hint">$pencil</v-icon>
              </template>
            </div>
          </template>
          <template #item.year="{ item }">
            <div class="editable-cell" @click="startInlineEdit(item, 'year')">
              <template v-if="inlineEditTarget?.id === item.id && inlineEditField === 'year'">
                <v-text-field
                  v-model="inlineEditValue"
                  type="number"
                  density="compact"
                  hide-details
                  autofocus
                  @keyup.enter="saveInlineEdit(item)"
                  @keyup.escape="cancelInlineEdit"
                  @blur="saveInlineEdit(item)"
                ></v-text-field>
              </template>
              <template v-else>
                {{ item.metadata?.releaseYear || '-' }}
                <v-icon size="x-small" class="ml-1 edit-hint">$pencil</v-icon>
              </template>
            </div>
          </template>
          <template #item.actions="{ item }">
            <v-btn
              icon="$pencil"
              variant="text"
              density="comfortable"
              size="small"
              title="Edit"
              @click="editMusicRelease(item)"
            ></v-btn>
            <v-btn
              icon="$link-variant"
              variant="text"
              density="comfortable"
              size="small"
              title="Associate with Artist"
              @click="associateRelease(item)"
            ></v-btn>
            <v-btn
              icon="$star-plus-outline"
              variant="text"
              density="comfortable"
              size="small"
              title="Feature"
              @click="featureRelease(item)"
            ></v-btn>
            <v-btn
              icon="$play"
              variant="text"
              density="comfortable"
              size="small"
              title="Play"
              @click="navigateToRelease(item.id)"
            ></v-btn>
          </template>
        </v-data-table>
      </div>
    </v-sheet>
  </v-container>

  <!-- Edit dialog -->
  <v-dialog v-model="editStructureDialog" max-width="600px">
    <v-card class="py-3" color="black">
      <v-card-title>
        <span class="text-h6 ml-2">{{ editedStructure?.metadata?.type === 'artist' ? 'Edit Artist' : 'Edit Structure' }}</span>
      </v-card-title>
      <v-card-text>
        <artist-form
          v-if="editedStructure?.metadata?.type === 'artist'"
          :initial-data="editedStructure"
          mode="edit"
          @update:error="handleError"
          @update:success="handleSuccess"
        />
        <structure-form
          v-else
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

  <!-- Edit Release Dialog -->
  <v-dialog v-model="editReleaseDialog" max-width="500px">
    <v-card class="py-3" color="black">
      <v-card-title>
        <span class="text-h6 ml-2">Edit Release</span>
      </v-card-title>
      <v-card-text>
        <release-form
          v-if="editedRelease"
          :initial-data="editedRelease"
          mode="edit"
          @update:success="handleReleaseSuccess"
          @update:error="handleError"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          color="blue-darken-1"
          variant="text"
          @click="editReleaseDialog = false"
        >
          Cancel
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- Associate with Artist Dialog -->
  <v-dialog v-model="associateDialog" max-width="400px">
    <v-card class="py-3" color="black">
      <v-card-title>
        <span class="text-h6 ml-2">Associate with Artist</span>
      </v-card-title>
      <v-card-text>
        <v-autocomplete
          v-model="selectedArtistForAssociation"
          :items="artists"
          item-title="name"
          item-value="id"
          label="Select Artist"
          placeholder="Search for an artist..."
          clearable
        ></v-autocomplete>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          color="blue-darken-1"
          variant="text"
          @click="associateDialog = false"
        >
          Cancel
        </v-btn>
        <v-btn
          color="primary"
          variant="flat"
          :disabled="!selectedArtistForAssociation"
          @click="confirmAssociation"
        >
          Associate
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

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
// @ts-nocheck
import { ref, computed, watchEffect } from 'vue';
import { useRouter } from 'vue-router';
import { useGetStructuresQuery, useGetReleasesQuery, useDeleteStructureMutation, useContentCategoriesQuery, useEditReleaseMutation, useAddReleaseMutation } from '/@/plugins/lensService/hooks';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import { parseUrlOrCid } from '/@/utils';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import structureForm from '/@/components/admin/structureForm.vue';
import artistForm from '/@/components/admin/artistForm.vue';
import ReleaseForm from '/@/components/releases/releaseForm.vue';

const router = useRouter();
const { data: structures, isLoading: isStructuresLoading } = useGetStructuresQuery();
const { data: releases, isLoading: isReleasesLoading } = useGetReleasesQuery();
const { data: contentCategories } = useContentCategoriesQuery();
const deleteStructureMutation = useDeleteStructureMutation();
const editReleaseMutation = useEditReleaseMutation();
const addReleaseMutation = useAddReleaseMutation();

const isLoading = computed(() => isStructuresLoading.value || isReleasesLoading.value);

// View state
const currentView = ref<'categories' | 'tv-list' | 'series-detail' | 'season-detail' | 'music-list' | 'artist-detail' | 'album-detail' | 'tags-list' | 'collections-list' | 'all-episodes' | 'all-music'>('categories');
const currentSeries = ref<any>(null);
const currentSeason = ref<any>(null);
const currentArtist = ref<any>(null);
const currentAlbum = ref<any>(null);

// Album editing state
const albumTracks = ref<Array<{ title: string; artist?: string; artwork?: string }>>([]);
const albumBackCover = ref<string>('');
const currentStructureType = ref<string>('series');
const parentContext = ref<any>(null);

// Dialog states
const createStructureDialog = ref(false);
const editStructureDialog = ref(false);
const confirmDeleteDialog = ref(false);
const editReleaseDialog = ref(false);
const associateDialog = ref(false);

const editedStructure = ref<any>({});
const editedRelease = ref<any>(null);
const deleteTarget = ref<any>(null);
const selectedArtistForAssociation = ref<string | null>(null);

// Inline editing state
const inlineEditTarget = ref<any>(null);
const inlineEditField = ref<string>('');
const inlineEditValue = ref<string | number>('');
const inlineArtistSearch = ref<string>('');

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
  if (!releases.value) return [];
  return releases.value.filter((r: any) => r.metadata?.type === 'artist');
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

  // Filter: music category AND not a structure release (like artists)
  return releases.value.filter((r: any) =>
    musicCategoryIds.has(r.categoryId) &&
    r.metadata?.type !== 'artist'
  );
});

// Show "Create artist" option in inline dropdown when search has text and no exact match
const shouldShowCreateArtistInline = computed(() => {
  if (!inlineArtistSearch.value || inlineArtistSearch.value.length < 2) return false;
  const searchLower = inlineArtistSearch.value.toLowerCase();
  return !artists.value.some((a: any) => a.name.toLowerCase() === searchLower);
});

// Show "Associate matching" when an existing artist matches the search
const matchingExistingArtist = computed(() => {
  if (!inlineArtistSearch.value || inlineArtistSearch.value.length < 2) return null;
  const searchLower = inlineArtistSearch.value.toLowerCase();
  return artists.value.find((a: any) => a.name.toLowerCase() === searchLower);
});

// Counts
const tvSeriesCount = computed(() => tvSeries.value.length);
const artistCount = computed(() => artists.value.length);
const tagCount = computed(() => tags.value.length);
const collectionCount = computed(() => collections.value.length);

// Dynamic title based on current view
const currentViewTitle = computed(() => {
  switch (currentView.value) {
    case 'categories':
      return 'Meta';
    case 'music-list':
    case 'artist-detail':
    case 'all-music':
      return 'Artists';
    case 'album-detail':
      return 'Album';
    case 'tv-list':
    case 'series-detail':
    case 'season-detail':
    case 'all-episodes':
      return 'TV Shows';
    case 'tags-list':
      return 'Tags';
    case 'collections-list':
      return 'Collections';
    default:
      return 'Meta';
  }
});

// Breadcrumbs
const breadcrumbs = computed(() => {
  const items = [];

  if (currentView.value !== 'categories') {
    items.push({ title: 'Meta', value: 'categories' });
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

  if (currentView.value === 'music-list' || currentView.value === 'artist-detail' || currentView.value === 'album-detail' || currentView.value === 'all-music') {
    items.push({ title: 'Music', value: 'music-list' });
  }

  if (currentView.value === 'artist-detail' || currentView.value === 'album-detail') {
    items.push({ title: currentArtist.value?.name || 'Artist', value: 'artist-detail' });
  }

  if (currentView.value === 'album-detail') {
    items.push({ title: currentAlbum.value?.name || 'Album', value: 'album-detail' });
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

function navigateToAlbumEdit(album: any) {
  currentAlbum.value = album;

  // Parse existing track metadata
  if (album.metadata?.trackMetadata) {
    try {
      const tracks = typeof album.metadata.trackMetadata === 'string'
        ? JSON.parse(album.metadata.trackMetadata)
        : album.metadata.trackMetadata;

      // Get existing track artwork
      let artworkArray: string[] = [];
      if (album.metadata?.trackArtwork) {
        try {
          artworkArray = typeof album.metadata.trackArtwork === 'string'
            ? JSON.parse(album.metadata.trackArtwork)
            : album.metadata.trackArtwork;
        } catch (e) {
          console.error('Failed to parse trackArtwork:', e);
        }
      }

      // Merge track data with artwork
      albumTracks.value = tracks.map((t: any, i: number) => ({
        title: t.title || '',
        artist: t.artist || '',
        artwork: artworkArray[i] || ''
      }));
    } catch (e) {
      console.error('Failed to parse trackMetadata:', e);
      albumTracks.value = [];
    }
  } else {
    albumTracks.value = [];
  }

  // Parse existing back cover
  albumBackCover.value = album.metadata?.backCoverCID || '';

  currentView.value = 'album-detail';
}

function navigateToCollection(collection: any) {
  // TODO: Implement collection detail view
  openSnackbar('Collection detail view coming soon', 'info');
}

function navigateToRelease(releaseId: string) {
  router.push(`/release/${releaseId}`);
}

function navigateToAlbum(releaseId: string) {
  router.push(`/album/${releaseId}`);
}

function navigateToBreadcrumb(item: any) {
  currentView.value = item.value;

  // Reset context when navigating up
  if (item.value === 'categories' || item.value === 'tv-list' || item.value === 'music-list') {
    currentSeries.value = null;
    currentSeason.value = null;
    currentArtist.value = null;
    currentAlbum.value = null;
  }

  if (item.value === 'series-detail') {
    currentSeason.value = null;
  }

  if (item.value === 'artist-detail') {
    currentAlbum.value = null;
  }
}

function showAllEpisodes() {
  currentView.value = 'all-episodes';
}

function showAllMusicReleases() {
  currentView.value = 'all-music';
}

// Album track management
function addTrack() {
  albumTracks.value.push({ title: '', artist: '', artwork: '' });
}

function removeTrack(index: number) {
  albumTracks.value.splice(index, 1);
}

async function saveAlbumDetails() {
  if (!currentAlbum.value) return;

  try {
    // Build track metadata (title and artist only, as per original structure)
    const trackMetadata = albumTracks.value.map(t => ({
      title: t.title,
      ...(t.artist ? { artist: t.artist } : {})
    }));

    // Build track artwork array (just the URLs/CIDs)
    const trackArtwork = albumTracks.value.map(t => t.artwork || '');

    // Build updated metadata
    const updatedMetadata = {
      ...currentAlbum.value.metadata,
      trackMetadata: JSON.stringify(trackMetadata),
      trackArtwork: JSON.stringify(trackArtwork),
      ...(albumBackCover.value ? { backCoverCID: albumBackCover.value } : {})
    };

    // Remove backCoverCID if empty
    if (!albumBackCover.value && updatedMetadata.backCoverCID) {
      delete updatedMetadata.backCoverCID;
    }

    await editReleaseMutation.mutateAsync({
      id: currentAlbum.value.id,
      name: currentAlbum.value.name,
      categoryId: currentAlbum.value.categoryId,
      contentCID: currentAlbum.value.contentCID,
      thumbnailCID: currentAlbum.value.thumbnailCID,
      metadata: updatedMetadata,
      siteAddress: currentAlbum.value.siteAddress,
      postedBy: currentAlbum.value.postedBy,
    });

    openSnackbar('Album updated successfully', 'success');
  } catch (error: any) {
    openSnackbar(`Failed to save album: ${error.message}`, 'error');
  }
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
  // For artists (releases), use the release edit dialog
  if (structure.metadata?.type === 'artist') {
    editedStructure.value = { ...structure };
    editStructureDialog.value = true;
  } else {
    // For other structures, use the dialog
    editedStructure.value = { ...structure };
    editStructureDialog.value = true;
  }
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

// Music release actions
function editMusicRelease(release: any) {
  editedRelease.value = release;
  editReleaseDialog.value = true;
}

function associateRelease(release: any) {
  editedRelease.value = release;
  associateDialog.value = true;
}

function featureRelease(release: any) {
  // Emit to parent or handle featuring
  openSnackbar('Feature functionality coming soon', 'info');
}

function handleReleaseSuccess(message: string) {
  openSnackbar(message, 'success');
  editReleaseDialog.value = false;
  editedRelease.value = null;
}

async function confirmAssociation() {
  if (!editedRelease.value || !selectedArtistForAssociation.value) return;

  try {
    await editReleaseMutation.mutateAsync({
      id: editedRelease.value.id,
      name: editedRelease.value.name,
      categoryId: editedRelease.value.categoryId,
      contentCID: editedRelease.value.contentCID,
      thumbnailCID: editedRelease.value.thumbnailCID,
      metadata: {
        ...editedRelease.value.metadata,
        artistId: selectedArtistForAssociation.value,
      },
      siteAddress: editedRelease.value.siteAddress,
      postedBy: editedRelease.value.postedBy,
    });
    openSnackbar('Release associated with artist successfully', 'success');
    associateDialog.value = false;
    editedRelease.value = null;
    selectedArtistForAssociation.value = null;
  } catch (error: any) {
    openSnackbar(`Failed to associate: ${error.message}`, 'error');
  }
}

// Inline editing functions
function startInlineEdit(item: any, field: string) {
  inlineEditTarget.value = item;
  inlineEditField.value = field;

  if (field === 'name') {
    inlineEditValue.value = item.name;
  } else if (field === 'artist') {
    inlineEditValue.value = item.metadata?.artistId || '';
  } else if (field === 'year') {
    inlineEditValue.value = item.metadata?.releaseYear || '';
  }
}

function cancelInlineEdit() {
  inlineEditTarget.value = null;
  inlineEditField.value = '';
  inlineEditValue.value = '';
}

async function saveInlineEdit(item: any) {
  if (!inlineEditTarget.value || !inlineEditField.value) return;

  try {
    const updates: any = {
      id: item.id,
      name: item.name,
      categoryId: item.categoryId,
      contentCID: item.contentCID,
      thumbnailCID: item.thumbnailCID,
      metadata: { ...item.metadata },
      siteAddress: item.siteAddress,
      postedBy: item.postedBy,
    };

    if (inlineEditField.value === 'name') {
      updates.name = inlineEditValue.value;
    } else if (inlineEditField.value === 'artist') {
      updates.metadata.artistId = inlineEditValue.value;
    } else if (inlineEditField.value === 'year') {
      updates.metadata.releaseYear = inlineEditValue.value ? Number(inlineEditValue.value) : undefined;
    }

    await editReleaseMutation.mutateAsync(updates);
    openSnackbar('Updated successfully', 'success');
  } catch (error: any) {
    openSnackbar(`Failed to update: ${error.message}`, 'error');
  }

  cancelInlineEdit();
}

function getArtistName(artistId: string | undefined): string | null {
  if (!artistId) return null;
  const artist = artists.value.find((a: any) => a.id === artistId);
  return artist?.name || null;
}

// Check if an artist ID corresponds to an actual artist structure/release
function hasArtistStructure(artistId: string | undefined): boolean {
  if (!artistId) return false;
  return artists.value.some((a: any) => a.id === artistId);
}

// Create a new artist and associate all unassociated releases with matching author name
async function createArtistAndAssociateMatching(artistName: string) {
  if (!artistName || artistName.length < 2) return;

  try {
    // Get music category ID
    const musicCategory = contentCategories.value?.find(
      (c: any) => c.categoryId === 'music'
    );
    if (!musicCategory) {
      openSnackbar('Music category not found', 'error');
      return;
    }

    // Create artist release (artists are temporarily stored as releases with metadata.type = 'artist')
    const artistData = {
      name: artistName,
      categoryId: musicCategory.id,
      contentCID: '',
      thumbnailCID: '',
      metadata: {
        type: 'artist',
      },
    };

    const response = await addReleaseMutation.mutateAsync(artistData);
    const newArtistId = response?.id;

    if (!newArtistId) {
      openSnackbar('Failed to create artist - no ID returned', 'error');
      return;
    }

    // Find all unassociated music releases with matching author name
    const releasesToAssociate = allMusicReleases.value.filter((r: any) => {
      const author = r.metadata?.author || '';
      const hasNoArtistId = !r.metadata?.artistId;
      const authorMatches = author.toLowerCase() === artistName.toLowerCase();
      return hasNoArtistId && authorMatches;
    });

    // Associate each matching release
    let associatedCount = 0;
    for (const release of releasesToAssociate) {
      try {
        await editReleaseMutation.mutateAsync({
          id: release.id,
          name: release.name,
          categoryId: release.categoryId,
          contentCID: release.contentCID,
          thumbnailCID: release.thumbnailCID,
          metadata: {
            ...release.metadata,
            artistId: newArtistId,
          },
          siteAddress: release.siteAddress,
          postedBy: release.postedBy,
        });
        associatedCount++;
      } catch (e) {
        console.error('Failed to associate release:', release.id, e);
      }
    }

    // Also associate the currently editing release if it's open
    if (inlineEditTarget.value) {
      try {
        await editReleaseMutation.mutateAsync({
          id: inlineEditTarget.value.id,
          name: inlineEditTarget.value.name,
          categoryId: inlineEditTarget.value.categoryId,
          contentCID: inlineEditTarget.value.contentCID,
          thumbnailCID: inlineEditTarget.value.thumbnailCID,
          metadata: {
            ...inlineEditTarget.value.metadata,
            artistId: newArtistId,
          },
          siteAddress: inlineEditTarget.value.siteAddress,
          postedBy: inlineEditTarget.value.postedBy,
        });
        associatedCount++;
      } catch (e) {
        console.error('Failed to associate inline target:', e);
      }
    }

    openSnackbar(`Created "${artistName}" and associated ${associatedCount} release${associatedCount !== 1 ? 's' : ''}`, 'success');
    cancelInlineEdit();
  } catch (error: any) {
    openSnackbar(`Failed to create artist: ${error.message}`, 'error');
  }
}

// Associate all unassociated releases with matching author to an EXISTING artist
async function associateMatchingWithExistingArtist(artist: any) {
  const artistId = artist.id;
  const artistName = artist.name;

  // Find all unassociated music releases with matching author name
  const releasesToAssociate = allMusicReleases.value.filter((r: any) => {
    const author = r.metadata?.author || '';
    const hasNoArtistId = !r.metadata?.artistId;
    const authorMatches = author.toLowerCase() === artistName.toLowerCase();
    return hasNoArtistId && authorMatches;
  });

  if (releasesToAssociate.length === 0) {
    openSnackbar(`No unassociated releases found matching "${artistName}"`, 'warning');
    cancelInlineEdit();
    return;
  }

  // Associate each matching release
  let associatedCount = 0;
  for (const release of releasesToAssociate) {
    try {
      await editReleaseMutation.mutateAsync({
        id: release.id,
        name: release.name,
        categoryId: release.categoryId,
        contentCID: release.contentCID,
        thumbnailCID: release.thumbnailCID,
        metadata: {
          ...release.metadata,
          artistId: artistId,
        },
        siteAddress: release.siteAddress,
        postedBy: release.postedBy,
      });
      associatedCount++;
    } catch (e) {
      console.error('Failed to associate release:', release.id, e);
    }
  }

  // Also associate the currently editing release if it matches
  if (inlineEditTarget.value && !inlineEditTarget.value.metadata?.artistId) {
    try {
      await editReleaseMutation.mutateAsync({
        id: inlineEditTarget.value.id,
        name: inlineEditTarget.value.name,
        categoryId: inlineEditTarget.value.categoryId,
        contentCID: inlineEditTarget.value.contentCID,
        thumbnailCID: inlineEditTarget.value.thumbnailCID,
        metadata: {
          ...inlineEditTarget.value.metadata,
          artistId: artistId,
        },
        siteAddress: inlineEditTarget.value.siteAddress,
        postedBy: inlineEditTarget.value.postedBy,
      });
      associatedCount++;
    } catch (e) {
      console.error('Failed to associate inline target:', e);
    }
  }

  openSnackbar(`Associated ${associatedCount} release${associatedCount !== 1 ? 's' : ''} with "${artistName}"`, 'success');
  cancelInlineEdit();
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

.editable-cell {
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.editable-cell:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.editable-cell .edit-hint {
  opacity: 0;
  transition: opacity 0.2s;
}

.editable-cell:hover .edit-hint {
  opacity: 0.5;
}

.text-purple {
  color: #9c27b0 !important;
}

.clickable-thumbnail {
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.clickable-thumbnail:hover {
  transform: scale(1.1);
  box-shadow: 0 2px 8px rgba(156, 39, 176, 0.4);
}

.track-artwork-preview {
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.3);
}
</style>
