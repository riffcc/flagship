import { defineStore } from 'pinia';
import { useOrbiter } from '../plugins/peerbit/utils';
import { computed, onScopeDispose, ref, watch, type Ref } from 'vue';
import type { types as orbiterTypes } from '../plugins/peerbit/orbiter-types';
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

  const orbiterReleases = ref<orbiterTypes.ReleaseWithId<string>[]>([]);
  const orbiterFeaturedReleases = ref<orbiterTypes.FeaturedReleaseWithId[]>([]);

  if (orbiter) {
    if (orbiter.listenForReleases) {
      orbiter.listenForReleases({
        f: (releases: orbiterTypes.ReleaseWithId<string>[]) => {
          orbiterReleases.value = releases;
        },
      });
    }
    if (orbiter.listenForSiteFeaturedReleases) {
      orbiter.listenForSiteFeaturedReleases({
        f: (featuredReleases: orbiterTypes.FeaturedReleaseWithId[]) => {
          orbiterFeaturedReleases.value = featuredReleases;
        },
      });
    }
  }

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
    [orbiterReleases, orbiterFeaturedReleases, staticStatus],
    ([currentOrbiterReleases, currentOrbiterFeaturedRels, currentStaticMode]) => {
      const isStatic = currentStaticMode === 'static';
      const isLoaded = isStatic || (currentOrbiterReleases !== undefined && currentOrbiterFeaturedRels !== undefined);
      const hasContentNow = activedFeaturedReleases.value.length > 0 || promotedFeaturedReleases.value.length > 0;
      const newTargetStatus = determineTargetStatus(status.value, isStatic, isLoaded, hasContentNow);

      if (newTargetStatus !== 'checking' && timerId.value !== null) {
        clearTimeout(timerId.value);
        timerId.value = null;
      }

      if (newTargetStatus === 'checking' && timerId.value === null) {
        timerId.value = setTimeout(() => {
          const stillNoContentAfterDelay = activedFeaturedReleases.value.length === 0 && promotedFeaturedReleases.value.length === 0;

          if (status.value === 'checking') {
            status.value = stillNoContentAfterDelay ? 'empty' : 'idle';
          }
          timerId.value = null;
        }, NO_CONTENT_DELAY_MS);
      }

      if (status.value !== newTargetStatus) {
        status.value = newTargetStatus;
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
    activedFeaturedReleases,
    promotedFeaturedReleases,
    isLoading,
    noContent,
  };
});
