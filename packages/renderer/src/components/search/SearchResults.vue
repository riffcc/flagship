<template>
  <v-card class="search-results" elevation="8">
    <v-list density="compact">
      <v-list-item
        v-for="result in results"
        :key="result.id"
        @click="$emit('select', result)"
        class="search-result-item"
      >
        <template #prepend>
          <v-avatar
            v-if="result.thumbnailCID"
            size="48"
            rounded="sm"
            class="thumbnail-avatar"
          >
            <v-img
              :src="parseUrlOrCid(result.thumbnailCID)"
              cover
              :alt="result.title"
            />
          </v-avatar>
          <v-icon v-else :icon="getIconForType(result.type)" />
        </template>

        <v-list-item-title>
          <span v-html="highlightMatch(result.title, query)" />
        </v-list-item-title>

        <v-list-item-subtitle v-if="result.artist">
          {{ result.artist }}
          <span v-if="result.year" class="year">{{ result.year }}</span>
        </v-list-item-subtitle>

        <template #append>
          <v-chip size="x-small" variant="tonal">
            {{ result.type }}
          </v-chip>
          <v-btn
            v-if="canAccessAdminPanel"
            icon
            size="x-small"
            variant="text"
            class="ml-2"
            @click.stop="goToEdit(result)"
          >
            <v-icon size="18">$pencil</v-icon>
          </v-btn>
        </template>
      </v-list-item>

      <v-divider v-if="results.length > 0" />

      <v-list-item class="search-footer">
        <v-list-item-subtitle class="text-center">
          {{ results.length }} result{{ results.length !== 1 ? 's' : '' }} found
        </v-list-item-subtitle>
      </v-list-item>
    </v-list>
  </v-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { parseUrlOrCid } from '/@/utils';
import { useAccountStatusQuery } from '/@/plugins/lensService/hooks';

const router = useRouter();
const { data: accountStatus } = useAccountStatusQuery();

defineProps<{
  results: any[];
  query: string;
}>();

const emit = defineEmits<{
  select: [result: any];
  close: [];
}>();

const canAccessAdminPanel = computed(() => {
  if (!accountStatus.value) return false;
  if (accountStatus.value.isAdmin) return true;
  if (accountStatus.value.roles.includes('moderator')) return true;
  return false;
});

function goToEdit(result: any) {
  emit('close');

  // For artists, navigate to the artist page
  if (result.type === 'artist') {
    // Navigate to artist page - the page will handle opening the edit modal
    router.push(`/artists/${result.id}?edit=true`);
  } else {
    router.push(`/admin/releases/${result.id}/edit`);
  }
}

/**
 * Get icon for content type
 */
function getIconForType(type: string): string {
  const iconMap: Record<string, string> = {
    artist: '$account-music',
    music: '$music',
    movie: '$movie',
    tv: '$television',
    other: '$file'
  };

  return iconMap[type] || '$file';
}

/**
 * Highlight matching terms in text
 */
function highlightMatch(text: string, query: string): string {
  if (!query || !text) return text;

  const terms = query.toLowerCase().split(/\s+/);
  let highlighted = text;

  terms.forEach(term => {
    if (term.length < 2) return;

    const regex = new RegExp(`(${escapeRegex(term)})`, 'gi');
    highlighted = highlighted.replace(regex, '<strong>$1</strong>');
  });

  return highlighted;
}

/**
 * Escape special regex characters
 */
function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
</script>

<style scoped>
.search-results {
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 8px;
  width: 800px;
  max-width: 90vw;
  max-height: 500px;
  overflow-y: auto;
  z-index: 1000;
}

.search-result-item {
  cursor: pointer;
  transition: background-color 0.2s ease;
  min-height: 64px;
}

.search-result-item:hover {
  background-color: rgba(138, 43, 226, 0.1);
}

.thumbnail-avatar {
  margin-right: 8px;
  border: 1px solid rgba(138, 43, 226, 0.3);
  overflow: hidden;
}

.year {
  color: rgba(255, 255, 255, 0.6);
  margin-left: 8px;
}

.search-footer {
  background-color: rgba(0, 0, 0, 0.2);
  min-height: 32px;
}

:deep(strong) {
  color: #8a2be2;
  font-weight: 600;
}

:deep(.v-list-item-title) {
  white-space: normal;
  overflow-wrap: break-word;
}
</style>
