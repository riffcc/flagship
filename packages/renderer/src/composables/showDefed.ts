import {ref, type Ref} from 'vue';

const showDefederation = ref(false);
const showDHTDebug = ref(false);

export const useShowDefederation = function (): {
  showDefederation: Ref<boolean>;
  showDHTDebug: Ref<boolean>;
} {
  return {
    showDefederation,
    showDHTDebug,
  };
};
