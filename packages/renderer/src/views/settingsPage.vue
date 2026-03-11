<template>
  <div class="settings-viewport">
    <div class="settings-container">
      <h1 class="settings-title">Settings</h1>

      <div class="settings-grid">
        <!-- Appearance -->
        <div class="settings-card">
          <h2 class="card-title">Appearance</h2>
          <div class="card-content">
            <v-tooltip location="top" text="Switch between light and dark interface">
              <template #activator="{ props }">
                <v-switch
                  v-bind="props"
                  v-model="isDark"
                  label="Dark mode"
                  color="primary"
                  hide-details
                  inset
                  density="comfortable"
                />
              </template>
            </v-tooltip>

            <!-- Hidden until we implement animation levels for real -->
            <div v-if="false" class="control-group">
              <v-tooltip location="top" text="Control motion and visual effects">
                <template #activator="{ props }">
                  <label v-bind="props" class="control-label">Animations</label>
                </template>
              </v-tooltip>
              <v-slider
                v-model="animationLevel"
                :ticks="animationTicks"
                :min="0"
                :max="3"
                step="1"
                show-ticks="always"
                tick-size="4"
                color="primary"
                track-color="grey-darken-3"
                hide-details
              />
              <p class="control-hint">{{ animationDescriptions[animationLevel] }}</p>
            </div>
          </div>
        </div>

        <!-- Playback -->
        <div class="settings-card">
          <h2 class="card-title">
            Playback
            <v-btn
              variant="text"
              size="x-small"
              class="preview-btn"
              @click="previewPlayback"
            >
<v-icon size="small">$play</v-icon> Preview
            </v-btn>
          </h2>
          <div class="card-content">
            <div class="control-group">
              <div class="slider-labels">
                <span class="slider-label-left">Save bandwidth</span>
                <span class="slider-label-right">Prefer quality</span>
              </div>
              <div
                class="binary-slider-wrapper"
                @click="snapStreamingSlider"
                @mouseup="snapStreamingSlider"
                @touchend="snapStreamingSlider"
              >
                <v-slider
                  v-model="streamingSliderPosition"
                  :min="0"
                  :max="100"
                  :step="1"
                  color="primary"
                  track-color="grey-darken-3"
                  hide-details
                  class="binary-slider"
                  :class="{ 'will-snap-right': streamingSliderPosition > 50, 'will-snap-left': streamingSliderPosition <= 50 }"
                  @update:model-value="onSliderDrag"
                />
              </div>
              <p class="control-hint">{{ streamingBias === 0 ? 'Adapts aggressively to save data' : 'Prioritise quality. Uses more bandwidth.' }}</p>
            </div>

            <v-tooltip location="top" text="Automatically play the next track when current ends">
              <template #activator="{ props }">
                <v-switch
                  v-bind="props"
                  v-model="autoplayNext"
                  label="Autoplay next"
                  color="primary"
                  hide-details
                  inset
                  density="comfortable"
                />
              </template>
            </v-tooltip>

            <!-- Advanced Playback (collapsed by default) -->
            <v-expansion-panels v-model="advancedExpanded" variant="accordion" class="advanced-panel">
              <v-expansion-panel value="advanced">
                <v-expansion-panel-title class="advanced-title">
                  <v-tooltip v-if="advancedExpanded === 'advanced'" location="top" text="Target audio quality when available">
                    <template #activator="{ props }">
                      <span v-bind="props">Quality</span>
                    </template>
                  </v-tooltip>
                  <span v-else>Advanced</span>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
                  <div class="advanced-content">
                    <div class="control-group quality-slider-inline">
                      <v-slider
                        v-model="qualityLevel"
                        :ticks="qualityTicks"
                        :min="0"
                        :max="3"
                        step="1"
                        show-ticks="always"
                        tick-size="4"
                        color="primary"
                        track-color="grey-darken-3"
                        hide-details
                      />
                    </div>

                    <v-tooltip location="top" text="Seamless transitions between tracks">
                      <template #activator="{ props }">
                        <v-switch
                          v-bind="props"
                          v-model="gaplessPlayback"
                          label="Gapless playback"
                          color="primary"
                          hide-details
                          inset
                          density="comfortable"
                        />
                      </template>
                    </v-tooltip>
                  </div>
                </v-expansion-panel-text>
              </v-expansion-panel>
            </v-expansion-panels>
          </div>
        </div>

        <!-- Privacy -->
        <div class="settings-card">
          <h2 class="card-title">Privacy</h2>
          <div class="card-content">
            <v-tooltip location="top" text="History is used to tune your experience and is for you alone. We never share your listening history.">
              <template #activator="{ props }">
                <v-switch
                  v-bind="props"
                  v-model="historyEnabled"
                  label="Enable history"
                  color="primary"
                  hide-details
                  inset
                  density="comfortable"
                />
              </template>
            </v-tooltip>

            <div class="control-group">
              <v-tooltip location="top" text="View, forget, or exclude items from history">
                <template #activator="{ props }">
                  <v-btn
                    v-bind="props"
                    variant="outlined"
                    size="small"
                    block
                    @click="goToHistory"
                  >
                    View & manage history
                  </v-btn>
                </template>
              </v-tooltip>
            </div>

            <div class="control-group">
              <v-tooltip location="top" :text="historyEnabled ? 'Permanently forget all listening history and start fresh' : 'Permanently forget all listening history'">
                <template #activator="{ props }">
                  <v-btn
                    v-bind="props"
                    variant="outlined"
                    color="error"
                    size="small"
                    block
                    @click="clearHistory"
                    :loading="clearingHistory"
                  >
                    Clear all history
                  </v-btn>
                </template>
              </v-tooltip>
            </div>
          </div>
        </div>

        <!-- Downloads -->
        <div class="settings-card settings-card--placeholder">
          <h2 class="card-title">
            Downloads
            <span class="coming-soon">Soon</span>
          </h2>
          <div class="card-content card-content--muted">
            <p>Offline playback and download management.</p>
          </div>
        </div>

        <!-- Notifications -->
        <div class="settings-card settings-card--placeholder">
          <h2 class="card-title">
            Notifications
            <span class="coming-soon">Soon</span>
          </h2>
          <div class="card-content card-content--muted">
            <p>Control what you're notified about.</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useSettings } from '/@/composables/useSettings';
