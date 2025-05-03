import { defineStore } from 'pinia';
import { suivre as follow } from '@constl/vue';
import { useOrbiter } from '../plugins/orbiter/utils';
import { computed, ref, type Ref } from 'vue';
import type { TvSeries } from '../@types/release';
import { useStaticStatus } from '../composables/staticStatus';
// Import static data if needed, similar to releases store
// import { useStaticTvSeries } from '../composables/staticTvSeries'; // Assuming this exists

// Placeholder type for Orbiter's TV Series data structure
// Adjust based on actual Orbiter implementation
type OrbiterTvSeriesData = {
  id: string;
  series: {
    name: string;
    description?: string;
    thumbnail?: string;
    cover?: string;
    // Add other fields returned by Orbiter
  };
  site: string; // Source site ID
};

export const useTvSeriesStore = defineStore('tvSeries', () => {
  const { orbiter } = useOrbiter();
  const { staticStatus } = useStaticStatus();
  // const { staticTvSeries } = useStaticTvSeries(); // If static mode is needed

  // --- State ---
  const orbiterTvSeries = follow<OrbiterTvSeriesData[] | undefined>(orbiter.listenForTvSeries.bind(orbiter)); // Assuming this method exists
  const status: Ref<'loading' | 'idle' | 'empty' | 'error'> = ref('loading'); // Simplified status

  // --- Getters ---
  const tvSeries = computed<TvSeries[]>(() => {
    if (staticStatus.value === 'static') {
      // return staticTvSeries.value; // Handle static mode if needed
      return []; // Placeholder for static mode
    } else {
      if (orbiterTvSeries.value === undefined) {
        status.value = 'loading';
        return [];
      }
      if (orbiterTvSeries.value === null) {
        // Handle potential null return from follow/Orbiter if applicable
        status.value = 'error'; // Or 'empty' depending on Orbiter behavior
        return [];
      }

      const mappedSeries = (orbiterTvSeries.value || [])
      .filter(s => s && s.series) // Add filter to ensure s.series exists
      .map((s): TvSeries => ({
        id: s.id,
        name: s.series.name,
        description: s.series.description,
        thumbnail: s.series.thumbnail,
        cover: s.series.cover,
        sourceSite: s.site,
        // Map other fields as needed
      }));

      status.value = mappedSeries.length > 0 ? 'idle' : 'empty';
      return mappedSeries;
    }
  });

  const isLoading = computed(() => status.value === 'loading');
  const noContent = computed(() => status.value === 'empty');

  // --- Actions ---
  // Add actions for adding/editing/deleting series if needed, e.g.:
  // async function addSeries(seriesData: PartialTvSeries) { ... await orbiter.addTvSeries(seriesData); ... }
  // async function editSeries(seriesId: string, seriesData: PartialTvSeries) { ... await orbiter.editTvSeries({ seriesId, series: seriesData }); ... }
  // async function deleteSeries(seriesId: string) { ... await orbiter.deleteTvSeries(seriesId); ... }

  // Fetch a single series by ID (useful for detail pages)
  const getSeriesById = (id: string): ComputedRef<TvSeries | undefined> => {
    return computed(() => tvSeries.value.find(s => s.id === id));
  };


  return {
    tvSeries,
    isLoading,
    noContent,
    getSeriesById,
    // Expose actions like addSeries, editSeries, deleteSeries here
  };
});
