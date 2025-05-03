import { defineStore } from 'pinia';
import { suivre as follow } from '@constl/vue';
import { useOrbiter } from '../plugins/orbiter/utils';
import { computed, onScopeDispose, ref, watch, type Ref } from 'vue';
import type { ReleaseItem, FeaturedReleaseItem } from '../@types/release';
import { filterActivedFeature } from '../utils';
import { useStaticReleases } from '../composables/staticReleases';
import { useStaticStatus } from '../composables/staticStatus';

const NO_CONTENT_DELAY_MS = 20000;

type ContentStatus = 'loading' | 'checking' | 'idle' | 'empty';
const determineTargetStatus = (
  currentStatus: ContentStatus,
  isStatic: boolean,
  isLoaded: boolean,
  hasContent: boolean,
): ContentStatus => {
  if (isStatic) {
    return hasContent ? 'idle' : 'empty';
  }
  if (!isLoaded) {
    return 'loading';
  }
  if (hasContent) {
    return 'idle';
  }
  return currentStatus === 'empty' ? 'empty' : 'checking';
};

export const useReleasesStore = defineStore('releases', () => {
  const { orbiter } = useOrbiter();
  const { staticReleases, staticFeaturedReleases } = useStaticReleases();
  const { staticStatus } = useStaticStatus();

  const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));
  const orbiterFeaturedReleases = follow(orbiter.listenForSiteFeaturedReleases.bind(orbiter));

  const status: Ref<ContentStatus> = ref('loading');
  const timerId = ref<ReturnType<typeof setTimeout> | null>(null);

  const releases = computed<ReleaseItem[]>(() => {
    if (staticStatus.value === 'static') return staticReleases.value;
    else {
      return (orbiterReleases.value || []).map((r) => ({
        id: r.release.id,
        name: r.release.release.contentName,
        contentCID: r.release.release.file,
        category: r.release.release.category,
        author: r.release.release.author,
        thumbnail: r.release.release.thumbnail,
       cover: r.release.release.cover,
       metadata: r.release.release.metadata ? JSON.parse(r.release.release.metadata as string) : {},
       sourceSite: r.site,
       // Add mapping for new TV Show fields (adjust keys based on actual Orbiter response)
       seriesId: r.release.release.seriesId, // Example key - Adjust if Orbiter uses different field names
       seasonNumber: r.release.release.seasonNumber, // Example key - Adjust if Orbiter uses different field names
       episodeNumber: r.release.release.episodeNumber, // Example key - Adjust if Orbiter uses different field names
     }) as ReleaseItem); // Correctly close the object literal before type assertion
   }
 });

  const featuredReleases = computed<FeaturedReleaseItem[]>(() => {
    if (staticStatus.value === 'static') return staticFeaturedReleases.value.filter(filterActivedFeature);
    else {
      return (orbiterFeaturedReleases.value || []).map((fr): FeaturedReleaseItem => ({
        id: fr.id,
        releaseId: fr.featured.releaseId,
        startTime: fr.featured.startTime,
        endTime: fr.featured.endTime,
      })).filter(filterActivedFeature);
    }
  });

  watch(
    [releases, featuredReleases, staticStatus],
    ([rels, featuredRels, staticMode]) => {
      // Calculate inputs for status determination
      const isStatic = staticMode === 'static';
      // For dynamic, both must be defined (not undefined). For static, it's always considered loaded.
      const isLoaded = isStatic || (rels !== undefined && featuredRels !== undefined);
      // Check content based on mode
      const hasContent = releases.value.length > 0 || featuredReleases.value.length > 0;
      const targetStatus = determineTargetStatus(status.value, isStatic, isLoaded, hasContent);

      const shouldBeChecking = targetStatus === 'checking';
      const timerIsRunning = timerId.value !== null;

      if (shouldBeChecking && !timerIsRunning) {
        timerId.value = setTimeout(() => {
          const stillEmpty = releases.value.length === 0 &&
            featuredReleases.value.length === 0;
          if (status.value === 'checking') {
            const finalStatus = stillEmpty ? 'empty' : 'idle';
            status.value = finalStatus;
          }
          timerId.value = null;
        }, NO_CONTENT_DELAY_MS);

      } else if (!shouldBeChecking && timerIsRunning) {
        if (timerId.value) {
          clearTimeout(timerId.value);
          timerId.value = null;
        }
      }
      if (status.value !== targetStatus) {
        status.value = targetStatus;
      }
    },
    { immediate: true, deep: false },
  );
  onScopeDispose(() => {
    if (timerId.value !== null) {
      clearTimeout(timerId.value);
    }
  });

  const isLoading = computed(() => status.value === 'loading' || status.value === 'checking');
  const noContent = computed(() => status.value === 'empty');

  return {
    releases,
    featuredReleases,
    isLoading,
    noContent,
  };
});
