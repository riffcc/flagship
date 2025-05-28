<template>
  <v-alert
    v-if="showAlert"
    type="error"
    prominent
    closable
    class="ma-4"
    @click:close="dismissAlert"
  >
    <v-alert-title>Database Corruption Detected</v-alert-title>
    <div>
      The local database appears to be corrupted. This can happen during development.
    </div>
    <div class="mt-2">
      <strong>To fix this:</strong>
      <ol>
        <li>Click the button below to clear your browser storage</li>
        <li>The page will reload automatically</li>
        <li>Your content will be re-synced from the network</li>
      </ol>
    </div>
    <v-btn
      color="white"
      variant="outlined"
      class="mt-3"
      @click="clearStorageAndReload"
    >
      Clear Storage and Reload
    </v-btn>
  </v-alert>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

const showAlert = ref(false);

// Check for corruption indicators in console
onMounted(() => {
  // Listen for corruption errors
  const originalError = console.error;
  console.error = function(...args) {
    originalError.apply(console, args);
    const errorStr = args.join(' ');
    if (errorStr.includes('SQLITE_CORRUPT') || errorStr.includes('database disk image is malformed')) {
      showAlert.value = true;
    }
  };
});

function dismissAlert() {
  showAlert.value = false;
  // Re-show after 30 seconds if corruption persists
  setTimeout(() => {
    showAlert.value = true;
  }, 30000);
}

async function clearStorageAndReload() {
  try {
    // Clear all browser storage
    if ('localStorage' in window) {
      localStorage.clear();
    }
    if ('sessionStorage' in window) {
      sessionStorage.clear();
    }
    
    // Clear IndexedDB
    const databases = await indexedDB.databases();
    for (const db of databases) {
      if (db.name) {
        await indexedDB.deleteDatabase(db.name);
      }
    }
    
    // Reload the page
    window.location.reload();
  } catch (error) {
    console.error('Failed to clear storage:', error);
    // Force reload anyway
    window.location.reload();
  }
}
</script>