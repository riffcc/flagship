import {inject} from 'vue';

import type {Orbiter} from '@riffcc/orbiter';

export const useOrbiter = () => {
  const orbiter = inject<Orbiter>('orbiter');
  if (orbiter) return {orbiter};
  throw new Error('Orbiter not found. Did you register the Orbiter plugin with Vue?');
};
