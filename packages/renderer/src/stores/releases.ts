import { defineStore } from 'pinia';
import { suivre as follow } from '@constl/vue';
import { useOrbiter } from '../plugins/orbiter/utils';
import { computed } from 'vue';
import type { ReleaseItem, FeaturedReleaseItem } from '../@types/release';
import { filterActivedFeature } from '../utils';
import { useStaticReleases } from '../composables/staticReleases';
import { useStaticStatus } from '../composables/staticStatus';

export const useReleasesStore = defineStore('releases', () => {
  const { orbiter } = useOrbiter();
  const {staticReleases, staticFeaturedReleases} = useStaticReleases();
  const {staticStatus} = useStaticStatus();

  const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));
  const orbiterFeaturedReleases = follow(orbiter.listenForSiteFeaturedReleases.bind(orbiter));

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
          metadata: JSON.parse(r.release.release.metadata as string),
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
  return { releases, featuredReleases, orbiterReleases, orbiterFeaturedReleases };
});
