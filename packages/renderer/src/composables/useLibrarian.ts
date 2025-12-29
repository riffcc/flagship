/**
 * Librarian API composable for Archive.org browsing and import management
 *
 * Uses TanStack Query for caching and real-time updates.
 * Only active when VITE_LIBRARIAN_API_URL is configured.
 */

import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';

// ============================================================================
// Configuration
// ============================================================================

/**
 * Get the Librarian API URL from environment, or null if not configured
 */
export function getLibrarianApiUrl(): string | null {
  const url = import.meta.env.VITE_LIBRARIAN_API_URL;
  return url && url.trim() !== '' ? url : null;
}

/**
 * Check if Librarian integration is enabled
 */
export function isLibrarianEnabled(): boolean {
  return getLibrarianApiUrl() !== null;
}

// ============================================================================
// Types
// ============================================================================

export interface LibrarianStatus {
  status: string;
  node_id: string;
  peer_count: number;
  pending_jobs: number;
  running_jobs: number;
  load: number;
}

export interface SearchResultItem {
  identifier: string;
  title: string | null;
  creator: string | null;
  mediatype: string | null;
  description: string | null;
  date: string | null;
  thumbnail_url: string;
}

export interface SearchResponse {
  total: number;
  page: number;
  rows: number;
  items: SearchResultItem[];
}

export interface ItemFile {
  name: string;
  format: string | null;
  size: string | null;
  source: string | null;
  stream_url: string;
  // Rich metadata from ID3/Vorbis tags
  title: string | null;
  artist: string | null;
  album: string | null;
  track: string | null;
  length: string | null;
  genre: string | null;
}

export interface ItemMetadata {
  title: string | null;
  creator: string | null;
  date: string | null;
  description: string | null;
  mediatype: string | null;
  collection: string[] | null;
}

export interface ItemResponse {
  identifier: string;
  metadata: ItemMetadata;
  files: ItemFile[];
}

export interface JobResult {
  type: 'Import' | 'Audit' | 'Transcode' | 'Migrate' | 'SourceImport' | 'Error';
  // Import result
  files_imported?: number;
  // Audit result
  missing_formats?: string[];
  source_quality?: string;
  // Transcode result
  outputs?: { quality: string; cid: string; size: number }[];
  // Migrate result
  old_cid?: string;
  new_cid?: string;
  size?: number;
  // SourceImport result
  source?: string;
  content_type?: string;
  // Error result
  message?: string;
}

/**
 * Parameters for source import (URL or CID)
 */
export interface SourceImportParams {
  /** The source URL or CID to import */
  source: string;
  /** Optional gateway URL for CID resolution */
  gateway?: string;
  /** If set, update this release's contentCID after import */
  existing_release_id?: string;
  /** Pre-authorized public key for Archivist uploads */
  pubkey?: string;
  /** Signature for Archivist authorization */
  signature?: string;
  /** Timestamp when signature was created */
  timestamp?: number;
}

/**
 * Response from source import endpoint
 */
export interface SourceImportResponse {
  /** Job ID (hex-encoded) */
  job_id: string;
  /** Current status */
  status: string;
}

export interface Job {
  id: string;
  job_type: string;
  target: string;
  status: string;
  created_at: number;
  claim_count: number;
  executor: string | null;
  result: JobResult | null;
  // Progress fields (set during execution)
  progress?: number;
  progress_message?: string;
}

export interface ImportFile {
  original_name: string;
  edited_name: string | null;
  format: string | null;
  size: number | null;
  selected: boolean;
}

export interface ImportMetadata {
  artist: string | null;
  album: string | null;
  year: number | null;
  tags: string[];
}

export interface Import {
  id: string;
  source: string;
  status: string;
  files: ImportFile[];
  metadata: ImportMetadata;
  auto_approve: boolean;
  created_at: number;
}

// ============================================================================
// API Client
// ============================================================================