import { useUserSession } from '/@/composables/userSession';

const router = useRouter();
const { userData } = useUserSession();

const {
  isDark,
  animationLevel,
  animationTicks,
  animationDescriptions,
  streamingBias,
  autoplayNext,
  qualityLevel,
  qualityTicks,
  gaplessPlayback,
  historyEnabled,
  clearListeningHistory,
} = useSettings();

// Slider position for smooth dragging (0-100), snaps to 0 or 100 on release
const streamingSliderPosition = ref(streamingBias.value === 0 ? 0 : 100);
let isDragging = false;

// Track drag state
function onSliderDrag() {
  isDragging = true;
}

// Snap to nearest endpoint on click/release
function snapStreamingSlider() {
  // Use requestAnimationFrame to ensure we snap after the value updates
  requestAnimationFrame(() => {
    const snapTo = streamingSliderPosition.value > 50 ? 100 : 0;
    streamingSliderPosition.value = snapTo;
    streamingBias.value = (snapTo === 0 ? 0 : 1) as 0 | 1;
    isDragging = false;
  });
}

const clearingHistory = ref(false);
const advancedExpanded = ref<string | undefined>(undefined);

async function clearHistory() {
  clearingHistory.value = true;
  try {
    await clearListeningHistory();
  } finally {
    clearingHistory.value = false;
  }
}

function goToHistory() {
  router.push('/settings/history');
}

function previewPlayback() {
  // Play last FLAC song from history (MP3/Ogg can't demo quality difference)
  // Fallback: most recently featured FLAC album if no history or history disabled
  // If already playing, apply new settings to current playback via DAA
}
</script>

<style scoped>
/* Viewport: full height, centered content */
.settings-viewport {
  min-height: 100%;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 2rem 1rem;
}

@media (min-width: 768px) {
  .settings-viewport {
    padding: 3rem 2rem;
    align-items: center;
  }
}

/* Container: max width for ultrawide */
.settings-container {
  width: 100%;
  max-width: 1400px;
}

