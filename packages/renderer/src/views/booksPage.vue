<template>
  <v-container fluid class="books-page">
    <!-- Hero Section -->
    <div class="books-hero mb-8">
      <h1 class="text-h2 text-md-h1 font-weight-bold mb-4">Books</h1>
      <p class="text-h6 text-grey-lighten-1">
        Discover stories in every format - written, spoken, visual, and beyond
      </p>
    </div>

    <!-- Book Categories -->
    <v-row>
      <!-- eBooks/Written -->
      <v-col cols="12" md="6" class="mb-8">
        <v-card
          class="book-category-card"
          rounded="lg"
          elevation="8"
          @click="router.push('/books/written')"
        >
          <v-img
            src="https://images.unsplash.com/photo-1481627834876-b7833e8f5570?w=800&h=400&fit=crop"
            height="300"
            cover
            gradient="to bottom, rgba(0,0,0,.1), rgba(0,0,0,.7)"
          >
            <div class="pa-6 d-flex flex-column justify-end fill-height">
              <v-icon size="48" color="white" class="mb-2">mdi-book-open-page-variant</v-icon>
              <h2 class="text-h4 text-white font-weight-bold">Written Books</h2>
              <p class="text-body-1 text-white mt-2">
                Classic eBooks and digital literature - read at your own pace
              </p>
            </div>
          </v-img>
        </v-card>
      </v-col>

      <!-- Audiobooks -->
      <v-col cols="12" md="6" class="mb-8">
        <v-card
          class="book-category-card"
          rounded="lg"
          elevation="8"
          @click="router.push('/audiobooks')"
        >
          <v-img
            src="https://images.unsplash.com/photo-1590602846989-e99596d2a6ee?w=800&h=400&fit=crop"
            height="300"
            cover
            gradient="to bottom, rgba(0,0,0,.1), rgba(0,0,0,.7)"
          >
            <div class="pa-6 d-flex flex-column justify-end fill-height">
              <v-icon size="48" color="white" class="mb-2">mdi-book-music</v-icon>
              <h2 class="text-h4 text-white font-weight-bold">Audiobooks</h2>
              <p class="text-body-1 text-white mt-2">
                Narrated stories and literature - listen anywhere, anytime
              </p>
            </div>
          </v-img>
        </v-card>
      </v-col>

      <!-- Graphic Novels & Comics -->
      <v-col cols="12" md="6" class="mb-8">
        <v-card
          class="book-category-card"
          rounded="lg"
          elevation="8"
          @click="router.push('/books/visual')"
        >
          <v-img
            src="https://images.unsplash.com/photo-1612036782180-6f0b6cd846fe?w=800&h=400&fit=crop"
            height="300"
            cover
            gradient="to bottom, rgba(0,0,0,.1), rgba(0,0,0,.7)"
          >
            <div class="pa-6 d-flex flex-column justify-end fill-height">
              <v-icon size="48" color="white" class="mb-2">mdi-book-multiple</v-icon>
              <h2 class="text-h4 text-white font-weight-bold">Visual Books</h2>
              <p class="text-body-1 text-white mt-2">
                Graphic novels, comics, and illustrated stories
              </p>
            </div>
          </v-img>
        </v-card>
      </v-col>

      <!-- Accessible Books -->
      <v-col cols="12" md="6" class="mb-8">
        <v-card
          class="book-category-card"
          rounded="lg"
          elevation="8"
          @click="router.push('/books/accessible')"
        >
          <v-img
            src="https://images.unsplash.com/photo-1456513080510-7bf3a84b82f8?w=800&h=400&fit=crop"
            height="300"
            cover
            gradient="to bottom, rgba(0,0,0,.1), rgba(0,0,0,.7)"
          >
            <div class="pa-6 d-flex flex-column justify-end fill-height">
              <v-icon size="48" color="white" class="mb-2">mdi-braille</v-icon>
              <h2 class="text-h4 text-white font-weight-bold">Accessible Books</h2>
              <p class="text-body-1 text-white mt-2">
                Braille, large print, and accessibility-enhanced formats
              </p>
            </div>
          </v-img>
        </v-card>
      </v-col>
    </v-row>

    <!-- All Books Section -->
    <v-divider class="my-8"></v-divider>

    <div class="all-books-section">
      <h2 class="text-h4 font-weight-bold mb-6">All Books</h2>

      <v-row>
        <v-col
          v-for="book in allBooks"
          :key="book.id"
          cols="6"
          sm="4"
          md="3"
          lg="2"
        >
          <v-card
            class="book-card"
            rounded="lg"
            @click="navigateToBook(book)"
          >
            <v-img
              :src="parseUrlOrCid(book.thumbnailCID)"
              :aspect-ratio="book.metadata?.type === 'audiobook' ? 1 : 0.67"
              cover
            >
              <template #placeholder>
                <v-sheet
                  color="grey-darken-3"
                  class="d-flex align-center justify-center fill-height"
                >
                  <v-icon size="48" color="grey">
                    {{ getBookIcon(book.metadata?.type) }}
                  </v-icon>
                </v-sheet>
              </template>
            </v-img>

            <v-card-text class="pa-2">
              <p class="text-body-2 font-weight-medium text-truncate mb-1">
                {{ book.name }}
              </p>
              <p v-if="book.metadata?.author" class="text-caption text-grey text-truncate">
                {{ book.metadata.author }}
              </p>
              <v-chip
                v-if="book.metadata?.type"
                size="x-small"
                class="mt-1"
                :color="getBookTypeColor(book.metadata.type)"
              >
                {{ getBookTypeLabel(book.metadata.type) }}
              </v-chip>
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useGetReleasesQuery } from '/@/plugins/lensService/hooks';
import { parseUrlOrCid } from '/@/utils';
import type { ReleaseItem } from '/@/types';

