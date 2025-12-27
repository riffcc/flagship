<template>
  <div class="history-viewport">
    <div class="history-container">
      <div class="history-header">
        <v-btn
          variant="text"
          size="small"
          class="back-btn"
          @click="router.push('/settings')"
        >
          <v-icon size="small">$arrowLeft</v-icon>
          Settings
        </v-btn>
        <h1 class="history-title">Listening History</h1>
      </div>

      <div v-if="!historyEnabled" class="history-disabled">
        <v-icon size="64" color="grey-darken-1">$eyeOff</v-icon>
        <p>History is disabled</p>
        <v-btn
          variant="outlined"
          size="small"
          @click="router.push('/settings')"
        >
          Enable in Settings
        </v-btn>
      </div>

      <template v-else>
        <div v-if="historyItems.length === 0" class="history-empty">
          <v-icon size="64" color="grey-darken-1">$history</v-icon>
          <p>No listening history yet</p>
          <p class="history-empty-hint">Start playing music and it will appear here</p>
        </div>

        <div v-else class="history-content">
          <div class="history-controls">
            <v-text-field
              v-model="searchQuery"
              placeholder="Search history..."
              variant="outlined"
              density="compact"
              hide-details
              clearable
              class="search-field"
            >
              <template #prepend-inner>
                <v-icon size="small">$magnify</v-icon>
              </template>
            </v-text-field>
          </div>

          <div class="history-list">
            <div
              v-for="item in filteredHistory"
              :key="item.id"
              class="history-item"
            >
              <div class="item-artwork">
                <v-img
                  v-if="item.artwork"
                  :src="item.artwork"
                  :alt="item.title"
                  aspect-ratio="1"
                  cover
                />
                <div v-else class="artwork-placeholder">
                  <v-icon>$musicNote</v-icon>
                </div>
              </div>

              <div class="item-info">
                <div class="item-title">{{ item.title }}</div>
                <div class="item-artist">{{ item.artist }}</div>
                <div class="item-meta">
                  <span class="item-album">{{ item.album }}</span>
                  <span class="item-date">{{ formatDate(item.playedAt) }}</span>
                </div>
              </div>

              <div class="item-actions">
                <v-tooltip location="top" text="Play again">
                  <template #activator="{ props }">
                    <v-btn
                      v-bind="props"
                      icon
                      size="small"
                      variant="text"
                      @click="playItem(item)"
                    >
                      <v-icon size="small">$play</v-icon>
                    </v-btn>
                  </template>
                </v-tooltip>

                <v-menu>
                  <template #activator="{ props: menuProps }">
                    <v-btn
                      v-bind="menuProps"
                      icon
                      size="small"
                      variant="text"
                    >
                      <v-icon size="small">$dotsVertical</v-icon>
                    </v-btn>
                  </template>
                  <v-list density="compact">
                    <v-list-item @click="forgetItem(item)">
                      <template #prepend>
                        <v-icon size="small">$close</v-icon>
                      </template>
                      <v-list-item-title>Forget this</v-list-item-title>
                    </v-list-item>
                    <v-list-item @click="excludeFromHistory(item)">
                      <template #prepend>
                        <v-icon size="small">$eyeOff</v-icon>
                      </template>
                      <v-list-item-title>Never track this song</v-list-item-title>
                    </v-list-item>
                  </v-list>
                </v-menu>
              </div>
            </div>
          </div>

          <div v-if="filteredHistory.length === 0 && searchQuery" class="history-no-results">
            <p>No matches for "{{ searchQuery }}"</p>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useSettings } from '/@/composables/useSettings';

interface HistoryItem {
  id: string;
  title: string;
  artist: string;
  album: string;
  artwork?: string;
  playedAt: Date;
  releaseId: string;
  trackId: string;
}

const router = useRouter();
const { historyEnabled } = useSettings();

const searchQuery = ref('');
const historyItems = ref<HistoryItem[]>([]);

// Load history from localStorage
const HISTORY_KEY = 'riffcc-listening-history';
function loadHistory() {
  const stored = localStorage.getItem(HISTORY_KEY);
  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      historyItems.value = parsed.map((item: any) => ({
        ...item,
        playedAt: new Date(item.playedAt),
      }));
    } catch {
      historyItems.value = [];
    }
  }
}

