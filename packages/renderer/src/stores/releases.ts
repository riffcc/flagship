import { defineStore } from 'pinia';
import { computed, onScopeDispose, ref, watch, type Ref } from 'vue';
import { useStaticReleases } from '../composables/staticReleases';
import { useStaticStatus } from '../composables/staticStatus';
import type { FeaturedRelease, Release } from '/@/lib/schema';
import { usePeerbitService } from '../plugins/peerbit/utils';
import type { FeaturedReleaseData, IdData, ReleaseData, AnyObject } from '/@/lib/types';
import { RELEASE_METADATA_PROPERTY } from '/@/lib/constants';

const NO_CONTENT_DELAY_MS = 20000;
type ContentStatus = 'loading' | 'checking' | 'idle' | 'empty';

export type ReleaseItem<T = string> = IdData & ReleaseData<T>;

export type PartialReleaseItem<T = string> = Partial<ReleaseItem<T>>;

export type FeaturedReleaseItem = IdData & FeaturedReleaseData;

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
  const { peerbitServiceRef } = usePeerbitService();
  const { staticReleases, staticFeaturedReleases } = useStaticReleases();
  const { staticStatus } = useStaticStatus();

  const releasesRaw = ref<Release[] | null>(null);
  const featuredReleasesRaw = ref<FeaturedRelease[] | null>(null);

  const status: Ref<ContentStatus> = ref('loading');
  const timerId = ref<ReturnType<typeof setTimeout> | null>(null);

  const releases = computed<ReleaseItem<AnyObject>[]>(() => {
    if (releasesRaw.value) {
      if (releasesRaw.value.length > 0) {
        console.log('[ReleasesStore] Using Peerbit releases. Count:', releasesRaw.value.length);
        return releasesRaw.value.map((pr) => {
          let parsedMetadata: AnyObject = {};
          try {
            if (pr.metadata) {
              parsedMetadata = JSON.parse(pr.metadata);
            }
          } catch (e) {
            console.error('Failed to parse release metadata from Peerbit:', e);
          }
          return {
            ...pr,
            [RELEASE_METADATA_PROPERTY]: parsedMetadata,
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
      console.log('[ReleasesStore] Peerbit not configured.');
      return [];
    }
  });

  const unfilteredFeaturedReleases = computed<FeaturedReleaseItem[]>(() => {
    if (staticStatus.value === 'static') return staticFeaturedReleases.value;
    else {
      return (featuredReleasesRaw.value || []).map((fr): FeaturedReleaseItem => ({
        id: fr.id,
        releaseId: fr.releaseId,
        startTime: fr.startTime,
        endTime: fr.endTime,
        promoted: fr.promoted,
      }));
    }
  });

  const activedFeaturedReleases = computed<ReleaseItem<AnyObject>[]>(() => {
    const activedFeaturedReleasesIds = unfilteredFeaturedReleases.value
      .filter(filterActivedFeatured)
      .map(fr => fr.releaseId);
    return releases.value.filter(r => r.id && activedFeaturedReleasesIds.includes(r.id));
  });

  const promotedFeaturedReleases = computed<ReleaseItem<AnyObject>[]>(() => {
    const promotedActivedFeaturedReleasesIds = unfilteredFeaturedReleases.value
      .filter(filterActivedFeatured)
      .filter(filterPromotedFeatured)
      .map(fr => fr.releaseId);
    return releases.value.filter(r => r.id && promotedActivedFeaturedReleasesIds.includes(r.id));
  });

  watch(
    [releasesRaw, featuredReleasesRaw, staticStatus],
    ([currentReleases, _currentFeaturedReleases, currentStaticMode]) => {
      const isStatic = currentStaticMode === 'static';
      const hasPeerbitContent = currentReleases && currentReleases.length > 0;
      const isLoaded = isStatic || !!hasPeerbitContent;
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
    // Ensure the peerbitServiceRef Ref itself was injected AND its .value (the service instance) is available
    if (!peerbitServiceRef || !peerbitServiceRef.value) {
      console.warn('[ReleasesStore] fetchReleasesFromPeerbit: Peerbit service Ref not injected or service instance not available.');
      // Set to empty array or null, depending on how "no service" should be represented.
      // Empty array is consistent with error handling below.
      releasesRaw.value = [];
      return;
    }

    const serviceInstance = peerbitServiceRef.value; // Safe to use now
    try {
      console.log('[ReleasesStore] Fetching releases from Peerbit service instance...');
      const results = await serviceInstance.getLatestReleases();
      console.log('[ReleasesStore] Fetched from Peerbit, raw results:', results);
      releasesRaw.value = results;
      // console.log('[ReleasesStore] Parsed Peerbit releases:', releasesRaw.value); // Already logged by computed if needed
    } catch (error) {
      console.error('[ReleasesStore] Error fetching releases from Peerbit:', error);
      releasesRaw.value = []; // Set to empty on error
    }
  }

  // Watch for the Peerbit service to become available
  watch(
    () => peerbitServiceRef?.value, // Safely access .value from the potentially undefined ref
    (newServiceInstance, oldServiceInstance) => {
      if (newServiceInstance && !oldServiceInstance) {
        console.log('[ReleasesStore] Peerbit service became available. Fetching initial releases.');
        fetchReleasesFromPeerbit();
      } else if (!newServiceInstance && oldServiceInstance) {
        console.log('[ReleasesStore] Peerbit service became unavailable.');
        releasesRaw.value = null; // Or [], to clear data if service disappears
        status.value = 'loading'; // Reset status
      }
    },
    { immediate: true }, // immediate:true ensures it runs on setup and on change
  );

  // The onMounted hook calling fetchReleasesFromPeerbit() was removed.
  // Fetching is now triggered by the watcher when the peerbitServiceRef becomes available.

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
