import {ref, type Ref} from 'vue';

export const usePlayerVolume = function (): {
  mute: () => void;
  unmute: () => void;
  toggleVolume: () => void;
  volume: Ref<number>;
} {
  const volume = ref(1);
  const mute = () => {
    volume.value = 0;
  };
  const unmute = () => {
    volume.value = 1;
  };
  const toggleVolume = () => (volume.value > 0 ? mute() : unmute());
  return {
    mute,
    unmute,
    toggleVolume,
    volume,
  };
};
