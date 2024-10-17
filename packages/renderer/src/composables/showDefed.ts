import {ref, type Ref} from 'vue';

const showDefederation = ref(false);

export const useShowDefederation = function (): {
  showDefederation: Ref<boolean>;
} {
  return {
    showDefederation,
  };
};