function saveHistory() {
  localStorage.setItem(HISTORY_KEY, JSON.stringify(historyItems.value));
}

loadHistory();

const filteredHistory = computed(() => {
  if (!searchQuery.value) {
    return historyItems.value;
  }
  const query = searchQuery.value.toLowerCase();
  return historyItems.value.filter(
    item =>
      item.title.toLowerCase().includes(query) ||
      item.artist.toLowerCase().includes(query) ||
      item.album.toLowerCase().includes(query)
  );
});

function formatDate(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) {
    const hours = Math.floor(diff / (1000 * 60 * 60));
    if (hours === 0) {
      const minutes = Math.floor(diff / (1000 * 60));
      return minutes <= 1 ? 'Just now' : `${minutes}m ago`;
    }
    return hours === 1 ? '1 hour ago' : `${hours} hours ago`;
  }
  if (days === 1) return 'Yesterday';
  if (days < 7) return `${days} days ago`;
  if (days < 30) {
    const weeks = Math.floor(days / 7);
    return weeks === 1 ? '1 week ago' : `${weeks} weeks ago`;
  }
  return date.toLocaleDateString();
}

function playItem(item: HistoryItem) {
  // Navigate to the album/release and trigger playback
  router.push(`/album/${item.releaseId}`);
}

function forgetItem(item: HistoryItem) {
  historyItems.value = historyItems.value.filter(i => i.id !== item.id);
  saveHistory();
}

function excludeFromHistory(item: HistoryItem) {
  // Store excluded track IDs
  const EXCLUDED_KEY = 'riffcc-history-excluded';
  const stored = localStorage.getItem(EXCLUDED_KEY);
  const excluded: string[] = stored ? JSON.parse(stored) : [];

  if (!excluded.includes(item.trackId)) {
    excluded.push(item.trackId);
    localStorage.setItem(EXCLUDED_KEY, JSON.stringify(excluded));
  }

  // Also remove all instances of this track from history
  historyItems.value = historyItems.value.filter(i => i.trackId !== item.trackId);
  saveHistory();
}
</script>

<style scoped>
.history-viewport {
  min-height: 100%;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 2rem 1rem;
}

@media (min-width: 768px) {
  .history-viewport {
    padding: 3rem 2rem;
  }
}

.history-container {
  width: 100%;
  max-width: 800px;
}

.history-header {
  margin-bottom: 1.5rem;
}

.back-btn {
  font-size: 0.75rem;
  text-transform: none;
  letter-spacing: 0;
  margin-bottom: 0.5rem;
  margin-left: -8px;
  opacity: 0.7;
}

.back-btn:hover {
  opacity: 1;
}

.history-title {
  font-size: 1.75rem;
  font-weight: 300;
  opacity: 0.9;
}

@media (min-width: 768px) {
  .history-title {
    font-size: 2rem;
  }
}

.history-disabled,
.history-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  text-align: center;
  opacity: 0.6;
}

.history-disabled p,
.history-empty p {
  margin: 1rem 0 0;
  font-size: 1rem;
}

.history-empty-hint {
  font-size: 0.875rem !important;
  opacity: 0.7;
}

.history-disabled .v-btn {
  margin-top: 1rem;
}

.history-controls {
  margin-bottom: 1rem;
}

.search-field {
  max-width: 300px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  background: rgba(var(--v-theme-surface), 0.5);
  border-radius: 8px;
  transition: background-color 0.15s ease;
}

.history-item:hover {
  background: rgba(var(--v-theme-surface), 0.8);
}

.item-artwork {
  width: 48px;
  height: 48px;
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
}

.artwork-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
}

.item-info {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.item-title {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-artist {
  font-size: 0.875rem;
  opacity: 0.8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-meta {
  display: flex;
  gap: 0.5rem;
  font-size: 0.75rem;
  opacity: 0.5;
}

.item-album {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-date {
  flex-shrink: 0;
}

.item-meta .item-album::after {
  content: '·';
  margin-left: 0.5rem;
}

.item-actions {
  display: flex;
  gap: 0.25rem;
  flex-shrink: 0;
}

.history-no-results {
  text-align: center;
  padding: 2rem;
  opacity: 0.6;
}
</style>
