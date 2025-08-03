<template>
  <v-dialog
    v-model="show"
    width="400"
    persistent
    no-click-animation
  >
    <v-card
      color="grey-darken-4"
      :elevation="24"
      class="start-menu-card"
    >
      <v-list
        color="transparent"
        density="comfortable"
      >
        <v-list-item
          title="⚙️ Settings"
          :data-navigable="true"
          @click="handleSettings"
        ></v-list-item>
        <v-list-item
          title="🌓 Toggle Dark/Light Mode"
          :data-navigable="true"
          @click="toggleTheme"
        ></v-list-item>
        <v-list-item
          title="🎮 Controls"
          :data-navigable="true"
          @click="showControls"
        ></v-list-item>
        <v-list-item
          title="ℹ️ About Riff.CC"
          :data-navigable="true"
          @click="showAbout"
        ></v-list-item>
        <v-divider class="my-2"></v-divider>
        <v-list-item
          title="🏠 Home"
          :data-navigable="true"
          @click="goHome"
        ></v-list-item>
        <v-list-item
          title="🎵 Music"
          :data-navigable="true"
          @click="goMusic"
        ></v-list-item>
        <v-list-item
          title="🎬 Movies"
          :data-navigable="true"
          @click="goMovies"
        ></v-list-item>
        <v-list-item
          title="📺 TV Shows"
          :data-navigable="true"
          @click="goTVShows"
        ></v-list-item>
        <v-divider class="my-2"></v-divider>
        <v-list-item
          title="❌ Close Menu"
          :data-navigable="true"
          @click="show = false"
        ></v-list-item>
      </v-list>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { useGamepad } from '/@/composables/useGamepad';
import { useGamepadNavigation } from '/@/composables/useGamepadNavigation';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const show = ref(props.modelValue);
const dialogRef = ref<HTMLElement>();
const selectedIndex = ref(0);
const menuItems = ref<HTMLElement[]>([]);

const { onButtonPress, gamepadState } = useGamepad();
const { focusedElement, lockNavigation, unlockNavigation } = useGamepadNavigation();

// Store the previously focused element
let previousFocus: HTMLElement | null = null;

watch(() => props.modelValue, (value) => {
  show.value = value;
  if (value) {
    // Lock navigation to menu only
    lockNavigation();
    // Store current focus and focus the menu
    previousFocus = focusedElement.value;
    nextTick(() => {
      focusFirstMenuItem();
    });
  } else {
    // Unlock navigation
    unlockNavigation();
    // Restore previous focus
    if (previousFocus) {
      previousFocus.focus();
    }
  }
});

watch(show, (value) => {
  emit('update:modelValue', value);
});

// Focus management
const focusFirstMenuItem = () => {
  selectedIndex.value = 0;
  const items = document.querySelectorAll('.start-menu-card .v-list-item');
  if (items.length > 0) {
    (items[0] as HTMLElement).focus();
  }
};

// Handle gamepad navigation within menu
const handleMenuNavigation = () => {
  if (!show.value) return;
  
  const items = document.querySelectorAll('.start-menu-card .v-list-item');
  if (items.length === 0) return;
  
  // D-pad navigation
  if (gamepadState.value.buttons.down) {
    selectedIndex.value = Math.min(selectedIndex.value + 1, items.length - 1);
    (items[selectedIndex.value] as HTMLElement).focus();
  } else if (gamepadState.value.buttons.up) {
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
    (items[selectedIndex.value] as HTMLElement).focus();
  }
  
  // A button to select
  if (gamepadState.value.buttons.a) {
    (items[selectedIndex.value] as HTMLElement).click();
  }
  
  // B button to close
  if (gamepadState.value.buttons.b) {
    show.value = false;
  }
};

onMounted(() => {
  // Set up gamepad handlers for menu
  const interval = setInterval(() => {
    if (show.value) {
      handleMenuNavigation();
    }
  }, 100);
  
  onUnmounted(() => {
    clearInterval(interval);
  });
});

// Menu actions that just close the menu
const handleSettings = () => {
  console.log('Settings clicked');
  show.value = false;
};

const toggleTheme = () => {
  console.log('Toggle theme - not yet implemented');
  show.value = false;
};

const showControls = () => {
  console.log('Show controls - not yet implemented');
  show.value = false;
};

const showAbout = () => {
  console.log('Show about - not yet implemented');
  show.value = false;
};

const goHome = () => {
  console.log('Go home clicked');
  show.value = false;
};

const goMusic = () => {
  console.log('Go to music clicked');
  show.value = false;
};

const goMovies = () => {
  console.log('Go to movies clicked');
  show.value = false;
};

const goTVShows = () => {
  console.log('Go to TV shows clicked');
  show.value = false;
};
</script>

<style scoped>
/* Vuetify dialog backdrop override */
:deep(.v-overlay__scrim) {
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  background: rgba(0, 0, 0, 0.5) !important;
  transition: all 0.3s ease;
}

.start-menu-card {
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    transform: translateY(-20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.v-list-item {
  transition: background-color 0.2s ease;
  padding: 12px 16px;
}

.v-list-item:hover,
.v-list-item:focus {
  background-color: rgba(138, 43, 226, 0.2);
  outline: 2px solid rgba(138, 43, 226, 0.6);
  outline-offset: -2px;
}

/* Hide focus outline from keyboard/gamepad but keep it accessible */
.v-list-item:focus:not(:focus-visible) {
  outline: 2px solid rgba(138, 43, 226, 0.6);
}
</style>