import {ref, type Ref} from 'vue';

// Module-level state so volume persists across component instances
const volume = ref(1);
const previousVolume = ref(1);

export const usePlayerVolume = function (): {
  mute: () => void;
  unmute: () => void;
  toggleVolume: () => void;
  setVolume: (v: number) => void;
  volume: Ref<number>;
} {
  const mute = () => {
    if (volume.value > 0) {
      previousVolume.value = volume.value;
    }
    volume.value = 0;
  };

  const unmute = () => {
    volume.value = previousVolume.value > 0 ? previousVolume.value : 1;
  };

  const toggleVolume = () => (volume.value > 0 ? mute() : unmute());

  const setVolume = (v: number) => {
    volume.value = Math.max(0, Math.min(1, v));
    if (v > 0) {
      previousVolume.value = v;
    }
  };

  return {
    mute,
    unmute,
    toggleVolume,
    setVolume,
    volume,
  };
};
