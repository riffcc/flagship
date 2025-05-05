import { defineStore } from 'pinia';
import { suivre as follow } from '@constl/vue';
import { useOrbiter } from '../plugins/orbiter/utils';
import { computed, onScopeDispose, ref, watch, type Ref } from 'vue';
import { useStaticReleases } from '../composables/staticReleases';
import { useStaticStatus } from '../composables/staticStatus';

const NO_CONTENT_DELAY_MS = 20000;
type ContentStatus = 'loading' | 'checking' | 'idle' | 'empty';

export type ReleaseItem = {
  id?: string;
  name: string;
  contentCID: string;
  category: string;
  author: string;
  thumbnail?: string;
  cover?: string;
  sourceSite?: string;
  metadata: Record<string, unknown>
}

export type PartialReleaseItem = Partial<ReleaseItem>;

export type FeaturedReleaseItem = {
  id: string;
  releaseId: string;
  startTime: string;
  endTime: string;
};

export type PartialFeaturedReleaseItem = Partial<FeaturedReleaseItem>;

export function filterActivedFeatured(featured: FeaturedReleaseItem) {
  const now = new Date();
  const startTime = new Date(featured.startTime);
  const endTime = new Date(featured.endTime);

  return now >= startTime && now <= endTime;
};
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
      })) as ReleaseItem[];
    }
  });

  const unfilteredFeaturedReleases = computed<FeaturedReleaseItem[]>(() => {
    if (staticStatus.value === 'static') return staticFeaturedReleases.value;
    else {
      return (orbiterFeaturedReleases.value || []).map((fr): FeaturedReleaseItem => ({
        id: fr.id,
        releaseId: fr.featured.releaseId,
        startTime: fr.featured.startTime,
        endTime: fr.featured.endTime,
      }));
    }
  });

  const featuredReleases = computed<FeaturedReleaseItem[]>(() => {
    return unfilteredFeaturedReleases.value.filter(filterActivedFeature);
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
    unfilteredFeaturedReleases,
    featuredReleases,
    isLoading,
    noContent,
  };
});
