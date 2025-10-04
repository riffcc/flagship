<template>
  <div class="epub-reader" ref="readerContainer">
    <!-- Reader toolbar -->
    <v-app-bar color="surface" elevation="1" class="reader-toolbar">
      <v-btn icon @click="goBack">
        <v-icon>mdi-arrow-left</v-icon>
      </v-btn>

      <v-toolbar-title class="text-truncate mx-4">
        {{ bookTitle }}
      </v-toolbar-title>

      <v-spacer />

      <v-btn icon @click="toggleToc">
        <v-icon>mdi-format-list-bulleted</v-icon>
      </v-btn>

      <v-btn icon @click="toggleSettings">
        <v-icon>mdi-cog</v-icon>
      </v-btn>
    </v-app-bar>

    <!-- Main reader area -->
    <div class="reader-content" ref="contentArea">
      <div class="reader-viewport" :style="viewportStyle">
        <div
          class="reader-page"
          v-html="currentPageContent"
          :style="pageStyle"
        ></div>
      </div>
    </div>

    <!-- Navigation controls -->
    <div class="reader-nav">
      <v-btn
        icon
        size="large"
        @click="previousPage"
        :disabled="currentPage === 0"
        class="nav-btn nav-prev"
      >
        <v-icon>mdi-chevron-left</v-icon>
      </v-btn>

      <div class="progress-indicator">
        <span class="text-caption">
          {{ Math.round(progress * 100) }}%
        </span>
        <v-progress-linear
          :model-value="progress * 100"
          color="primary"
          height="4"
          rounded
          class="mt-1"
        ></v-progress-linear>
      </div>

      <v-btn
        icon
        size="large"
        @click="nextPage"
        :disabled="currentPage >= totalPages - 1"
        class="nav-btn nav-next"
      >
        <v-icon>mdi-chevron-right</v-icon>
      </v-btn>
    </div>

    <!-- Table of Contents drawer -->
    <v-navigation-drawer
      v-model="showToc"
      temporary
      location="right"
      width="300"
    >
      <v-list>
        <v-list-subheader>Table of Contents</v-list-subheader>
        <v-list-item
          v-for="(chapter, index) in toc"
          :key="index"
          @click="goToChapter(chapter)"
        >
          <v-list-item-title>{{ chapter.label }}</v-list-item-title>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>

    <!-- Settings drawer -->
    <v-navigation-drawer
      v-model="showSettings"
      temporary
      location="right"
      width="300"
    >
      <v-list>
        <v-list-subheader>Reader Settings</v-list-subheader>

        <v-list-item>
          <v-list-item-title>Font Size</v-list-item-title>
          <v-slider
            v-model="fontSize"
            min="12"
            max="32"
            step="2"
            thumb-label
          ></v-slider>
        </v-list-item>

        <v-list-item>
          <v-list-item-title>Line Height</v-list-item-title>
          <v-slider
            v-model="lineHeight"
            min="1.2"
            max="2.5"
            step="0.1"
            thumb-label
          ></v-slider>
        </v-list-item>

        <v-list-item>
          <v-list-item-title>Page Width</v-list-item-title>
          <v-slider
            v-model="pageWidth"
            min="60"
            max="100"
            step="5"
            thumb-label
          ></v-slider>
        </v-list-item>

        <v-list-item>
          <v-list-item-title>Theme</v-list-item-title>
          <v-btn-toggle v-model="theme" mandatory>
            <v-btn value="light">Light</v-btn>
            <v-btn value="sepia">Sepia</v-btn>
            <v-btn value="dark">Dark</v-btn>
          </v-btn-toggle>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { parseUrlOrCid } from '/@/utils';

interface Chapter {
  label: string;
  href: string;
}

const props = defineProps<{
  bookId: string;
  contentCID: string;
  title?: string;
}>();

const router = useRouter();
const readerContainer = ref<HTMLElement | null>(null);
const contentArea = ref<HTMLElement | null>(null);

// Reader state
const currentPage = ref(0);
const totalPages = ref(0);
const currentPageContent = ref('');
const bookTitle = computed(() => props.title || 'Reading...');
const toc = ref<Chapter[]>([]);

// UI state
const showToc = ref(false);
const showSettings = ref(false);

// Reader settings (persisted to localStorage)
const fontSize = ref(18);
const lineHeight = ref(1.6);
const pageWidth = ref(80);
const theme = ref<'light' | 'sepia' | 'dark'>('light');

// Computed styles
const viewportStyle = computed(() => ({
  maxWidth: `${pageWidth.value}%`,
  margin: '0 auto',
}));

const pageStyle = computed(() => ({
  fontSize: `${fontSize.value}px`,
  lineHeight: lineHeight.value,
  color: theme.value === 'dark' ? '#e0e0e0' : theme.value === 'sepia' ? '#5c4a35' : '#1a1a1a',
  backgroundColor: theme.value === 'dark' ? '#1a1a1a' : theme.value === 'sepia' ? '#f4ecd8' : '#ffffff',
}));