const router = useRouter();

// Fetch all releases
const { data: releases } = useGetReleasesQuery({
  searchOptions: { fetch: 1000 }
});

// Filter for all book-related content
const allBooks = computed(() => {
  if (!releases.value) return [];

  return releases.value
    .filter((r: ReleaseItem) =>
      r.categoryId === 'books' ||
      r.categoryId === 'audiobooks' ||
      r.metadata?.type === 'audiobook'
    )
    .slice(0, 24); // Limit for performance
});

function navigateToBook(book: ReleaseItem) {
  if (book.metadata?.type === 'audiobook') {
    router.push(`/audiobook/${book.id}`);
  } else {
    router.push(`/book/${book.id}`);
  }
}

function getBookIcon(type?: string) {
  switch (type) {
    case 'audiobook':
      return 'mdi-book-music';
    case 'graphic-novel':
    case 'comic':
      return 'mdi-book-multiple';
    case 'braille':
      return 'mdi-braille';
    default:
      return 'mdi-book-open-page-variant';
  }
}

function getBookTypeLabel(type?: string) {
  switch (type) {
    case 'audiobook':
      return 'Audio';
    case 'graphic-novel':
      return 'Graphic Novel';
    case 'comic':
      return 'Comic';
    case 'braille':
      return 'Braille';
    default:
      return 'eBook';
  }
}

function getBookTypeColor(type?: string) {
  switch (type) {
    case 'audiobook':
      return 'primary';
    case 'graphic-novel':
    case 'comic':
      return 'secondary';
    case 'braille':
      return 'accent';
    default:
      return '';
  }
}
</script>

<style scoped>
.books-hero {
  padding: 2rem 0;
  text-align: center;
}

.book-category-card {
  cursor: pointer;
  transition: all 0.3s ease;
}

.book-category-card:hover {
  transform: translateY(-8px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.4) !important;
}

.book-card {
  cursor: pointer;
  transition: all 0.2s ease;
}

.book-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
}

.all-books-section {
  margin-top: 2rem;
}
</style>
