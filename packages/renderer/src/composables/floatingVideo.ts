import {ref} from 'vue';

const floatingVideoSource = ref<string>();
const floatingVideoInitialTime = ref<number>();
const floatingVideoRelease = ref<{
  id: string;
  name: string;
  contentCID: string;
} | undefined>();

const closeFloatingVideo = () => {
  floatingVideoSource.value = undefined;
  floatingVideoInitialTime.value = undefined;
  floatingVideoRelease.value = undefined;
};

export const useFloatingVideo = () => {
  return {
    floatingVideoSource,
    floatingVideoInitialTime,
    floatingVideoRelease,
    closeFloatingVideo,
  };
};
