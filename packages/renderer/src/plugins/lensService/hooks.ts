import { inject, type Ref, computed, unref } from 'vue';
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query';
import { API_URL } from '../router';
import { useIdentity } from '/@/composables/useIdentity';
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
  const { publicKey, isInitialized } = useIdentity();
  return useQuery({
    queryKey: ['accountStatus'],
    queryFn: async () => {
      if (!publicKey.value) {
        console.warn('[useAccountStatusQuery] Public key not available yet');
        return { isAdmin: false, roles: [], permissions: [] };
      }

      const encodedKey = encodeURIComponent(publicKey.value);
      console.log('[useAccountStatusQuery] Fetching account status for:', publicKey.value);
      const response = await fetch(`${API_URL}/account/${encodedKey}`);

      if (!response.ok) {
        console.warn('[useAccountStatusQuery] API returned error:', response.status);
        return { isAdmin: false, roles: [], permissions: [] };
      }

      const result = await response.json();
      console.log('[useAccountStatusQuery] Account status:', result);
      return result;
    },
    refetchInterval: 15000,
    // Only run when identity is initialized and publicKey is available
    enabled: computed(() => {
      const shouldEnable = (options?.enabled !== false) && isInitialized.value && !!publicKey.value;
      console.log('[useAccountStatusQuery] Query enabled:', shouldEnable, {
        optionsEnabled: options?.enabled,
        isInitialized: isInitialized.value,
        hasPublicKey: !!publicKey.value
      });
      return shouldEnable;
    }),
  });
}