const progress = computed(() => {
  if (totalPages.value === 0) return 0;
  return currentPage.value / totalPages.value;
});

// WASM module reference (will be initialized)
let epubParser: any = null;

// Navigation functions
function goBack() {
  router.back();
}

function toggleToc() {
  showToc.value = !showToc.value;
}

function toggleSettings() {
  showSettings.value = !showSettings.value;
}

function previousPage() {
  if (currentPage.value > 0) {
    currentPage.value--;
    renderPage();
  }
}

function nextPage() {
  if (currentPage.value < totalPages.value - 1) {
    currentPage.value++;
    renderPage();
  }
}

function goToChapter(chapter: Chapter) {
  // TODO: Navigate to chapter
  showToc.value = false;
}

// ePub parsing and rendering
async function loadEpub() {
  try {
    console.log('[EpubReader] Loading ePub from CID:', props.contentCID);

    // Fetch the ePub file
    const epubUrl = parseUrlOrCid(props.contentCID);
    const response = await fetch(epubUrl);
    const arrayBuffer = await response.arrayBuffer();

    // Initialize WASM parser (placeholder for now - will add actual WASM module)
    // For now, use a simple HTML parser
    await initializeParser(arrayBuffer);

    // Extract content and TOC
    await extractContent();

    // Render first page
    renderPage();
  } catch (error) {
    console.error('[EpubReader] Failed to load ePub:', error);
    currentPageContent.value = '<p style="color: red;">Failed to load book. Please try again.</p>';
  }
}

async function initializeParser(arrayBuffer: ArrayBuffer) {
  // TODO: Load WASM module for fast parsing
  // For now, create a simple mock parser
  epubParser = {
    content: 'Sample book content. This will be replaced with actual ePub parsing using WASM for blazing fast performance.',
    chapters: [
      { label: 'Chapter 1', href: '#ch1' },
      { label: 'Chapter 2', href: '#ch2' },
      { label: 'Chapter 3', href: '#ch3' },
    ],
  };
}

async function extractContent() {
  if (!epubParser) return;

  // Extract TOC
  toc.value = epubParser.chapters || [];

  // Calculate pagination based on viewport size
  // This would use WASM for fast text layout calculation
  const content = epubParser.content || '';
  const wordsPerPage = 300; // Approximate
  const words = content.split(/\s+/);
  totalPages.value = Math.ceil(words.length / wordsPerPage);
}

function renderPage() {
  if (!epubParser) return;

  // TODO: Use WASM for fast page rendering
  // For now, simple pagination
  const content = epubParser.content || '';
  const words = content.split(/\s+/);
  const wordsPerPage = 300;
  const start = currentPage.value * wordsPerPage;
  const end = start + wordsPerPage;
  const pageWords = words.slice(start, end);

  currentPageContent.value = `
    <div style="padding: 2rem;">
      <p style="text-align: justify;">
        ${pageWords.join(' ')}
      </p>
    </div>
  `;
}

// Keyboard navigation
function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowLeft') {
    previousPage();
  } else if (event.key === 'ArrowRight') {
    nextPage();
  }
}

// Lifecycle
onMounted(() => {
  loadEpub();
  window.addEventListener('keydown', handleKeydown);

  // Load saved settings
  const savedSettings = localStorage.getItem('epub-reader-settings');
  if (savedSettings) {
    const settings = JSON.parse(savedSettings);
    fontSize.value = settings.fontSize || 18;
    lineHeight.value = settings.lineHeight || 1.6;
    pageWidth.value = settings.pageWidth || 80;
    theme.value = settings.theme || 'light';
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

// Watch settings and save to localStorage
watch([fontSize, lineHeight, pageWidth, theme], () => {
  localStorage.setItem('epub-reader-settings', JSON.stringify({
    fontSize: fontSize.value,
    lineHeight: lineHeight.value,
    pageWidth: pageWidth.value,
    theme: theme.value,
  }));

  // Re-render page when settings change
  renderPage();
});
</script>

<style scoped>
.epub-reader {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  background: var(--v-theme-surface);
}

.reader-toolbar {
  flex-shrink: 0;
}

.reader-content {
  flex: 1;
  overflow: auto;
  position: relative;
}

.reader-viewport {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem 1rem;
}

.reader-page {
  width: 100%;
  padding: 2rem;
  border-radius: 8px;
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.reader-nav {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  background: var(--v-theme-surface);
  border-top: 1px solid rgba(var(--v-border-color), 0.12);
}

.nav-btn {
  flex-shrink: 0;
}

.progress-indicator {
  flex: 1;
  min-width: 200px;
}

/* Touch-friendly hit areas */
.nav-prev {
  position: fixed;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  opacity: 0.3;
  transition: opacity 0.2s;
}

.nav-next {
  position: fixed;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  opacity: 0.3;
  transition: opacity 0.2s;
}

.nav-prev:hover,
.nav-next:hover {
  opacity: 1;
}

/* Ensure text is selectable */
.reader-page ::v-deep(*) {
  user-select: text;
}
</style>
