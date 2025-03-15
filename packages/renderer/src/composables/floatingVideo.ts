import {ref} from 'vue';

const floatingVideoSource = ref<string>();
const floatingVideoInitialTime = ref<number>();

const closeFloatingVideo = () => {
  floatingVideoSource.value = undefined;
  floatingVideoInitialTime.value = undefined;
};

export const useFloatingVideo = () => {
  return {
    floatingVideoSource,
    floatingVideoInitialTime,
    closeFloatingVideo,
  };
};
