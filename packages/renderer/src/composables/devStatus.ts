import {ref, type Ref} from 'vue';

export type StatusTypes = 'static' | 'live';

export const utiliserDevStatus = function (): {
  status: Ref<StatusTypes>;
} {
  const status = ref<StatusTypes>('static');

  return {
    status,
  };
};
