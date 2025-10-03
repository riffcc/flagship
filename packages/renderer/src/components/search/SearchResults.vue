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
          <v-icon :icon="getIconForType(result.type)" />
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
            {{ result.category || result.type }}
          </v-chip>
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
defineProps<{
  results: any[];
  query: string;
}>();

defineEmits<{
  select: [result: any];
  close: [];
}>();

/**
 * Get icon for content type
 */
function getIconForType(type: string): string {
  const iconMap: Record<string, string> = {
    music: 'mdi-music',
    movie: 'mdi-movie',
    tv: 'mdi-television',
    other: 'mdi-file'
  };

  return iconMap[type] || 'mdi-file';
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
  left: 0;
  right: 0;
  margin-top: 8px;
  max-height: 400px;
  overflow-y: auto;
  z-index: 1000;
}

.search-result-item {
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.search-result-item:hover {
  background-color: rgba(138, 43, 226, 0.1);
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
</style>
