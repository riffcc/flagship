import { inject } from 'vue';
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query';
import type {
  HashResponse,
  IdResponse,
  AnyObject,
  LensService,
  ReleaseData,
  SearchOptions,
  IdData,
  FeaturedReleaseData,
  SubscriptionData,
  Subscription,
} from '@riffcc/lens-sdk';
import {
  AccountType,
  RELEASE_METADATA_PROPERTY,
} from '@riffcc/lens-sdk';
import type { FeaturedReleaseItem, ReleaseItem } from '/@/types';
import { useStaticData } from '/@/composables/staticData';

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
    initialData: AccountType.GUEST,
    staleTime: 0, // Always consider data stale
    refetchInterval: 1000 * 30,
    refetchIntervalInBackground: true,
    refetchOnMount: 'always',
    refetchOnWindowFocus: false, // Don't refetch on window focus
    networkMode: 'offlineFirst', // Use cached data first
  });
}

export function useGetReleaseQuery(props: IdData) {
  const { lensService } = useLensService();
  return useQuery<ReleaseItem<AnyObject> | undefined>({
    queryKey: ['release', props.id],
    queryFn: async () => {
      const r = await lensService.getRelease(props);
      const rMetadata = r?.[RELEASE_METADATA_PROPERTY];
      return r ?
        {
          ...r,
          [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.parse(rMetadata) : undefined,
        } : undefined;
    },
  });
}

export function useGetReleasesQuery(options?: {
  enabled?: boolean,
  staleTime?: number,
  searchOptions?: SearchOptions,
}) {
  const { lensService } = useLensService();
  return useQuery<ReleaseItem<AnyObject>[]>({
    queryKey: ['releases', options?.searchOptions],
    queryFn: async () => {
      // PeerBit-only loading with optimized timeouts
      // Default to fetching 100 releases at a time
      const searchOptions = {
        fetch: 100,
        ...options?.searchOptions,
      };
      const result = await lensService.getReleases(searchOptions);
      return result.map((r) => {
        const rMetadata = r?.[RELEASE_METADATA_PROPERTY];
        return {
          ...r,
          [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.parse(rMetadata) : undefined,
        };
      });
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
    gcTime: 1000 * 60 * 15,
    refetchInterval: (data) => {
      if (data.state.data && !(data.state.data.length > 0) && data.state.dataUpdateCount > 0) {
        return 500;
      }
    },
    retry: (failureCount, error) => {
      // Handle specific PeerBit delivery errors with appropriate retry strategy
      if (error?.message?.includes('delivery acknowledges from all nodes (0/1)')) {
        return failureCount < 2; // Limited retry for node connectivity issues
      }
      if (error?.message?.includes('Failed to get message')) {
        return failureCount < 3; // More retries for message delivery issues
      }
      if (error?.message?.includes('try reducing fetch size')) {
        return false; // Don't retry timeout errors
      }
      return failureCount < 2;
    },
    retryDelay: (attemptIndex) => Math.min(500 * Math.pow(2, attemptIndex), 2000), // Exponential backoff
  });
}

export function useGetAllReleasesQuery(options?: {
  enabled?: boolean,
  staleTime?: number,
  onProgress?: (loaded: number, total: number) => void,
}) {
  const { lensService } = useLensService();
  return useQuery<ReleaseItem<AnyObject>[]>({
    queryKey: ['allReleases'],
    queryFn: async () => {
      const allReleases: ReleaseItem<AnyObject>[] = [];
      let hasMore = true;

      // First, get initial batch to see how many we might have
      while (hasMore) {
        const searchOptions: SearchOptions = {
          fetch: 100,
          // Note: Since lens-sdk doesn't support offset yet,
          // we're getting the first 100 releases multiple times
          // In production, you'd want to add offset support to lens-sdk
        };

        const result = await lensService.getReleases(searchOptions);

        // Transform the data
        const transformedBatch = result.map((r) => {
          const rMetadata = r?.[RELEASE_METADATA_PROPERTY];
          return {
            ...r,
            [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.parse(rMetadata) : undefined,
          };
        });

        // For now, since we can't paginate server-side, just get one batch
        // TODO: When lens-sdk supports offset, implement proper batching
        allReleases.push(...transformedBatch);
        hasMore = false; // Stop after first batch until offset is supported

        options?.onProgress?.(allReleases.length, allReleases.length);
      }

      return allReleases;
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
    gcTime: 1000 * 60 * 15,
  });
}

export function useGetFeaturedReleaseQuery(props: IdData) {
  const { lensService } = useLensService();
  return useQuery<FeaturedReleaseItem | undefined>({
    queryKey: ['featuredRelease', props.id],
    queryFn: async () => {
      return await lensService.getFeaturedRelease(props);
    },
  });
}

export function useGetFeaturedReleasesQuery(options?: {
  enabled?: boolean,
  staleTime?: number,
  searchOptions?: SearchOptions,
}) {
  const { lensService } = useLensService();
  return useQuery<FeaturedReleaseItem[]>({
    queryKey: ['featuredReleases', options?.searchOptions],
    queryFn: async () => {
      // PeerBit-only loading with optimized timeouts
      // Fetch all featured releases (up to 1000 - should be more than enough)
      const searchOptions = {
        fetch: 1000,
        ...options?.searchOptions,
      };
      return await lensService.getFeaturedReleases(searchOptions);
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
    gcTime: 1000 * 60 * 15,
    refetchInterval: (data) => {
      if (data.state.data && !(data.state.data.length > 0) && data.state.dataUpdateCount > 0) {
        return 500;
      }
    },
    retry: (failureCount, error) => {
      // Handle specific PeerBit delivery errors with appropriate retry strategy
      if (error?.message?.includes('delivery acknowledges from all nodes (0/1)')) {
        return failureCount < 2; // Limited retry for node connectivity issues
      }
      if (error?.message?.includes('Failed to get message')) {
        return failureCount < 3; // More retries for message delivery issues
      }
      if (error?.message?.includes('try reducing fetch size')) {
        return false; // Don't retry timeout errors
      }
      return failureCount < 2;
    },
    retryDelay: (attemptIndex) => Math.min(500 * Math.pow(2, attemptIndex), 2000), // Exponential backoff
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

export function useGetSubscriptionsQuery(options?: {
  enabled?: boolean,
  staleTime?: number,
  searchOptions?: SearchOptions,
}) {
  const { lensService } = useLensService();
  return useQuery<Subscription[]>({
    queryKey: ['subscriptions', options?.searchOptions],
    queryFn: async () => {
      const searchOptions = {
        fetch: 100,
        ...options?.searchOptions,
      };
      return await lensService.getSubscriptions(searchOptions);
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5, // 5 minutes
    gcTime: 1000 * 60 * 15, // 15 minutes
  });
}


// #### MUTATIONS ####

export function useAddReleaseMutation(options?: {
  onSuccess?: (response: HashResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: Omit<ReleaseData<AnyObject>, 'siteAddress'>) => {
      const rMetadata = data[RELEASE_METADATA_PROPERTY];
      return await lensService.addRelease({
        ...data,
        [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.stringify(rMetadata) : undefined,
      });
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
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData & ReleaseData<AnyObject>) => {
      const rMetadata = data[RELEASE_METADATA_PROPERTY];
      return await lensService.editRelease({
        ...data,
        [RELEASE_METADATA_PROPERTY]: rMetadata ? JSON.stringify(rMetadata) : undefined,
      });
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
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData) => {
      return await lensService.deleteRelease(data);
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
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: Omit<FeaturedReleaseData, 'siteAddress'>) => {
      return await lensService.addFeaturedRelease(data);
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
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData & FeaturedReleaseData) => {
      return await lensService.editFeaturedRelease(data);
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
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData) => {
      return await lensService.deleteFeaturedRelease(data);
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

export function useAddSubscriptionMutation(options?: {
  onSuccess?: (response: HashResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: SubscriptionData) => {
      return await lensService.addSubscription(data);
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useDeleteSubscriptionMutation(options?: {
  onSuccess?: (response: IdResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: IdData) => {
      return await lensService.deleteSubscription(data);
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['subscriptions'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}
