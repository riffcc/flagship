import {ref} from 'vue';
import {formatTime} from '/@/utils';

export const usePlaybackController = <T extends HTMLMediaElement>() => {
  const playerRef = ref<T | null>(null);

  const currentTime = ref('00:00');
  const duration = ref('00:00');
  const progress = ref(0);

  const isLoading = ref(true);
  const isPlaying = ref(false);

  const seekingTrack = (v: number) => {
    if (playerRef.value) {
      isLoading.value = true;
      pause();
      playerRef.value.currentTime = v;
    }
  };

  const togglePlay = () => (isPlaying.value ? pause() : play());

  const pause = () => {
    if (playerRef.value) {
      playerRef.value.pause();
      isPlaying.value = false;
      navigator.mediaSession.playbackState = 'paused';
    }
  };

  const play = () => {
    if (playerRef.value) {
      playerRef.value
        .play()
        .then(() => {
          isPlaying.value = true;
          navigator.mediaSession.playbackState = 'playing';
        })
        .catch((error: unknown) => {
          console.error(error);
        });
    }
  };

  const updateProgress = () => {
    if (playerRef.value) {
      currentTime.value = formatTime(playerRef.value.currentTime);
      duration.value = formatTime(playerRef.value.duration);

      progress.value = playerRef.value.currentTime;

      requestAnimationFrame(updateProgress);
    }
  };

  const canPlay = () => {
    isLoading.value = false;
    if (playerRef.value && playerRef.value.currentTime > 0) {
      play();
    }
  };
  return {
    playerRef,
    currentTime,
    duration,
    progress,
    isLoading,
    isPlaying,
    seekingTrack,
    togglePlay,
    updateProgress,
    canPlay,
    play,
    pause,
  };
};
