import { readonly, ref, watch } from 'vue';

const staticStatus = ref<boolean>(Boolean(import.meta.env.VITE_STATIC_STATUS));

const alreadyChanged = ref(false);

watch(staticStatus, () => {
  alreadyChanged.value = true;
});

export const useStaticStatus = () => {
  return {
    staticStatus,
    alreadyChanged: readonly(alreadyChanged),
  };
};