/* Title */
.settings-title {
  font-size: 1.75rem;
  font-weight: 300;
  margin-bottom: 1.5rem;
  opacity: 0.9;
}

@media (min-width: 768px) {
  .settings-title {
    font-size: 2rem;
    margin-bottom: 2rem;
    text-align: center;
  }
}

/* Grid: the magic */
.settings-grid {
  display: grid;
  gap: 1rem;
  grid-template-columns: 1fr;
}

@media (min-width: 600px) {
  .settings-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 1.25rem;
  }
}

@media (min-width: 1024px) {
  .settings-grid {
    grid-template-columns: repeat(3, 1fr);
    gap: 1.5rem;
  }
}

@media (min-width: 1600px) {
  .settings-grid {
    grid-template-columns: repeat(4, minmax(280px, 320px));
    justify-content: center;
    gap: 1.5rem;
  }
}

@media (min-width: 2000px) {
  .settings-grid {
    grid-template-columns: repeat(5, minmax(260px, 300px));
  }
}

/* Cards: instrument panels */
.settings-card {
  background: rgba(var(--v-theme-surface), 0.7);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
}

.settings-card--placeholder {
  opacity: 0.6;
}

.card-title {
  font-size: 0.875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.preview-btn {
  margin-left: auto;
  font-size: 0.625rem;
  text-transform: none;
  letter-spacing: 0;
  opacity: 0.7;
}

.preview-btn:hover {
  opacity: 1;
}

.coming-soon {
  font-size: 0.625rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  padding: 0.2em 0.5em;
  border-radius: 4px;
  background: rgba(138, 43, 226, 0.2);
  color: rgb(180, 130, 255);
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
  flex: 1;
}

.card-content--muted {
  justify-content: center;
  color: rgba(255, 255, 255, 0.5);
  font-size: 0.875rem;
}

.card-content--muted p {
  margin: 0;
}

/* Control groups */
.control-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.control-label {
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  opacity: 0.7;
  margin-bottom: 0.25rem;
}

.control-hint {
  font-size: 0.75rem;
  opacity: 0.5;
  margin: 0;
  line-height: 1.4;
}

/* Slider tick labels */
:deep(.v-slider-track__tick-label) {
  font-size: 9px !important;
  opacity: 0.6;
}

/* Streaming slider labels */
.slider-labels {
  display: flex;
  justify-content: space-between;
  font-size: 0.7rem;
  opacity: 0.6;
  margin-bottom: 0.25rem;
}

.slider-label-left,
.slider-label-right {
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Card description */
.card-description {
  font-size: 0.8125rem;
  opacity: 0.7;
  margin: 0 0 0.75rem 0;
  line-height: 1.4;
}

/* Advanced panel styling */
.advanced-panel {
  margin-top: 0.5rem;
}

.advanced-panel :deep(.v-expansion-panel) {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
}

.advanced-panel :deep(.v-expansion-panel-title) {
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  min-height: 36px;
  padding: 0 12px;
  opacity: 0.7;
}

.advanced-panel :deep(.v-expansion-panel-title:hover) {
  opacity: 1;
}

.advanced-panel :deep(.v-expansion-panel-text__wrapper) {
  padding: 0 0.75rem 0.75rem;
}

.advanced-content {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

/* Binary slider with snap behavior */
.binary-slider-wrapper {
  position: relative;
}

.binary-slider :deep(.v-slider-thumb) {
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1),
              box-shadow 0.15s ease;
}

/* Snap animation when released */
.binary-slider :deep(.v-slider-thumb__surface) {
  transition: background-color 0.2s ease;
}

/* Visual feedback: approaching the "other" side */
.binary-slider.will-snap-right :deep(.v-slider-thumb__surface) {
  box-shadow: 2px 0 8px rgba(var(--v-theme-primary), 0.4);
}

.binary-slider.will-snap-left :deep(.v-slider-thumb__surface) {
  box-shadow: -2px 0 8px rgba(var(--v-theme-primary), 0.4);
}

/* Quality slider sits tight under the title when expanded */
.quality-slider-inline {
  margin-top: -1.5rem;
  margin-bottom: 0.5rem;
}

</style>
