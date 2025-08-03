import { ref } from 'vue';

// Global playback state
const globalPlaybackHandlers = ref<{
  play: (() => void) | null;
  pause: (() => void) | null;
  togglePlay: (() => void) | null;
}>({
  play: null,
  pause: null,
  togglePlay: null,
});

export function useGlobalPlayback() {
  const registerPlaybackHandlers = (handlers: {
    play: () => void;
    pause: () => void;
    togglePlay: () => void;
  }) => {
    globalPlaybackHandlers.value = handlers;
  };

  const clearPlaybackHandlers = () => {
    globalPlaybackHandlers.value = {
      play: null,
      pause: null,
      togglePlay: null,
    };
  };

  const globalTogglePlay = () => {
    if (globalPlaybackHandlers.value.togglePlay) {
      globalPlaybackHandlers.value.togglePlay();
    }
  };

  const globalPlay = () => {
    if (globalPlaybackHandlers.value.play) {
      globalPlaybackHandlers.value.play();
    }
  };

  const globalPause = () => {
    if (globalPlaybackHandlers.value.pause) {
      globalPlaybackHandlers.value.pause();
    }
  };

  return {
    registerPlaybackHandlers,
    clearPlaybackHandlers,
    globalTogglePlay,
    globalPlay,
    globalPause,
  };
}