async function librarianFetch<T>(path: string, options?: RequestInit): Promise<T> {
  const baseUrl = getLibrarianApiUrl();
  if (!baseUrl) {
    throw new Error('Librarian API not configured');
  }

  const response = await fetch(`${baseUrl}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Librarian API error: ${response.status} - ${error}`);
  }

  return response.json();
}

// ============================================================================
// Query Hooks
// ============================================================================

/**
 * Query Librarian daemon status
 */
export function useLibrarianStatus() {
  return useQuery({
    queryKey: ['librarian', 'status'],
    queryFn: () => librarianFetch<LibrarianStatus>('/api/v1/status'),
    enabled: isLibrarianEnabled(),
    refetchInterval: 10000, // Refresh every 10 seconds
  });
}

/**
 * Search Archive.org items via Librarian proxy
 */
export function useArchiveSearch(query: Ref<string>, options?: { page?: Ref<number>; rows?: number }) {
  const page = options?.page ?? ref(1);
  const rows = options?.rows ?? 24;

  return useQuery({
    queryKey: computed(() => ['librarian', 'archive', 'search', query.value, page.value]),
    queryFn: () => {
      const params = new URLSearchParams({
        q: query.value,
        page: String(page.value),
        rows: String(rows),
      });
      return librarianFetch<SearchResponse>(`/api/v1/archive/search?${params}`);
    },
    enabled: computed(() => isLibrarianEnabled() && query.value.trim().length > 0),
    staleTime: 1000 * 60 * 5, // Cache for 5 minutes
  });
}

/**
 * Get collection items from Archive.org
 */
export function useArchiveCollection(collectionId: Ref<string>, options?: { page?: Ref<number>; rows?: number }) {
  const page = options?.page ?? ref(1);
  const rows = options?.rows ?? 24;

  return useQuery({
    queryKey: computed(() => ['librarian', 'archive', 'collection', collectionId.value, page.value]),
    queryFn: () => {
      const params = new URLSearchParams({
        page: String(page.value),
        rows: String(rows),
      });
      return librarianFetch<SearchResponse>(`/api/v1/archive/collections/${encodeURIComponent(collectionId.value)}?${params}`);
    },
    enabled: computed(() => isLibrarianEnabled() && collectionId.value.trim().length > 0),
    staleTime: 1000 * 60 * 5,
  });
}

/**
 * Fetch ALL items in a collection (server-side pagination)
 * Use for "Select All in Collection" - returns all items in one request
 */
export async function fetchAllCollectionItems(collectionId: string): Promise<SearchResultItem[]> {
  const response = await librarianFetch<SearchResponse>(
    `/api/v1/archive/collections/${encodeURIComponent(collectionId)}/all`
  );
  return response.items;
}

/**
 * Get Archive.org item details
 */
export function useArchiveItem(itemId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => ['librarian', 'archive', 'item', itemId.value]),
    queryFn: () => librarianFetch<ItemResponse>(`/api/v1/archive/items/${encodeURIComponent(itemId.value)}`),
    enabled: computed(() => isLibrarianEnabled() && itemId.value.trim().length > 0),
    staleTime: 1000 * 60 * 10, // Cache for 10 minutes
  });
}

/**
 * List all jobs
 */
export function useLibrarianJobs() {
  return useQuery({
    queryKey: ['librarian', 'jobs'],
    queryFn: () => librarianFetch<Job[]>('/api/v1/jobs'),
    enabled: isLibrarianEnabled(),
    refetchInterval: 5000, // Refresh every 5 seconds
  });
}

/**
 * List imports
 */
export function useLibrarianImports() {
  return useQuery({
    queryKey: ['librarian', 'imports'],
    queryFn: () => librarianFetch<Import[]>('/api/v1/imports'),
    enabled: isLibrarianEnabled(),
    refetchInterval: 5000,
  });
}

// ============================================================================
// Mutation Hooks
// ============================================================================

/**
 * Create a new job
 */
export function useCreateJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (params: { job_type: string; target: string }) =>
      librarianFetch<Job>('/api/v1/jobs', {
        method: 'POST',
        body: JSON.stringify(params),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['librarian', 'jobs'] });
    },
  });
}

