import { inject } from 'vue';
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query';
import type { AddReleaseResponse, AnyObject, LensService, ReleaseData } from '@riffcc/lens-sdk';
import {
  ID_PROPERTY,
  RELEASE_CATEGORY_ID_PROPERTY,
  RELEASE_CONTENT_CID_PROPERTY,
  RELEASE_METADATA_PROPERTY,
  RELEASE_NAME_PROPERTY,
  RELEASE_THUMBNAIL_CID_PROPERTY,
} from '@riffcc/lens-sdk';
import type { FeaturedReleaseItem, ReleaseItem } from '/@/types';
import { useStaticData } from '../../composables/staticData';
import { useStaticStatus } from '/@/composables/staticStatus';

export function useLensService() {
  const lensService = inject<LensService>('lensService');
  if (!lensService) {
    throw new Error('Lens Service plugin not initialized.');
  }
  return { lensService };
}

// #### QUERIES ####

export function usePublicKeyQuery() {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['publicKey'],
    queryFn: async () => {
      return await lensService.getPublicKey();
    },
  });
}

export function usePeerIdQuery() {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['peerId'],
    queryFn: async () => {
      return await lensService.getPeerId();
    },
  });
}

export function useAccountStatusQuery() {
  const { lensService } = useLensService();
  return useQuery({
  queryKey: ['accountStatus'],
  queryFn: async () => {
    return await lensService.getAccountStatus();
  },
  staleTime: 1000 * 60,
});
}

export function useReleaseQuery(id: string) {
  const { staticStatus } = useStaticStatus();
  const { staticReleases } = useStaticData();
  const { lensService } = useLensService();
  return useQuery<ReleaseItem<AnyObject> | undefined>({
    queryKey: ['release', id],
    queryFn: async () => {
      if (staticStatus.value) {
        return staticReleases.value.find(x => x.id === id);
      } else {
        const r = await lensService.getRelease(id);
        return r ?
          {
            [ID_PROPERTY]: r.id,
            [RELEASE_NAME_PROPERTY]: r.name,
            [RELEASE_CATEGORY_ID_PROPERTY]: r.categoryId,
            [RELEASE_CONTENT_CID_PROPERTY]: r.contentCID,
            [RELEASE_THUMBNAIL_CID_PROPERTY]: r.thumbnailCID,
            [RELEASE_METADATA_PROPERTY]: r.metadata ? JSON.parse(r.metadata) : undefined,
          } :
          undefined;
      }
    },
  });
}

export function useReleasesQuery(options?: {
  enabled?: boolean,
  staleTime?: number,
}) {
  const { lensService } = useLensService();
  const { staticStatus } = useStaticStatus();
  const { staticReleases } = useStaticData();
  return useQuery<ReleaseItem<AnyObject>[]>({
    queryKey: ['releases'],
    queryFn: async () => {
      if (staticStatus.value) {
        return staticReleases.value;
      } else {
        const result = await lensService.getLatestReleases();
        return result.map(r => ({
          [ID_PROPERTY]: r.id,
          [RELEASE_NAME_PROPERTY]: r.name,
          [RELEASE_CATEGORY_ID_PROPERTY]: r.categoryId,
          [RELEASE_CONTENT_CID_PROPERTY]: r.contentCID,
          [RELEASE_THUMBNAIL_CID_PROPERTY]: r.thumbnailCID,
          [RELEASE_METADATA_PROPERTY]: r.metadata ? JSON.parse(r.metadata) : undefined,
        }));;
      }
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
    gcTime: 1000 * 60 * 15,
  });
}

export function useFeaturedReleasesQuery() {
  // const { lensService } = useLensService();
  const { staticStatus } = useStaticStatus();
  const { staticFeaturedReleases } = useStaticData();
  return useQuery<FeaturedReleaseItem[]>({
  queryKey: ['featuredReleases'],
  queryFn: async () => {
    if (staticStatus.value) {
      return staticFeaturedReleases.value;
    } else {
      // Not implemented
      return [];
    }
  },
  });
}

export function useContentCategoriesQuery() {
  // const { lensService } = useLensService();
  // const { staticStatus } = useStaticStatus();
  const { staticContentCategories } = useStaticData();
  return useQuery({
  queryKey: ['contentCategories'],
  queryFn: async () => {
    // const result = await lensService.getContentCategories();
    return staticContentCategories;
  },
  initialData: staticContentCategories,
});
}


// #### MUTATIONS ####

export function useAddReleaseMutation(options?: {
  onSuccess?: (response: AddReleaseResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { staticStatus } = useStaticStatus();
  const { staticReleases } = useStaticData();
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation<AddReleaseResponse, Error, ReleaseData>({
  mutationFn: async (releaseData: ReleaseData) => {
    if (staticStatus.value) {
      const srId = String(staticReleases.value.length + 1);
      const srParsed = {
        [ID_PROPERTY]: srId,
        [RELEASE_NAME_PROPERTY]: releaseData.name,
        [RELEASE_CATEGORY_ID_PROPERTY]: releaseData.categoryId,
        [RELEASE_CONTENT_CID_PROPERTY]: releaseData.contentCID,
        [RELEASE_THUMBNAIL_CID_PROPERTY]: releaseData.thumbnailCID,
        [RELEASE_METADATA_PROPERTY]: releaseData.metadata ? JSON.parse(releaseData.metadata) : undefined,
      };
      staticReleases.value.push(srParsed);
      return {
        id: srId,
        hash: 'test-hash',
      };
    } else {
      return await lensService.addRelease(releaseData);
    }
  },
  onSuccess: (response) => {
    options?.onSuccess?.(response);
    queryClient.invalidateQueries({ queryKey: ['releases']});
  },
  onError: (error) => {
    options?.onError?.(error);
  },
});
}
