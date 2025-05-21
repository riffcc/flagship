import { inject } from 'vue';
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query';
import type { HashResponse, IdResponse, AnyObject, LensService, ReleaseData, SearchOptions, IdData, FeaturedReleaseData } from '@riffcc/lens-sdk';
import {
  ID_PROPERTY,
  RELEASE_METADATA_PROPERTY,
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

export function useGetReleaseQuery(props: IdData) {
  const { staticStatus } = useStaticStatus();
  const { staticReleases } = useStaticData();
  const { lensService } = useLensService();
  return useQuery<ReleaseItem<AnyObject> | undefined>({
    queryKey: ['release', props.id],
    queryFn: async () => {
      if (staticStatus.value) {
        return staticReleases.value.find(x => x.id === props.id);
      } else {
        const r = await lensService.getRelease(props);
        const rMetadata = r?.[RELEASE_METADATA_PROPERTY];
        return r ?
          {
            [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.parse(rMetadata) : undefined,
            ...r,
          } : undefined;
      }
    },
  });
}

export function useGetReleasesQuery(options?: {
  enabled?: boolean,
  staleTime?: number,
  searchOptions?: SearchOptions,
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
        const result = await lensService.getReleases(options?.searchOptions);
        return result.map((r) => {
          const rMetadata = r?.[RELEASE_METADATA_PROPERTY];
          return {
            [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.parse(rMetadata) : undefined,
            ...r,
          };
        });
      }
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
    gcTime: 1000 * 60 * 15,
  });
}

export function useGetFeaturedReleaseQuery(props: IdData) {
  const { lensService } = useLensService();
  const { staticStatus } = useStaticStatus();
  const { staticFeaturedReleases } = useStaticData();
  return useQuery<FeaturedReleaseItem | undefined>({
    queryKey: ['featuredRelease', props.id],
    queryFn: async () => {
      if (staticStatus.value) {
        return staticFeaturedReleases.value.find(sfr => sfr.id === props.id);
      } else {
        return await lensService.getFeaturedRelease(props);
      }
    },
  });
}

export function useGetFeaturedReleasesQuery(options?: {
  enabled?: boolean,
  staleTime?: number,
  searchOptions?: SearchOptions,
}) {
  const { lensService } = useLensService();
  const { staticStatus } = useStaticStatus();
  const { staticFeaturedReleases } = useStaticData();
  return useQuery<FeaturedReleaseItem[]>({
    queryKey: ['featuredReleases'],
    queryFn: async () => {
      if (staticStatus.value) {
        return staticFeaturedReleases.value;
      } else {
        return await lensService.getFeaturedReleases(options?.searchOptions);
      }
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
    gcTime: 1000 * 60 * 15,
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
  onSuccess?: (response: HashResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { staticStatus } = useStaticStatus();
  const { staticReleases } = useStaticData();
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: ReleaseData) => {
      if (staticStatus.value) {
        const srId = String(staticReleases.value.length + 1);
        const rMetadata = data[RELEASE_METADATA_PROPERTY];
        const srParsed = {
          ...data,
          [ID_PROPERTY]: srId,
          [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.parse(rMetadata) : undefined,
        };
        staticReleases.value.push(srParsed);
        return {
          success: true,
          id: srId,
          hash: 'test-hash',
        };
      } else {
        return await lensService.addRelease(data);
      }
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['releases'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useEditReleaseMutation(options?: {
  onSuccess?: (response: IdResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { staticStatus } = useStaticStatus();
  const { staticReleases } = useStaticData();
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData & ReleaseData) => {
      if (staticStatus.value) {
        staticReleases.value = staticReleases.value.filter(sr => sr.id !== data.id);
        const rMetadata = data[RELEASE_METADATA_PROPERTY];
        const srParsed = {
          ...data,
          [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.parse(rMetadata) : undefined,
        };
        staticReleases.value.push(srParsed);
        return {
          success: true,
          id: data.id,
          hash: 'test-hash',
        };
      } else {
        return await lensService.editRelease(data);
      }
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['releases', response.id] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useDeleteReleaseMutation(options?: {
  onSuccess?: (response: IdResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { staticStatus } = useStaticStatus();
  const { staticReleases } = useStaticData();
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData) => {
      if (staticStatus.value) {
        staticReleases.value = staticReleases.value.filter(sr => sr.id !== data.id);
        return {
          success: true,
          id: data.id,
          hash: 'test-hash',
        };
      } else {
        return await lensService.deleteRelease(data);
      }
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['releases', response.id] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useAddFeaturedReleaseMutation(options?: {
  onSuccess?: (response: HashResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { staticStatus } = useStaticStatus();
  const { staticFeaturedReleases } = useStaticData();
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: FeaturedReleaseData) => {
      if (staticStatus.value) {
        const srId = String(staticFeaturedReleases.value.length + 1);
        const srParsed = {
          ...data,
          [ID_PROPERTY]: srId,
        };
        staticFeaturedReleases.value.push(srParsed);
        return {
          success: true,
          id: srId,
          hash: 'test-hash',
        };
      } else {
        return await lensService.addFeaturedRelease(data);
      }
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useEditFeaturedReleaseMutation(options?: {
  onSuccess?: (response: IdResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { staticStatus } = useStaticStatus();
  const { staticFeaturedReleases } = useStaticData();
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData & FeaturedReleaseData) => {
      if (staticStatus.value) {
        staticFeaturedReleases.value = staticFeaturedReleases.value.filter(sfr => sfr.id !== data.id);
        staticFeaturedReleases.value.push(data);
        return {
          success: true,
          id: data.id,
          hash: 'test-hash',
        };
      } else {
        return await lensService.editFeaturedRelease(data);
      }
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
      queryClient.invalidateQueries({ queryKey: ['featuredReleases', response.id] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useDeleteFeaturedReleaseMutation(options?: {
  onSuccess?: (response: IdResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { staticStatus } = useStaticStatus();
  const { staticReleases } = useStaticData();
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData) => {
      if (staticStatus.value) {
        staticReleases.value = staticReleases.value.filter(sr => sr.id !== data.id);
        return {
          success: true,
          id: data.id,
          hash: 'test-hash',
        };
      } else {
        return await lensService.deleteFeaturedRelease(data);
      }
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
      queryClient.invalidateQueries({ queryKey: ['featuredReleases', response.id] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}
