import { defineStore } from 'pinia';
import { suivre as follow } from '@constl/vue';
import { useOrbiter } from '../plugins/orbiter/utils';
import { computed, onScopeDispose, ref, watch } from 'vue';
import type { ReleaseItem, FeaturedReleaseItem } from '../@types/release';
import { filterActivedFeature } from '../utils';
import { useStaticReleases } from '../composables/staticReleases';
import { useStaticStatus } from '../composables/staticStatus';

const NO_CONTENT_DELAY_MS = 7000;

export const useReleasesStore = defineStore('releases', () => {
  const { orbiter } = useOrbiter();
  const {staticReleases, staticFeaturedReleases} = useStaticReleases();
  const {staticStatus} = useStaticStatus();

  const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));
  const orbiterFeaturedReleases = follow(orbiter.listenForSiteFeaturedReleases.bind(orbiter));


  const initialLoadComplete = computed<boolean>(() => {
    if (staticStatus.value === 'static') {
      return true;
    } else {
      return orbiterReleases.value !== undefined && orbiterFeaturedReleases.value !== undefined;
    }
  });
  const releases = computed<ReleaseItem[]>(() => {
    if (staticStatus.value === 'static') return staticReleases.value;
    else {
      return (orbiterReleases.value || []).map((r) => {
        return {
          id: r.release.id,
          name: r.release.release.contentName,
          contentCID: r.release.release.file,
          category: r.release.release.category,
          author: r.release.release.author,
          thumbnail: r.release.release.thumbnail,
          cover: r.release.release.cover,
          metadata: r.release.release.metadata ? JSON.parse(r.release.release.metadata as string) : {},
          sourceSite: r.site,
        };
      }) as ReleaseItem[];
    }
  });

  const featuredReleases = computed<FeaturedReleaseItem[]>(() => {
    if (staticStatus.value === 'static') return staticFeaturedReleases.value.filter(fr => filterActivedFeature(fr));
    else {
      return (orbiterFeaturedReleases.value || []).map((fr): FeaturedReleaseItem => {
        return {
          id: fr.id,
          releaseId: fr.featured.releaseId,
          startTime: fr.featured.startTime,
          endTime: fr.featured.endTime,
        };
      }).filter(fr => filterActivedFeature(fr));
    }
  });

  const waitingForContentConfirmation = ref(false);
  const isContentConfirmedEmpty = ref(false);
  const noContentTimerId = ref<ReturnType<typeof setTimeout> | null>(null);

  watch(
    [initialLoadComplete, releases, featuredReleases],
    ([isLoaded, currentReleases, currentFeatured]) => {
      if (noContentTimerId.value !== null) {
        clearTimeout(noContentTimerId.value);
        noContentTimerId.value = null;
      }
      waitingForContentConfirmation.value = false;
      isContentConfirmedEmpty.value = false;

      const currentlyEmpty = isLoaded && currentReleases.length === 0 && currentFeatured.length === 0;

      if (currentlyEmpty) {
        waitingForContentConfirmation.value = true;
        noContentTimerId.value = setTimeout(() => {
          isContentConfirmedEmpty.value = true;
          noContentTimerId.value = null;
        }, NO_CONTENT_DELAY_MS);
      }
    },
    { immediate: false },
  );

  onScopeDispose(() => {
    if (noContentTimerId.value !== null) {
      clearTimeout(noContentTimerId.value);
    }
  });

  const noContent = computed(() => isContentConfirmedEmpty.value);
  const isLoading = computed(() => !initialLoadComplete.value || waitingForContentConfirmation.value);
  return {
    releases,
    featuredReleases,
    isLoading,
    noContent,
  };
});
