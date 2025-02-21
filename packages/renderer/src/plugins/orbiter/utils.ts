import {inject} from 'vue';

import type {Orbiter} from '@riffcc/orbiter';

export const useOrbiter = () => {
  const orbiter = inject<Orbiter>('orbiter')!;
  return {orbiter};
};