export function useGetReleaseQuery(id: string | Ref<string>, options?: { enabled?: boolean | Ref<boolean> }) {
  const { lensService } = useLensService();

  const actualId = computed(() => unref(id));

  return useQuery<ReleaseItem | undefined>({
    queryKey: () => ['release', actualId.value],
    queryFn: async () => {
      const r = await lensService.getRelease(actualId.value);
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
  const USE_PEERBIT = import.meta.env.VITE_USE_PEERBIT === 'true';
  const { lensService } = useLensService();
  const queryClient = useQueryClient();

  return useQuery<ReleaseItem[]>({
    queryKey: ['releases'],
    queryFn: async () => {
      // When Peerbit is disabled, fetch from HTTP API
      if (!USE_PEERBIT) {
        // Always fetch fresh data from HTTP API (don't return stale cache)
        const response = await fetch(`${API_URL}/releases`);
        return await response.json();
      }

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
    staleTime: options?.staleTime ?? 0, // Always fetch fresh data
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
  const USE_PEERBIT = import.meta.env.VITE_USE_PEERBIT === 'true';
  const { lensService } = useLensService();
  const queryClient = useQueryClient();

  return useQuery<FeaturedReleaseItem[]>({
    queryKey: ['featuredReleases'],
    queryFn: async () => {
      // When Peerbit is disabled, return from cache or fetch from HTTP API
      if (!USE_PEERBIT) {
        const cached = queryClient.getQueryData<FeaturedReleaseItem[]>(['featuredReleases']);
        if (cached) return cached;

        // Fallback: fetch from HTTP API if cache miss
        const response = await fetch(`${API_URL}/featured-releases`);
        return await response.json();
      }

      // PeerBit-only loading with optimized timeouts
      // Fetch all featured releases (up to 1000 - should be more than enough)
      const searchOptions = {
        fetch: 1000,
        ...options?.searchOptions,
      };
      return await lensService.getFeaturedReleases(searchOptions);
    },
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? 0, // Always fetch fresh data
    gcTime: 1000 * 60 * 15,
  });
}

export function useContentCategoriesQuery(options?: {
  enabled?: boolean | Ref<boolean>;
}) {
  const USE_PEERBIT = import.meta.env.VITE_USE_PEERBIT === 'true';
  const { lensService } = useLensService();
  const queryClient = useQueryClient();

  return useQuery<ContentCategoryItem[]>({
    queryKey: ['contentCategories'],
    queryFn: async () => {
      // When Peerbit is disabled, return from cache or fetch from HTTP API
      if (!USE_PEERBIT) {
        const cached = queryClient.getQueryData<ContentCategoryItem[]>(['contentCategories']);
        if (cached) return cached;

        // Fallback: fetch from HTTP API if cache miss
        const response = await fetch(`${API_URL}/content-categories`);
        const categories = await response.json();
        return categories.map((c: any) => ({
          id: c.id,
          categoryId: c.id,
          name: c.name,
          displayName: c.name,
          slug: c.slug,
          metadataSchema: c.metadata_schema,
          siteAddress: c.siteAddress,
          featured: c.featured,
        }));
      }

      try {
        // Try API first for immediate data
        const apiUrl = `${API_URL}/content-categories`;
        try {
          const response = await fetch(apiUrl);
          if (response.ok) {
            const categories = await response.json();
            return categories.map((c: any) => ({
              id: c.id,
              categoryId: c.id, // Map for compatibility
              name: c.name,
              displayName: c.name, // Map for compatibility
              slug: c.slug,
              metadataSchema: c.metadata_schema, // Already an object from API
              siteAddress: c.siteAddress, // For filtering by site
            }));
          }
        } catch (apiError) {
          console.warn('[ContentCategories] API fetch failed, trying Peerbit:', apiError);
        }

        // Fallback to Peerbit if API fails
        const result = await lensService.getContentCategories();
        return result.map((c) => {
          return {
            ...c,
            metadataSchema: c.metadataSchema ? JSON.parse(c.metadataSchema) : undefined,
          };
        });
      } catch (error) {
        console.error('[ContentCategories] Failed to fetch categories:', error);
        return [];
      }
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
  const USE_PEERBIT = import.meta.env.VITE_USE_PEERBIT === 'true';
  const { lensService } = useLensService();
  const { publicKey, sign } = useIdentity();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: AddInput<ReleaseData<AnyObject>>) => {
      if (!USE_PEERBIT) {
        // Use HTTP API when Peerbit is disabled
        if (!publicKey.value) {
          throw new Error('Identity not initialized');
        }

        // Create request payload
        const payload = JSON.stringify({
          ...data,
          metadata: data.metadata,
        });

        // Sign the payload
        const timestamp = Date.now().toString();
        const messageToSign = `${timestamp}:${payload}`;
        const signature = await sign(messageToSign);

        console.log('[AddRelease] Signing request:', {
          publicKey: publicKey.value,
          timestamp,
          signatureLength: signature.length,
          payloadLength: payload.length,
        });

        const headers: Record<string, string> = {
          'Content-Type': 'application/json',
          'X-Public-Key': publicKey.value,
          'X-Signature': signature,
          'X-Timestamp': timestamp,
        };

        const response = await fetch(`${API_URL}/releases`, {
          method: 'POST',
          headers,
          body: payload,
        });
        if (!response.ok) {
          const error = await response.json().catch(() => ({ error: response.statusText }));
          console.error('[AddRelease] Request failed:', {
            status: response.status,
            statusText: response.statusText,
            error,
            headers: {
              publicKey: headers['X-Public-Key'],
              timestamp: headers['X-Timestamp'],
              signatureLength: headers['X-Signature']?.length,
            },
          });
          throw new Error(error.error || `Failed to create release: ${response.statusText}`);
        }
        const result = await response.json();
        // Immediately invalidate queries after successful HTTP API call
        queryClient.invalidateQueries({ queryKey: ['releases'] });
        return result;
      }

      return await lensService.addRelease({
        ...data,
        metadata: data.metadata ? JSON.stringify(data.metadata) : undefined,
      });
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      // Also invalidate here for Peerbit path
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
  const USE_PEERBIT = import.meta.env.VITE_USE_PEERBIT === 'true';
  const { lensService } = useLensService();
  const { publicKey, sign } = useIdentity();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (data: EditInput<ReleaseData<AnyObject>>) => {
      if (!USE_PEERBIT) {
        // Use HTTP API when Peerbit is disabled
        if (!publicKey.value) {
          throw new Error('Identity not initialized');
        }

        // Extract the data payload (could be nested as data.data or flat)
        const payload = (data as any).data || data;
        const releaseId = data.id;

        // Create request payload
        const payloadStr = JSON.stringify({
          name: payload.name,
          categoryId: payload.categoryId,
          contentCID: payload.contentCID,
          thumbnailCID: payload.thumbnailCID,
          metadata: payload.metadata,
          siteAddress: payload.siteAddress,
          postedBy: payload.postedBy,
        });

        // Sign the payload
        const timestamp = Date.now().toString();
        const messageToSign = `${timestamp}:${payloadStr}`;
        const signature = await sign(messageToSign);

        const headers: Record<string, string> = {
          'Content-Type': 'application/json',
          'X-Public-Key': publicKey.value,
          'X-Signature': signature,
          'X-Timestamp': timestamp,
        };

        const response = await fetch(`${API_URL}/releases/${releaseId}`, {
          method: 'PUT',
          headers,
          body: payloadStr,
        });
        if (!response.ok) {
          const error = await response.json().catch(() => ({ error: response.statusText }));
          throw new Error(error.error || `Failed to update release: ${response.statusText}`);
        }
        const result = await response.json();
        // Immediately invalidate queries after successful HTTP API call
        queryClient.invalidateQueries({ queryKey: ['releases'] });
        queryClient.invalidateQueries({ queryKey: ['releases', releaseId] });
        queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
        return result;
      }

      return await lensService.editRelease({
        ...data,
        metadata: data.metadata ? JSON.stringify(data.metadata) : undefined,
      });
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      // Also invalidate here for Peerbit path
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['releases', response.id] });
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
  const USE_PEERBIT = import.meta.env.VITE_USE_PEERBIT === 'true';
  const { lensService } = useLensService();
  const { publicKey, sign } = useIdentity();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      if (!USE_PEERBIT) {
        // Use HTTP API when Peerbit is disabled
        if (!publicKey.value) {
          throw new Error('Identity not initialized');
        }

        // Sign the request
        const timestamp = Date.now().toString();
        const messageToSign = `${timestamp}:DELETE:/releases/${id}`;
        const signature = await sign(messageToSign);

        const headers: Record<string, string> = {
          'X-Public-Key': publicKey.value,
          'X-Signature': signature,
          'X-Timestamp': timestamp,
        };

        const response = await fetch(`${API_URL}/releases/${id}`, {
          method: 'DELETE',
          headers,
        });

        if (!response.ok) {
          const error = await response.json().catch(() => ({ error: response.statusText }));
          throw new Error(error.error || `Failed to delete release: ${response.statusText}`);
        }

        const result = await response.json();
        // Immediately invalidate queries after successful HTTP API call
        queryClient.invalidateQueries({ queryKey: ['releases'] });
        queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
        return result;
      }

      return await lensService.deleteRelease(id);
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      // Also invalidate here for Peerbit path
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
      queryClient.invalidateQueries({ queryKey: ['releases', response.id] });
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

export function useBulkDeleteAllReleasesMutation(options?: {
  onSuccess?: (response: { success: boolean; deleted: number; delete_transaction_id: string }) => void;
  onError?: (e: Error) => void;
}) {
  const { publicKey, sign } = useIdentity();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      if (!publicKey.value) {
        throw new Error('Identity not initialized');
      }

      // Sign the request
      const timestamp = Date.now().toString();
      const messageToSign = `${timestamp}:DELETE:/releases`;
      const signature = await sign(messageToSign);

      const headers: Record<string, string> = {
        'X-Public-Key': publicKey.value,
        'X-Signature': signature,
        'X-Timestamp': timestamp,
      };

      const response = await fetch(`${API_URL}/releases`, {
        method: 'DELETE',
        headers,
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new Error(error.error || `Failed to delete all releases: ${response.statusText}`);
      }

      const result = await response.json();
      // Immediately invalidate queries after successful bulk delete
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
      return result;
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
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
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: EditInput<FeaturedReleaseData>) => {
      const { API_URL } = await import('../router');
      // URL-encode the ID to handle special characters
      const encodedId = encodeURIComponent(data.id);
      const response = await fetch(`${API_URL}/admin/featured-releases/${encodedId}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Failed to edit featured release: ${error}`);
      }

      return await response.json();
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

// HTTP-based hooks for direct API calls (UBTS transactions)

function useHttpLensService() {
  return inject<any>('httpLensService');
}

export function useHttpDeleteReleaseMutation(options?: {
  onSuccess?: (response: any) => void;
  onError?: (e: Error) => void;
}) {
  const { publicKey, sign } = useIdentity();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      if (!publicKey.value) {
        throw new Error('Identity not initialized');
      }

      // Sign the request
      const timestamp = Date.now().toString();
      const messageToSign = `${timestamp}:DELETE:/releases/${id}`;
      const signature = await sign(messageToSign);

      console.log('[DeleteRelease] Signing request:', {
        publicKey: publicKey.value,
        timestamp,
        releaseId: id,
        signatureLength: signature.length,
      });

      const headers: Record<string, string> = {
        'X-Public-Key': publicKey.value,
        'X-Signature': signature,
        'X-Timestamp': timestamp,
      };

      const response = await fetch(`${API_URL}/releases/${id}`, {
        method: 'DELETE',
        headers,
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        console.error('[DeleteRelease] Request failed:', {
          status: response.status,
          statusText: response.statusText,
          error,
          headers: {
            publicKey: headers['X-Public-Key'],
            timestamp: headers['X-Timestamp'],
            signatureLength: headers['X-Signature']?.length,
          },
        });
        throw new Error(error.error || `Failed to delete release: ${response.statusText}`);
      }

      const result = await response.json();
      return result;
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

export function useHttpDeleteFeaturedReleaseMutation(options?: {
  onSuccess?: (response: any) => void;
  onError?: (e: Error) => void;
}) {
  const { publicKey, sign } = useIdentity();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      if (!publicKey.value) {
        throw new Error('Identity not initialized');
      }

      // Sign the request
      const timestamp = Date.now().toString();
      const messageToSign = `${timestamp}:DELETE:/admin/featured-releases/${id}`;
      const signature = await sign(messageToSign);

      console.log('[DeleteFeaturedRelease] Signing request:', {
        publicKey: publicKey.value,
        timestamp,
        featuredReleaseId: id,
        signatureLength: signature.length,
      });

      const headers: Record<string, string> = {
        'X-Public-Key': publicKey.value,
        'X-Signature': signature,
        'X-Timestamp': timestamp,
      };

      // URL-encode the ID to handle special characters
      const encodedId = encodeURIComponent(id);
      const response = await fetch(`${API_URL}/admin/featured-releases/${encodedId}`, {
        method: 'DELETE',
        headers,
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        console.error('[DeleteFeaturedRelease] Request failed:', {
          status: response.status,
          statusText: response.statusText,
          error,
          headers: {
            publicKey: headers['X-Public-Key'],
            timestamp: headers['X-Timestamp'],
            signatureLength: headers['X-Signature']?.length,
          },
        });
        throw new Error(error.error || `Failed to delete featured release: ${response.statusText}`);
      }

      const result = await response.json();
      return result;
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

export function useHttpAuthorizeAdminMutation(options?: {
  onSuccess?: (response: any) => void;
  onError?: (e: Error) => void;
}) {
  const httpLensService = useHttpLensService();
  return useMutation({
    mutationFn: async (publicKey: string) => {
      return await httpLensService.authorizeAdmin(publicKey);
    },
    onSuccess: (response) => {
      options?.onSuccess?.(response);
    },
    onError: (error) => {
      options?.onError?.(error);
    },
  });
}

// WASM P2P hooks for browser-based P2P participation

function useWasmP2pService() {
  return inject<any>('wasmP2pService');
}

export function useWasmP2pDeleteReleaseMutation(options?: {
  onSuccess?: () => void;
  onError?: (e: Error) => void;
}) {
  const wasmP2pService = useWasmP2pService();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      if (!wasmP2pService) {
        throw new Error('WASM P2P service not initialized');
      }

      // Ensure service is initialized
      if (!wasmP2pService.isInitialized()) {
        await wasmP2pService.initialize();
      }

      // Delete via P2P
      await wasmP2pService.deleteRelease(id);
      return { id };
    },
    onSuccess: async (data) => {
      // Call user's onSuccess callback
      options?.onSuccess?.();

      // Optimistically remove the deleted release from cache (immediate UI update)
      queryClient.setQueryData(['releases'], (old: any) => {
        if (!old) return old;
        return old.filter((release: any) => release.id !== data.id);
      });

      queryClient.setQueryData(['featuredReleases'], (old: any) => {
        if (!old) return old;
        return old.filter((release: any) => release.id !== data.id);
      });

      // Small delay to let nodes process the delete before refetching
      // Network latency + node processing is typically 50-200ms
      setTimeout(() => {
        queryClient.invalidateQueries({ queryKey: ['releases'] });
        queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
        queryClient.invalidateQueries({ queryKey: ['release', data.id] });
      }, 200);
    },
    onError: (error, variables, context) => {
      // Revert optimistic updates on error
      queryClient.invalidateQueries({ queryKey: ['releases'] });
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
      options?.onError?.(error);
    },
  });
}

export function useWasmP2pDeleteFeaturedReleaseMutation(options?: {
  onSuccess?: () => void;
  onError?: (e: Error) => void;
}) {
  const wasmP2pService = useWasmP2pService();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      if (!wasmP2pService) {
        throw new Error('WASM P2P service not initialized');
      }

      // Ensure service is initialized
      if (!wasmP2pService.isInitialized()) {
        await wasmP2pService.initialize();
      }

      // Delete featured release via P2P
      await wasmP2pService.deleteFeaturedRelease(id);
      return { id };
    },
    onSuccess: async (data) => {
      // Call user's onSuccess callback
      options?.onSuccess?.();

      // Optimistically remove the deleted featured release from cache (immediate UI update)
      queryClient.setQueryData(['featuredReleases'], (old: any) => {
        if (!old) return old;
        return old.filter((featured: any) => featured.id !== data.id);
      });

      // Small delay to let nodes process the delete before refetching
      // Network latency + node processing is typically 50-200ms
      setTimeout(() => {
        queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
        queryClient.invalidateQueries({ queryKey: ['featuredRelease', data.id] });
      }, 200);
    },
    onError: (error, variables, context) => {
      // Revert optimistic updates on error
      queryClient.invalidateQueries({ queryKey: ['featuredReleases'] });
      options?.onError?.(error);
    },
  });
}
