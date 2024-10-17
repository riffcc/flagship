import {ref, type Ref} from 'vue';

export type StatusTypes = 'static' | 'live';
const status = ref<StatusTypes>('static');

export const useDevStatus = function (): {
  status: Ref<StatusTypes>;
} {
  return {
    status,
  };
};
