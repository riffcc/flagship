import { inject, type Ref } from 'vue';
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query';
import { API_URL } from '../router';
import type {
  HashResponse,
  IdResponse,
  AnyObject,
  LensService,
  ReleaseData,
  SearchOptions,
  FeaturedReleaseData,
  SubscriptionData,
  AddInput,
  EditInput,
} from '@riffcc/lens-sdk';
import type { ContentCategoryItem, FeaturedReleaseItem, ReleaseItem } from '/@/types';

export function useLensService() {
  const lensService = inject<LensService>('lensService');
  if (!lensService) {
    throw new Error('Lens Service plugin not initialized.');
  }
  return { lensService };
}

// #### QUERIES ####
export function usePublicKeyQuery(options?: { enabled?: boolean | Ref<boolean> }) {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['publicKey'],
    queryFn: () => {
      return lensService.peerbit?.identity.publicKey.toString();
    },
    enabled: options?.enabled ?? true,
  });
}

export function usePeerIdQuery(options?: { enabled?: boolean | Ref<boolean> }) {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['peerId'],
    queryFn: () => {
      return lensService.peerbit?.peerId.toString();
    },
    enabled: options?.enabled ?? true,
  });
}

export function useAccountStatusQuery(options?: { enabled?: boolean | Ref<boolean> }) {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['accountStatus'],
    queryFn: async () => {
      return await lensService.getAccountStatus();
    },
    refetchInterval: 15000,
    enabled: options?.enabled ?? true,
  });
}

export function useGetReleaseQuery(id: string, options?: { enabled?: boolean | Ref<boolean> }) {
  const { lensService } = useLensService();
  return useQuery<ReleaseItem | undefined>({
    queryKey: ['release', id],
    queryFn: async () => {
      const r = await lensService.getRelease(id);
      return r ?
        {
          ...r,
          metadata: r.metadata ? JSON.parse(r.metadata) : undefined,
        } : undefined;
    },
    enabled: options?.enabled ?? true,
  });
}

