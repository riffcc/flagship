import {readonly, ref, watch, type Ref} from 'vue';

export type StatusTypes = 'static' | 'live';
const status = ref<StatusTypes>('static');
const alreadyChanged = ref(false);
watch(status, ()=>{
  alreadyChanged.value = true;
});

export const useDevStatus = function (): {
  status: Ref<StatusTypes>;
  alreadyChanged: Readonly<Ref<boolean>>;
} {
  return {
    status,
    alreadyChanged: readonly(alreadyChanged),
  };
};
