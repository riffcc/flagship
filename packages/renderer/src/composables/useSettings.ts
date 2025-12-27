/**
 * User Settings Composable
 *
 * Manages user preferences with localStorage persistence.
 * Only exposes settings that users actually care about.
 */

import { ref, watch, computed } from 'vue';
import { useTheme } from './useTheme';

const SETTINGS_KEY = 'riffcc-settings';
const HISTORY_KEY = 'riffcc-listening-history';

// Animation level descriptions
const animationDescriptions: Record<number, string> = {
  0: 'Minimal — Remove most animations for a snappy and efficient experience',
  1: 'Standard — Riff as it was meant to be experienced',
  2: 'Fancy — Additional eye candy, tasteful and measured',
  3: 'Ultimate — The full vision',
};

// Animation level tick labels for slider
const animationTicks: Record<number, string> = {
  0: 'Minimal',
  1: 'Standard',
  2: 'Fancy',
  3: 'Ultimate',
};

// Quality level ticks (codec + bitrate ladder)
const qualityTicks: Record<number, string> = {
  0: 'Opus 64',
  1: 'Opus 96',
  2: 'Opus 192',
  3: 'FLAC',
};

interface UserSettings {
  // Appearance
  animationLevel: 0 | 1 | 2 | 3;
  // Playback
  streamingBias: 0 | 1; // 0 = save bandwidth, 1 = prefer quality
  autoplayNext: boolean;
  // Advanced playback
  qualityLevel: 0 | 1 | 2 | 3; // 0=Opus64, 1=Opus96, 2=Opus192, 3=FLAC
  gaplessPlayback: boolean;
  // Privacy
  historyEnabled: boolean;
}

const defaultSettings: UserSettings = {
  // Appearance
  animationLevel: 1, // Standard
  // Playback
  streamingBias: 1, // 0 = save bandwidth, 1 = prefer quality
  autoplayNext: true,
  // Advanced playback
  qualityLevel: 2, // Opus 192
  gaplessPlayback: true,
  // Privacy
  historyEnabled: true,
};

// Singleton settings state
let settingsState: ReturnType<typeof createSettingsState> | null = null;

function createSettingsState() {
  // Load from localStorage
  const stored = localStorage.getItem(SETTINGS_KEY);
  const settings = ref<UserSettings>(
    stored ? { ...defaultSettings, ...JSON.parse(stored) } : defaultSettings
  );

  // Persist on change
  watch(
    settings,
    (val) => {
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(val));
      // Apply animation level to document
      applyAnimationLevel(val.animationLevel);
    },
    { deep: true }
  );

  // Apply initial animation level
  applyAnimationLevel(settings.value.animationLevel);

  return settings;
}

/**
 * Apply animation level as a data attribute on the document
 * CSS can use [data-animation-level="0"] to adjust animations
 */
function applyAnimationLevel(level: number) {
  document.documentElement.dataset.animationLevel = String(level);
}

/**
 * Clear listening history from localStorage
 */
async function clearListeningHistory(): Promise<void> {
  localStorage.removeItem(HISTORY_KEY);
  // Could also call an API to clear server-side history if needed
}

export function useSettings() {
  // Initialize singleton if needed
  if (!settingsState) {
    settingsState = createSettingsState();
  }

  const theme = useTheme();

  // Computed properties for v-model binding
  const animationLevel = computed({
    get: () => settingsState!.value.animationLevel,
    set: (val: 0 | 1 | 2 | 3) => {
      settingsState!.value.animationLevel = val;
    },
  });

  const streamingBias = computed({
    get: () => settingsState!.value.streamingBias,
    set: (val: 0 | 1) => {
      settingsState!.value.streamingBias = val;
    },
  });

  const autoplayNext = computed({
    get: () => settingsState!.value.autoplayNext,
    set: (val: boolean) => {
      settingsState!.value.autoplayNext = val;
    },
  });

  const qualityLevel = computed({
    get: () => settingsState!.value.qualityLevel,
    set: (val: 0 | 1 | 2 | 3) => {
      settingsState!.value.qualityLevel = val;
    },
  });

  const gaplessPlayback = computed({
    get: () => settingsState!.value.gaplessPlayback,
    set: (val: boolean) => {
      settingsState!.value.gaplessPlayback = val;
    },
  });

  const historyEnabled = computed({
    get: () => settingsState!.value.historyEnabled,
    set: (val: boolean) => {
      settingsState!.value.historyEnabled = val;
    },
  });

  return {
    // Appearance
    isDark: theme.isDark,
    toggleTheme: theme.toggleTheme,
    animationLevel,
    animationTicks,
    animationDescriptions,

    // Playback
    streamingBias,
    autoplayNext,

    // Advanced playback
    qualityLevel,
    qualityTicks,
    gaplessPlayback,

    // Privacy
    historyEnabled,
    clearListeningHistory,
  };
}

/**
 * Get current animation level (for use outside Vue components)
 */
export function getAnimationLevel(): number {
  const stored = localStorage.getItem(SETTINGS_KEY);
  if (stored) {
    const settings = JSON.parse(stored) as Partial<UserSettings>;
    return settings.animationLevel ?? defaultSettings.animationLevel;
  }
  return defaultSettings.animationLevel;
}