export function useGetReleasesQuery(options?: {
  enabled?: boolean | Ref<boolean>,
  staleTime?: number,
  searchOptions?: SearchOptions,
}) {
  const { lensService } = useLensService();
  return useQuery<ReleaseItem[]>({
    queryKey: ['releases'],
    queryFn: async () => {
      // PeerBit-only loading with optimized timeouts
      // Default to fetching 100 releases at a time
      const searchOptions = {
        fetch: 100,
        ...options?.searchOptions,
      };
      const result = await lensService.getReleases(searchOptions);
      return result.map((r) => {
        return {
          ...r,
          metadata: r.metadata ? JSON.parse(r.metadata) : undefined,
        };
      });
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
    gcTime: 1000 * 60 * 15,
  });
}

export function useGetFeaturedReleaseQuery(id: string) {
  const { lensService } = useLensService();
  return useQuery<FeaturedReleaseItem | undefined>({
    queryKey: ['featuredRelease', id],
    queryFn: async () => {
      return await lensService.getFeaturedRelease(id);
    },
  });
}

export function useGetFeaturedReleasesQuery(options?: {
  enabled?: boolean | Ref<boolean>,
  staleTime?: number,
  searchOptions?: SearchOptions,
}) {
  const { lensService } = useLensService();
  return useQuery<FeaturedReleaseItem[]>({
    queryKey: ['featuredReleases'],
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
  });
}

export function useContentCategoriesQuery(options?: {
  enabled?: boolean | Ref<boolean>;
}) {
  const { lensService } = useLensService();
  return useQuery<ContentCategoryItem[]>({
    queryKey: ['contentCategories'],
    queryFn: async () => {
      const result = await lensService.getContentCategories();
      return result.map((c) => {
        return {
          ...c,
          metadataSchema: c.metadataSchema ? JSON.parse(c.metadataSchema) : undefined,
        };
      });
    },
    enabled: options?.enabled ?? true,
  });
}

export function useGetSubscriptionsQuery(options?: {
  enabled?: boolean | Ref<boolean>,
  staleTime?: number,
  searchOptions?: SearchOptions,
}) {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['subscriptions'],
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
    mutationFn: async (data: AddInput<ReleaseData<AnyObject>>) => {
      return await lensService.addRelease({
        ...data,
        metadata: data.metadata ? JSON.stringify(data.metadata) : undefined,
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
    mutationFn: async (data: EditInput<ReleaseData<AnyObject>>) => {
      return await lensService.editRelease({
        ...data,
        metadata: data.metadata ? JSON.stringify(data.metadata) : undefined,
      });
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['releases', response.id] });
      // Also invalidate featured releases in case this release is featured
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
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
    mutationFn: async (id: string) => {
      return await lensService.deleteRelease(id);
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
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
    mutationFn: async (data: AddInput<FeaturedReleaseData>) => {
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
    mutationFn: async (data: EditInput<FeaturedReleaseData>) => {
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
    mutationFn: async (id: string) => {
      return await lensService.deleteFeaturedRelease(id);
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
    mutationFn: async (data: AddInput<SubscriptionData>) => {
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
    mutationFn: async (data: { id?: string; to?: string }) => {
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

// #### STRUCTURE QUERIES & MUTATIONS ####
export function useGetStructureQuery(id: string, options?: { enabled?: boolean | Ref<boolean> }) {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['structure', id],
    queryFn: async () => {
      try {
        // Try API first for immediate data
        const apiUrl = `${API_URL}/structures/${id}`;
        try {
          const response = await fetch(apiUrl);
          if (response.ok) {
            const structure = await response.json();
            return structure ? {
              ...structure,
              metadata: structure.metadata ? JSON.parse(structure.metadata) : undefined,
            } : null;
          }
        } catch (apiError) {
          console.warn('API fetch failed, falling back to Peerbit:', apiError);
        }
        
        // Fallback to Peerbit if API fails
        const structure = await lensService.getStructure(id);
        return structure ? {
          ...structure,
          metadata: structure.metadata ? JSON.parse(structure.metadata) : undefined,
        } : null;
      } catch (error) {
        console.error('Failed to fetch structure:', error);
        return null;
      }
    },
    enabled: options?.enabled ?? true,
    retry: 1,
  });
}

export function useGetStructuresQuery(options?: {
  enabled?: boolean | Ref<boolean>;
  staleTime?: number;
  searchOptions?: SearchOptions;
}) {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['structures', options?.searchOptions],
    queryFn: async () => {
      try {
        // Try API first for immediate data
        const apiUrl = `${API_URL}/structures`;
        try {
          const response = await fetch(apiUrl);
          if (response.ok) {
            const structures = await response.json();
            return structures.map((s: any) => ({
              ...s,
              metadata: s.metadata ? JSON.parse(s.metadata) : undefined,
            }));
          }
        } catch (apiError) {
          console.warn('API fetch failed, falling back to Peerbit:', apiError);
        }
        
        // Fallback to Peerbit if API fails
        const structures = await lensService.getStructures(options?.searchOptions);
        return structures.map(s => ({
          ...s,
          metadata: s.metadata ? JSON.parse(s.metadata) : undefined,
        }));
      } catch (error) {
        console.error('Failed to fetch structures:', error);
        return [];
      }
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
    retry: 1,
  });
}

export function useAddStructureMutation(options?: {
  onSuccess?: (response: HashResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: AddInput<{
      name: string;
      type: string;
      description?: string;
      thumbnailCID?: string;
      bannerCID?: string;
      parentId?: string;
      itemIds?: string[];
      metadata?: AnyObject;
      order?: number;
    }>) => {
      return await lensService.addStructure({
        ...data,
        metadata: data.metadata ? JSON.stringify(data.metadata) : undefined,
      });
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['structures'] });
      queryClient.invalidateQueries({ queryKey: ['artists'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useEditStructureMutation(options?: {
  onSuccess?: (response: HashResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: EditInput<{
      name: string;
      type: string;
      description?: string;
      thumbnailCID?: string;
      bannerCID?: string;
      parentId?: string;
      itemIds?: string[];
      metadata?: AnyObject;
      order?: number;
    }>) => {
      return await lensService.editStructure({
        ...data,
        metadata: data.metadata ? JSON.stringify(data.metadata) : undefined,
      });
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['structures'] });
      queryClient.invalidateQueries({ queryKey: ['structure', response.id] });
      queryClient.invalidateQueries({ queryKey: ['artists'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useDeleteStructureMutation(options?: {
  onSuccess?: (response: IdResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      return await lensService.deleteStructure(id);
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['structures'] });
      queryClient.invalidateQueries({ queryKey: ['structure', response.id] });
      queryClient.invalidateQueries({ queryKey: ['artists'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

// #### ARTIST QUERIES & MUTATIONS (convenience wrappers) ####
export function useGetArtistQuery(id: string, options?: { enabled?: boolean | Ref<boolean> }) {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['artist', id],
    queryFn: async () => {
      const artist = await lensService.getArtist(id);
      return artist ? {
        ...artist,
        metadata: artist.metadata ? JSON.parse(artist.metadata) : undefined,
      } : undefined;
    },
    enabled: options?.enabled ?? true,
  });
}

export function useGetArtistsQuery(options?: {
  enabled?: boolean | Ref<boolean>;
  staleTime?: number;
}) {
  const { lensService } = useLensService();
  return useQuery({
    queryKey: ['artists'],
    queryFn: async () => {
      const artists = await lensService.getArtists();
      return artists.map(a => ({
        ...a,
        metadata: a.metadata ? JSON.parse(a.metadata) : undefined,
      }));
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 1000 * 60 * 5,
  });
}

export function useAddArtistMutation(options?: {
  onSuccess?: (response: HashResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: AddInput<{
      name: string;
      bio?: string;
      avatarCID?: string;
      bannerCID?: string;
      links?: string[];
      metadata?: AnyObject;
    }>) => {
      return await lensService.addArtist({
        ...data,
        metadata: data.metadata ? JSON.stringify(data.metadata) : undefined,
      });
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['artists'] });
      queryClient.invalidateQueries({ queryKey: ['structures'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useEditArtistMutation(options?: {
  onSuccess?: (response: HashResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: EditInput<{
      name: string;
      bio?: string;
      avatarCID?: string;
      bannerCID?: string;
      links?: string[];
      metadata?: AnyObject;
    }>) => {
      return await lensService.editArtist({
        ...data,
        metadata: data.metadata ? JSON.stringify(data.metadata) : undefined,
      });
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['artists'] });
      queryClient.invalidateQueries({ queryKey: ['artist', response.id] });
      queryClient.invalidateQueries({ queryKey: ['structures'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useDeleteArtistMutation(options?: {
  onSuccess?: (response: IdResponse) => void;
  onError?: (e: Error) => void;
}) {
  const { lensService } = useLensService();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      return await lensService.deleteArtist(id);
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['artists'] });
      queryClient.invalidateQueries({ queryKey: ['artist', response.id] });
      queryClient.invalidateQueries({ queryKey: ['structures'] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}
