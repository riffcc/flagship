import { ref } from 'vue';

// This state will be shared across the entire application
const isLensReady = ref(false);

const initLensService = async () => {
  if (isLensReady.value) return;
  console.log('[Lens] Legacy Lens SDK is dead; using Citadel-native service path');
  isLensReady.value = true;
};

export function useLensInitialization() {
  return {
    isLensReady,
    initLensService,
  };
}
