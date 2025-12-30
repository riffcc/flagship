<template>
  <v-hover
    v-slot="{props: hoveringProps, isHovering}"
    open-delay="100"
    close-delay="100"
  >
    <v-sheet
      v-bind="hoveringProps"
      class="cursor-pointer mx-auto content-card"
      color="transparent"
      :height="cardHeight"
      :width="cardWidth"
      :style="showDefederation ? `border: 1px solid ${getSiteColor(item.siteAddress)};` : ''"
      data-navigable="true"
      tabindex="0"
      @click="onClick"
    >
      <template v-if="isOverlapping">
        <v-img
          :src="parseUrlOrCid(props.item.thumbnailCID) ?? '/no-image-icon.png'"
          width="100%"
          cover
          aspect-ratio="1"
          :gradient="cardBackgroundGradient"
        >
          <p class="ml-4 mt-2 text-subtitle-1">
            {{ cardTitle }}
          </p>
          <p
            v-if="cardSubtitle"
            class="ml-4 text-subtitle-2"
          >
            {{ cardSubtitle }}
          </p>
          <p
            v-if="item.metadata?.isSeries && item.metadata?.episodeCount"
            class="ml-4 text-caption"
          >
            {{ item.metadata.episodeCount }} episode{{ item.metadata.episodeCount !== 1 ? 's' : '' }}
          </p>
        </v-img>
      </template>
      <template v-else>
        <v-img
          :src="parseUrlOrCid(props.item.thumbnailCID) ?? '/no-image-icon.png'"
          width="100%"
          cover
          aspect-ratio="1"
          class="card-image"
        />
        <p class="text-caption text-sm-subtitle-1 text-center mt-1">
          <a
            class="title-link"
            @click.stop="handleTitleClick"
          >{{ cardTitle }}</a>
        </p>
        <p
          v-if="cardSubtitle"
          class="text-caption text-sm-subtitle-1 text-center text-medium-emphasis"
        >
          <a
            v-if="item.metadata?.artistId"
            class="artist-link"
            @click.stop="router.push(`/artist/${item.metadata.artistId}`)"
          >
            {{ cardSubtitle }}
          </a>
          <span v-else>{{ cardSubtitle }}</span>
        </p>
        <p
          v-if="item.metadata?.isSeries && item.metadata?.episodeCount"
          class="text-caption text-center text-medium-emphasis"
        >
          {{ item.metadata.episodeCount }} episode{{ item.metadata.episodeCount !== 1 ? 's' : '' }}
        </p>
      </template>
    </v-sheet>
  </v-hover>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue';
import { useShowDefederation } from '/@/composables/showDefed';
import { useSiteColors } from '/@/composables/siteColors';
import { useImageColorExtraction } from '/@/composables/imageColorExtraction';
import { type ReleaseItem } from '/@/types';
import { parseUrlOrCid } from '/@/utils';
import { useRouter } from 'vue-router';


const { showDefederation } = useShowDefederation();
const { getSiteColor } = useSiteColors();
const { getColorTintedGradient } = useImageColorExtraction();
const router = useRouter();

const props = defineProps<{
  item: ReleaseItem;
  cursorPointer?: boolean;
  onClick?: () => void;
}>();

const emit = defineEmits<{
  'play': [item: ReleaseItem];
  'info': [item: ReleaseItem];
}>();

function handleTitleClick() {
  navigateToInfoPage();
}

function navigateToInfoPage() {
  const item = props.item;
  const category = item.categoryId;
  const metadata = item.metadata;

  // Tile data we already have - pass through router state for instant render
  const tileState = {
    name: item.name,
    thumbnailCID: item.thumbnailCID,
    contentCID: item.contentCID,
    author: metadata?.author,
    artistId: metadata?.artistId,
    releaseYear: metadata?.releaseYear,
    trackCount: metadata?.trackCount,
  };

  // Route to appropriate info page based on content type
  if (metadata?.type === 'artist') {
    router.push({ path: `/artist/${item.id}`, state: tileState });
  } else if (metadata?.type === 'series' || metadata?.isSeries) {
    router.push({ path: `/series/${item.id}`, state: tileState });
  } else if (category === 'music') {
    router.push({ path: `/album/${item.id}`, state: tileState });
  } else if (category === 'movies') {
    router.push({ path: `/movie/${item.id}`, state: tileState });
  } else if (category === 'tv-shows' || category === 'tvShow') {
    router.push({ path: `/series/${item.id}`, state: tileState });
  } else if (category === 'books') {
    router.push({ path: `/book/${item.id}`, state: tileState });
  } else if (category === 'audiobooks') {
    router.push({ path: `/audiobook/${item.id}`, state: tileState });
  } else if (category === 'podcasts') {
    router.push({ path: `/podcast/${item.id}`, state: tileState });
  } else {
    // Fallback to generic release page
    router.push({ path: `/release/${item.id}`, state: tileState });
  }
}

// Dynamic gradient based on image color
const dynamicGradient = ref<string>('to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)');

// Extract color when component mounts or image changes
onMounted(async () => {
  if (props.item.categoryId === 'tvShow') {
    const imageUrl = parseUrlOrCid(props.item.thumbnailCID);
    dynamicGradient.value = await getColorTintedGradient(imageUrl);
  }
});

watch(() => props.item.thumbnailCID, async (newCID) => {
  if (props.item.categoryId === 'tvShow') {
    const imageUrl = parseUrlOrCid(newCID);
    dynamicGradient.value = await getColorTintedGradient(imageUrl);
  }
});

const cardWidth = computed(() => {
  // Fluid width - let the grid handle sizing
  return '100%';
});

const cardHeight = computed(() => {
  if (props.item.categoryId === 'tvShow') {
    return '10rem';
  }
  return undefined;
});

const cardTitle = computed(() => {
  const categoryId = props.item.categoryId;
  const metadata = props.item.metadata;

  if (categoryId === 'music') {
    return props.item.name;
  }

  // For TV content - check if it's a series or an episode
  if (categoryId === 'tvShow' || metadata?.seriesId) {
    // If it's a series tile (has isSeries flag)
    if (metadata?.isSeries) {
      return props.item.name;
    }
    // If it's an episode, show the series name if available
    if (metadata?.seriesName) {
      return metadata.seriesName;
    }
    return props.item.name;
  }

  // Default: show item name (covers movies, books, etc.)
  return props.item.name;
});

const cardSubtitle = computed(() => {
  const categoryId = props.item.categoryId;
  const metadata = props.item.metadata;

  // Music: show artist name (prefer 'artist', fallback to 'author' for legacy)
  if (categoryId === 'music') {
    return props.item.metadata?.['artist'] ?? props.item.metadata?.['author'] ?? '';
  }

  // Movies: no subtitle (no director/creator)
  if (categoryId === 'movies') {
    return undefined;
  }
  if (categoryId === 'tvShow' || categoryId === 'tv-shows' || metadata?.seriesId || metadata?.isSeries) {
    return undefined;
  }

  // Default: no subtitle
  return undefined;
});

const isOverlapping = computed(() => {
  return props.item.categoryId === 'tvShow';
});

const cardBackgroundGradient = computed(() => {
  return props.item.categoryId === 'tvShow' ? dynamicGradient.value : undefined;
});
</script>

<style scoped>
.artist-link,
.title-link {
  color: inherit;
  cursor: pointer;
  text-decoration: none;
}

.artist-link:hover,
.title-link:hover {
  text-decoration: underline;
}

</style>
