import { defineStore } from 'pinia';
import { suivre as follow } from '@constl/vue';
import { useOrbiter } from '../plugins/orbiter/utils';
import { computed, ref, watch, type Ref } from 'vue';
import { useStaticReleases } from '../composables/staticReleases';
import { useStaticStatus } from '../composables/staticStatus';

type ContentStatus = 'loading' | 'idle' | 'empty' | 'partial';

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
  promoted: boolean;
};

export type PartialFeaturedReleaseItem = Partial<FeaturedReleaseItem>;

export function filterActivedFeatured(featured: FeaturedReleaseItem) {
  const now = new Date();
  const startTime = new Date(featured.startTime);
  const endTime = new Date(featured.endTime);

  return now >= startTime && now <= endTime;
};

export function filterPromotedFeatured(featured: FeaturedReleaseItem) {
  return featured.promoted;
};

const determineTargetStatus = (
  currentStatus: ContentStatus,
  isStatic: boolean,
  isLoaded: boolean,
  isPartiallyLoaded: boolean,
  hasContent: boolean,
): ContentStatus => {
  if (isStatic) {
    return hasContent ? 'idle' : 'empty';
  }
  if (!isLoaded && !isPartiallyLoaded) {
    return 'loading';
  }
  if (hasContent) {
    return isLoaded ? 'idle' : 'partial';
  }
  return isLoaded ? 'empty' : 'loading';
};

export const useReleasesStore = defineStore('releases', () => {
  const { orbiter } = useOrbiter();
  const { staticReleases, staticFeaturedReleases } = useStaticReleases();
  const { staticStatus } = useStaticStatus();

  const syncComplete = ref(false);
  const partialSync = ref(false);
  
  const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));
  
  
  const orbiterFeaturedReleases = follow(orbiter.listenForSiteFeaturedReleases.bind(orbiter));

  const status: Ref<ContentStatus> = ref('loading');

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
        promoted: fr.featured.promoted,
      }));
    }
  });

  const activedFeaturedReleases = computed<ReleaseItem[]>(() => {
    const activedFeaturedReleasesIds = unfilteredFeaturedReleases.value
      .filter(filterActivedFeatured)
      .map(fr => fr.releaseId);
    return releases.value.filter(r => r.id && activedFeaturedReleasesIds.includes(r.id));
  });

  const promotedFeaturedReleases = computed<ReleaseItem[]>(() => {
    const promotedActivedFeaturedReleasesIds = unfilteredFeaturedReleases.value
      .filter(filterActivedFeatured)
      .filter(filterPromotedFeatured)
      .map(fr => fr.releaseId);
    return releases.value.filter(r => r.id && promotedActivedFeaturedReleasesIds.includes(r.id));
  });

  watch(
    [orbiterReleases, orbiterFeaturedReleases, staticStatus, syncComplete, partialSync],
    ([_currentOrbiterReleases, _currentOrbiterFeaturedRels, currentStaticMode]) => {
      const isStatic = currentStaticMode === 'static';
      const isLoaded = isStatic || syncComplete.value;
      const isPartiallyLoaded = partialSync.value;
      const hasContentNow = activedFeaturedReleases.value.length > 0 || promotedFeaturedReleases.value.length > 0;
      
      const newTargetStatus = determineTargetStatus(
        status.value, 
        isStatic, 
        isLoaded, 
        isPartiallyLoaded, 
        hasContentNow,
      );
      
      if (status.value !== newTargetStatus) {
        status.value = newTargetStatus;
      }
    },
    { immediate: true, deep: false },
  );
  

  const isLoading = computed(() => status.value === 'loading');
  const isPartiallyLoaded = computed(() => status.value === 'partial');
  const noContent = computed(() => status.value === 'empty');

  return {
    releases,
    unfilteredFeaturedReleases,
    activedFeaturedReleases,
    promotedFeaturedReleases,
    isLoading,
    isPartiallyLoaded,
    noContent,
  };
});
