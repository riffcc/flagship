import { defineStore } from 'pinia';
import { useOrbiter } from '../plugins/peerbit/utils';
import { computed, onScopeDispose, ref, watch, type Ref, inject } from 'vue';
import type { types as orbiterTypes } from '../plugins/peerbit/orbiter-types';
import { useStaticReleases } from '../composables/staticReleases';
import { useStaticStatus } from '../composables/staticStatus';
import { type Documents, SearchRequest } from '@peerbit/document';
import type { Release as PeerbitRelease } from '../plugins/peerbit/schema';

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

  const peerbitReleaseStore = inject<Documents<PeerbitRelease>>('peerbitReleaseStore');
  const peerbitReleasesRaw = ref<PeerbitRelease[]>([]);

  const orbiterReleases = ref<orbiterTypes.ReleaseWithId<string>[]>([]);
  const orbiterFeaturedReleases = ref<orbiterTypes.FeaturedReleaseWithId[]>([]);

  // Commenting out Orbiter listeners to focus on Peerbit integration
  /*
  if (orbiter.value && orbiter.value.listenForReleases && !peerbitReleaseStore) { 
    orbiter.value.listenForReleases({
      f: (releases: orbiterTypes.ReleaseWithId<string>[]) => {
        orbiterReleases.value = releases;
      },
    });
  }
  if (orbiter.value && orbiter.value.listenForSiteFeaturedReleases && !peerbitReleaseStore) { 
    orbiter.value.listenForSiteFeaturedReleases({
      f: (featuredReleases: orbiterTypes.FeaturedReleaseWithId[]) => {
        orbiterFeaturedReleases.value = featuredReleases;
      },
    });
  }
  */

  const status: Ref<ContentStatus> = ref('loading');
  const timerId = ref<ReturnType<typeof setTimeout> | null>(null);

  const releases = computed<ReleaseItem[]>(() => {
    if (peerbitReleaseStore) {
      if (peerbitReleasesRaw.value.length > 0) {
        console.log('[ReleasesStore] Using Peerbit releases. Count:', peerbitReleasesRaw.value.length);
        return peerbitReleasesRaw.value.map((pr) => {
          let parsedMetadata: Record<string, unknown> = {};
          try {
            if (pr.metadata) {
              parsedMetadata = JSON.parse(pr.metadata);
            }
          } catch (e) {
            console.error('Failed to parse release metadata from Peerbit:', e);
          }
          return {
            id: pr.id,
            name: pr.name,
            contentCID: pr.contentCID,
            category: pr.categoryId,
            author: parsedMetadata.author as string || 'N/A',
            thumbnail: pr.thumbnailCID,
            cover: parsedMetadata.cover as string,
            metadata: parsedMetadata,
          };
        });
      } else {
        console.log('[ReleasesStore] Peerbit store active, but no releases loaded yet or store is empty.');
        return [];
      }
    } else if (staticStatus.value === 'static') {
      console.log('[ReleasesStore] Using static releases.');
      return staticReleases.value;
    } else {
      console.log('[ReleasesStore] Peerbit not configured. Falling back to Orbiter releases.');
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
    [orbiterReleases, orbiterFeaturedReleases, staticStatus, peerbitReleasesRaw],
    ([currentOrbiterReleases, currentOrbiterFeaturedRels, currentStaticMode, currentPeerbitReleases]) => {
      const isStatic = currentStaticMode === 'static';
      const hasPeerbitContent = currentPeerbitReleases && currentPeerbitReleases.length > 0;
      const isLoaded = isStatic || hasPeerbitContent || (currentOrbiterReleases !== undefined && currentOrbiterFeaturedRels !== undefined);
      const hasContentNow = hasPeerbitContent || activedFeaturedReleases.value.length > 0 || promotedFeaturedReleases.value.length > 0;
      
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

  async function fetchReleasesFromPeerbit() {
    if (!peerbitReleaseStore) {
      console.log('[ReleasesStore] Peerbit release store not available for fetching.');
      return;
    }
    try {
      console.log('[ReleasesStore] Fetching releases from Peerbit...');
      const results = await peerbitReleaseStore.index.search(new SearchRequest({ query: [] }));
      console.log('[ReleasesStore] Fetched from Peerbit, raw results:', results);
      peerbitReleasesRaw.value = results as PeerbitRelease[];
      console.log('[ReleasesStore] Parsed Peerbit releases:', peerbitReleasesRaw.value);
    } catch (error) {
      console.error('[ReleasesStore] Error fetching releases from Peerbit:', error);
      peerbitReleasesRaw.value = [];
    }
  }

  watch(
    () => peerbitReleaseStore,
    (newStore) => {
      if (newStore) {
        fetchReleasesFromPeerbit();
      } else {
        peerbitReleasesRaw.value = [];
      }
    },
    { immediate: true }
  );

  return {
    releases,
    fetchReleasesFromPeerbit,
    unfilteredFeaturedReleases,
    activedFeaturedReleases,
    promotedFeaturedReleases,
    isLoading,
    noContent,
  };
});
