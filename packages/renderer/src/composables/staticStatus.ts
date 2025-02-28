import {readonly, ref, watch, type Ref} from 'vue';

export type StaticStatusTypes = 'static' | 'live';

const staticStatus = ref<StaticStatusTypes>(import.meta.env.VITE_STATIC_MODE ? 'static' : 'live');

const alreadyChanged = ref(false);

watch(staticStatus, ()=>{
  alreadyChanged.value = true;
});

export const useStaticStatus = function (): {
  staticStatus: Ref<StaticStatusTypes>;
  alreadyChanged: Readonly<Ref<boolean>>;
} {
  return {
    staticStatus,
    alreadyChanged: readonly(alreadyChanged),
  };
};