/**
 * Start/retry a job
 */
export function useStartJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (jobId: string) =>
      librarianFetch<Job>(`/api/v1/jobs/${jobId}/start`, {
        method: 'POST',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['librarian', 'jobs'] });
    },
  });
}

/**
 * Stop/cancel a job
 */
export function useStopJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (jobId: string) =>
      librarianFetch<Job>(`/api/v1/jobs/${jobId}/stop`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['librarian', 'jobs'] });
    },
  });
}

/**
 * Create a new import
 */
export function useCreateImport() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (params: {
      source: string;
      files: ImportFile[];
      metadata: ImportMetadata;
      auto_approve?: boolean;
    }) =>
      librarianFetch<Import>('/api/v1/imports', {
        method: 'POST',
        body: JSON.stringify(params),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['librarian', 'imports'] });
    },
  });
}

/**
 * Import content from a URL or CID
 *
 * Creates a job that fetches content from the source and uploads to Archivist.
 * Supports:
 * - HTTP/HTTPS URLs
 * - IPFS CIDs (Qm..., bafy...)
 * - Archivist CIDs (zD..., zE...)
 */
export function useCreateSourceImport() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (params: SourceImportParams) =>
      librarianFetch<SourceImportResponse>('/api/v1/sources/import', {
        method: 'POST',
        body: JSON.stringify(params),
      }),
    onSuccess: () => {
      // Invalidate jobs query to show the new import job
      queryClient.invalidateQueries({ queryKey: ['librarian', 'jobs'] });
    },
  });
}

// ============================================================================
// URL Helpers
// ============================================================================

/**
 * Get the proxied thumbnail URL for an Archive.org item
 */
export function getThumbnailUrl(identifier: string): string {
  const baseUrl = getLibrarianApiUrl();
  if (!baseUrl) return '';
  return `${baseUrl}/api/v1/archive/items/${encodeURIComponent(identifier)}/thumbnail`;
}

/**
 * Get the proxied stream URL for an Archive.org file
 */
export function getStreamUrl(identifier: string, filename: string): string {
  const baseUrl = getLibrarianApiUrl();
  if (!baseUrl) return '';
  return `${baseUrl}/api/v1/archive/items/${encodeURIComponent(identifier)}/stream/${encodeURIComponent(filename)}`;
}

// ============================================================================
// Main Composable
// ============================================================================

/**
 * Main Librarian composable providing all functionality
 */
export function useLibrarian() {
  const enabled = isLibrarianEnabled();
  const apiUrl = getLibrarianApiUrl();

  // Reactive search state
  const searchQuery = ref('');
  const currentPage = ref(1);
  const selectedItemId = ref('');

  // Queries
  const status = useLibrarianStatus();
  const searchResults = useArchiveSearch(searchQuery, { page: currentPage });
  const selectedItem = useArchiveItem(selectedItemId);
  const jobs = useLibrarianJobs();
  const imports = useLibrarianImports();

  // Mutations
  const createJob = useCreateJob();
  const createImport = useCreateImport();
  const createSourceImport = useCreateSourceImport();

  // Computed
  const pendingJobCount = computed(() =>
    jobs.data.value?.filter(j => j.status === 'Pending').length ?? 0
  );

  const runningJobCount = computed(() =>
    jobs.data.value?.filter(j => j.status === 'Running').length ?? 0
  );

  return {
    // Config
    enabled,
    apiUrl,

    // Search state
    searchQuery,
    currentPage,
    selectedItemId,

    // Queries
    status,
    searchResults,
    selectedItem,
    jobs,
    imports,

    // Mutations
    createJob,
    createImport,
    createSourceImport,

    // Computed
    pendingJobCount,
    runningJobCount,

    // URL helpers
    getThumbnailUrl,
    getStreamUrl,
  };
}

// ============================================================================
// Pending Count (for admin badge)
// ============================================================================

// Exported for use in adminPage.vue badge
export const pendingCount: ComputedRef<number> = computed(() => 0); // Will be updated by useAdminWebSocket pattern later
