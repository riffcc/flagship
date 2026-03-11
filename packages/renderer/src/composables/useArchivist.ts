/**
 * Archivist Upload Client
 *
 * Uploads files to Archivist nodes with multi-URL failover support.
 * Archivist stores files and returns CIDs.
 *
 * Single file: POST /api/archivist/v1/data
 *   - Body: raw binary stream (application/octet-stream)
 *   - Headers: Content-Type, Content-Disposition (for filename)
 *   - Returns: plain text CID
 *
 * Directory (two-phase upload):
 *   Phase 1: Upload each file via POST /api/archivist/v1/data
 *   Phase 2: POST /api/archivist/v1/directory
 *     - Body: JSON { entries: [{ path, cid, size, mimetype? }] }
 *     - Returns: JSON { cid, totalSize, filesCount }
 *   Benefits: per-file retry, resumable, no timeout issues
 *
 * Environment:
 *   VITE_ARCHIVIST_API_URL - Comma-separated list of Archivist endpoints
 *                           Default: https://uploads.island.riff.cc
 */

import { ref, computed } from 'vue';

// Default to local Archivist proxy (via nginx for CORS)
const DEFAULT_ARCHIVIST_URL = 'https://uploads.island.riff.cc';

// Parse comma-separated URLs from env
function getArchivistUrls(): string[] {
  const envUrls = import.meta.env.VITE_ARCHIVIST_API_URL as string | undefined;
  if (envUrls) {
    return envUrls.split(',').map(url => url.trim()).filter(Boolean);
  }
  return [DEFAULT_ARCHIVIST_URL];
}

export interface UploadProgress {
  loaded: number;
  total: number;
  percent: number;
}

export interface UploadResult {
  success: boolean;
  cid?: string;
  fileName: string;
  relativePath?: string;
  error?: string;
  archivistUrl?: string; // Which node handled the upload
}

export interface DirectoryUploadResult {
  success: boolean;
  cid?: string; // Root CID for entire directory
  fileCount: number;
  error?: string;
  archivistUrl?: string;
}

export interface ArchivistNodeInfo {
  url: string;
  alive: boolean;
  lastCheck: Date;
  location?: string;
}

/**
 * State for tracking individual file uploads in parallel
 */
export interface FileUploadState {
  id: string;           // Unique ID for this upload
  fileName: string;     // Display name
  relativePath: string; // Full path within directory
  size: number;         // File size in bytes
  status: 'pending' | 'uploading' | 'complete' | 'error';
  progress: number;     // 0-100 percent
  loaded: number;       // Bytes uploaded
  cid?: string;         // CID once complete
  error?: string;       // Error message if failed
}

// Module-level state for node health tracking
const nodeHealth = ref<Map<string, ArchivistNodeInfo>>(new Map());
const lastHealthCheck = ref<Date | null>(null);

/**
 * Upload a single file to Archivist with multi-URL failover
 */
async function uploadFile(
  file: File,
  options?: {
    relativePath?: string;
    publicKey?: string;
    sign?: (message: string) => Promise<string>;
    onProgress?: (progress: UploadProgress) => void;
    signal?: AbortSignal;
  },
): Promise<UploadResult> {
  const urls = getArchivistUrls();
  let lastError: Error | null = null;

  for (const baseUrl of urls) {
    try {
      const result = await uploadToNode(baseUrl, file, options);

      // Update health status on success
      nodeHealth.value.set(baseUrl, {
        url: baseUrl,
        alive: true,
        lastCheck: new Date(),
      });

      return {
        success: true,
        cid: result.cid,
        fileName: file.name,
        relativePath: options?.relativePath,
        archivistUrl: baseUrl,
      };
    } catch (error) {
      console.warn(`[Archivist] Upload to ${baseUrl} failed:`, error);
      lastError = error instanceof Error ? error : new Error(String(error));

      // Update health status on failure
      nodeHealth.value.set(baseUrl, {
        url: baseUrl,
        alive: false,
        lastCheck: new Date(),
      });
    }
  }

  return {
    success: false,
    fileName: file.name,
    relativePath: options?.relativePath,
    error: lastError?.message || 'All Archivist nodes failed',
  };
}

/**
 * Upload to a specific Archivist node
 *
 * Archivist API: POST /api/archivist/v1/data
 * - Body: raw binary (application/octet-stream)
 * - Headers: Content-Type, Content-Disposition for filename
 * - Returns: plain text CID
 */
async function uploadToNode(
  baseUrl: string,
  file: File,
  options?: {
    relativePath?: string;
    publicKey?: string;
    sign?: (message: string) => Promise<string>;
    onProgress?: (progress: UploadProgress) => void;
    signal?: AbortSignal;
  },
): Promise<{ cid: string }> {
  // Generate auth headers if sign function provided
  let timestamp: string | undefined;
  let signature: string | undefined;
  if (options?.sign && options?.publicKey) {
    timestamp = Math.floor(Date.now() / 1000).toString();
    signature = await options.sign(`${timestamp}:UPLOAD`);
  }

  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();

    // Progress tracking
    if (options?.onProgress) {
      xhr.upload.addEventListener('progress', (event) => {
        if (event.lengthComputable) {
          options.onProgress!({
            loaded: event.loaded,
            total: event.total,
            percent: Math.round((event.loaded / event.total) * 100),
          });
        }
      });
    }

    // Handle completion
    xhr.addEventListener('load', () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        // Archivist returns plain text CID
        const cid = xhr.responseText.trim();
        if (cid) {
          resolve({ cid });
        } else {
          reject(new Error('No CID in response'));
        }
      } else if (xhr.status === 403) {
        reject(new Error('Upload permission denied'));
      } else if (xhr.status === 422) {
        const serverError = xhr.responseText || 'Validation error';
        console.error(`[Archivist] 422 - file: "${file.name}", error: "${serverError}"`);
        reject(new Error(serverError));
      } else {
        reject(new Error(`HTTP ${xhr.status}: ${xhr.statusText}`));
      }
    });

    xhr.addEventListener('error', () => {
      reject(new Error('Network error'));
    });

    xhr.addEventListener('abort', () => {
      reject(new Error('Upload aborted'));
    });

    xhr.addEventListener('timeout', () => {
      reject(new Error('Upload timeout'));
    });

    // Handle abort signal
    if (options?.signal) {
      options.signal.addEventListener('abort', () => {
        xhr.abort();
      });
    }

    // Archivist streaming upload API
    xhr.open('POST', `${baseUrl}/api/archivist/v1/data`);
    xhr.timeout = 5 * 60 * 1000; // 5 minute timeout for large files

    // Set content headers for Archivist
    xhr.setRequestHeader('Content-Type', file.type || 'application/octet-stream');
    const filename = options?.relativePath || file.name;
    xhr.setRequestHeader('Content-Disposition', `attachment; filename="${filename}"`);

    // Set auth headers for Caddy forward_auth validation
    if (options?.publicKey) {
      xhr.setRequestHeader('X-Pubkey', options.publicKey);
    }
    if (timestamp) {
      xhr.setRequestHeader('X-Timestamp', timestamp);
    }
    if (signature) {
      xhr.setRequestHeader('X-Signature', signature);
    }

    // Send raw file data
    xhr.send(file);
  });
}

/**
 * Upload multiple files with progress tracking
 */
async function uploadFiles(
  files: File[],
  options?: {
    preservePaths?: boolean;
    onFileProgress?: (fileName: string, progress: UploadProgress) => void;
    onFileComplete?: (result: UploadResult) => void;
    signal?: AbortSignal;
  },
): Promise<UploadResult[]> {
  const results: UploadResult[] = [];

  for (const file of files) {
    if (options?.signal?.aborted) {
      results.push({
        success: false,
        fileName: file.name,
        error: 'Upload cancelled',
      });
      continue;
    }

    const relativePath = options?.preservePaths
      ? (file as any).webkitRelativePath || file.name
      : undefined;

    const result = await uploadFile(file, {
      relativePath,
      onProgress: options?.onFileProgress
        ? (progress) => options.onFileProgress!(file.name, progress)
        : undefined,
      signal: options?.signal,
    });

    results.push(result);
    options?.onFileComplete?.(result);
  }

  return results;
}

/**
 * Upload a directory (multiple files) as a single unit, getting one root CID
 *
 * Two-phase upload process:
 * 1. Upload files in parallel (configurable concurrency)
 * 2. Finalize directory structure with path→CID mappings
 *
 * Benefits:
 * - Parallel uploads for speed (like Estuary.tech)
 * - Per-file progress tracking with real-time state updates
 * - Each file upload can be retried independently
 * - No timeout issues with large directories
 */
async function uploadDirectory(
  files: File[],
  options?: {
    publicKey?: string;
    sign?: (message: string) => Promise<string>;
    concurrency?: number;  // Max parallel uploads (default: 4)
    onProgress?: (progress: UploadProgress) => void;
    onFileProgress?: (fileName: string, progress: UploadProgress) => void;
    onFileComplete?: (fileName: string, cid: string) => void;
    onFileStates?: (states: FileUploadState[]) => void;  // Real-time state for all files
    signal?: AbortSignal;
  },
): Promise<DirectoryUploadResult> {
  if (files.length === 0) {
    return { success: false, fileCount: 0, error: 'No files to upload' };
  }

  const urls = getArchivistUrls();
  let lastError: Error | null = null;
  const concurrency = options?.concurrency ?? 4;

  // Initialize file states for UI
  const fileStates: FileUploadState[] = files.map((file, index) => ({
    id: `file-${index}-${Date.now()}`,
    fileName: file.name,
    relativePath: (file as any).webkitRelativePath || file.name,
    size: file.size,
    status: 'pending' as const,
    progress: 0,
    loaded: 0,
  }));

  // Notify UI of initial state
  options?.onFileStates?.(fileStates);

  // Helper to update and broadcast file state
  const updateFileState = (index: number, updates: Partial<FileUploadState>) => {
    Object.assign(fileStates[index], updates);
    options?.onFileStates?.([...fileStates]); // Spread to trigger reactivity
  };

  // Calculate overall progress from all file states
  const broadcastOverallProgress = () => {
    const totalSize = fileStates.reduce((sum, f) => sum + f.size, 0);
    const totalLoaded = fileStates.reduce((sum, f) => sum + f.loaded, 0);
    options?.onProgress?.({
      loaded: totalLoaded,
      total: totalSize,
      percent: totalSize > 0 ? Math.round((totalLoaded / totalSize) * 100) : 0,
    });
  };

  // Try each node until one works
  for (const baseUrl of urls) {
    try {
      // Reset states for retry on new node
      fileStates.forEach((state, i) => {
        if (state.status !== 'complete') {
          updateFileState(i, { status: 'pending', progress: 0, loaded: 0, error: undefined });
        }
      });

      // Phase 1: Upload files in parallel with concurrency limit
      const entries: Array<{ path: string; cid: string; size: number; mimetype?: string }> = [];
      let activeUploads = 0;
      let nextFileIndex = 0;
      const errors: Error[] = [];

      await new Promise<void>((resolve, reject) => {
        const startNextUpload = () => {
          // Check for abort
          if (options?.signal?.aborted) {
            reject(new Error('Upload aborted'));
            return;
          }

          // Find next pending file
          while (nextFileIndex < files.length && fileStates[nextFileIndex].status !== 'pending') {
            nextFileIndex++;
          }

          // No more files to start
          if (nextFileIndex >= files.length) {
            if (activeUploads === 0) {
              if (errors.length > 0) {
                reject(errors[0]);
              } else {
                resolve();
              }
            }
            return;
          }

          const fileIndex = nextFileIndex;
          const file = files[fileIndex];
          const state = fileStates[fileIndex];
          nextFileIndex++;
          activeUploads++;

          updateFileState(fileIndex, { status: 'uploading' });

          uploadToNode(baseUrl, file, {
            relativePath: state.relativePath,
            publicKey: options?.publicKey,
            sign: options?.sign,
            onProgress: (progress) => {
              updateFileState(fileIndex, {
                progress: progress.percent,
                loaded: progress.loaded,
              });
              options?.onFileProgress?.(file.name, progress);
              broadcastOverallProgress();
            },
            signal: options?.signal,
          })
            .then((result) => {
              updateFileState(fileIndex, {
                status: 'complete',
                progress: 100,
                loaded: file.size,
                cid: result.cid,
              });

              entries.push({
                path: state.relativePath,
                cid: result.cid,
                size: file.size,
                mimetype: file.type || undefined,
              });

              options?.onFileComplete?.(file.name, result.cid);
              broadcastOverallProgress();

              activeUploads--;
              startNextUpload();
            })
            .catch((error) => {
              const errorMsg = error instanceof Error ? error.message : String(error);
              updateFileState(fileIndex, {
                status: 'error',
                error: errorMsg,
              });

              errors.push(error instanceof Error ? error : new Error(errorMsg));
              activeUploads--;

              // Continue with other files even if one fails
              startNextUpload();
            });
        };

        // Start initial batch of uploads
        for (let i = 0; i < concurrency && i < files.length; i++) {
          startNextUpload();
        }
      });

      // Check if all files uploaded successfully
      const failedFiles = fileStates.filter(s => s.status === 'error');
      if (failedFiles.length > 0) {
        throw new Error(`${failedFiles.length} file(s) failed to upload`);
      }

      // Phase 2: Finalize directory structure
      const dirResult = await finalizeDirectory(baseUrl, entries, options?.publicKey, options?.sign);

      // Update health status on success
      nodeHealth.value.set(baseUrl, {
        url: baseUrl,
        alive: true,
        lastCheck: new Date(),
      });

      return {
        success: true,
        cid: dirResult.cid,
        fileCount: files.length,
        archivistUrl: baseUrl,
      };
    } catch (error) {
      console.warn(`[Archivist] Directory upload to ${baseUrl} failed:`, error);
      lastError = error instanceof Error ? error : new Error(String(error));

      nodeHealth.value.set(baseUrl, {
        url: baseUrl,
        alive: false,
        lastCheck: new Date(),
      });
    }
  }

  return {
    success: false,
    fileCount: files.length,
    error: lastError?.message || 'All Archivist nodes failed',
  };
}

/**
 * Finalize a directory from pre-uploaded files
 *
 * Archivist API: POST /api/archivist/v1/directory
 *   - Body: JSON with entries array (path, cid, size, mimetype)
 *   - Returns: JSON { cid, totalSize, filesCount }
 */
async function finalizeDirectory(
  baseUrl: string,
  entries: Array<{ path: string; cid: string; size: number; mimetype?: string }>,
  publicKey?: string,
  sign?: (message: string) => Promise<string>,
): Promise<{ cid: string }> {
  // Generate auth headers if sign function provided
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (publicKey) {
    headers['X-Pubkey'] = publicKey;
  }
  if (sign && publicKey) {
    const timestamp = Math.floor(Date.now() / 1000).toString();
    const signature = await sign(`${timestamp}:UPLOAD`);
    headers['X-Timestamp'] = timestamp;
    headers['X-Signature'] = signature;
  }

  const response = await fetch(`${baseUrl}/api/archivist/v1/directory`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ entries }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Directory finalize failed: ${response.status} ${errorText}`);
  }

  const result = await response.json();
  if (!result.cid) {
    throw new Error('No CID in directory response');
  }

  return { cid: result.cid };
}

/**
 * Check health of all configured Archivist nodes
 */
async function checkNodeHealth(): Promise<ArchivistNodeInfo[]> {
  const urls = getArchivistUrls();
  const results: ArchivistNodeInfo[] = [];

  await Promise.all(
    urls.map(async (url) => {
      try {
        const response = await fetch(`${url}/health`, {
          method: 'GET',
          signal: AbortSignal.timeout(5000),
        });

        const info: ArchivistNodeInfo = {
          url,
          alive: response.ok,
          lastCheck: new Date(),
        };

        nodeHealth.value.set(url, info);
        results.push(info);
      } catch {
        const info: ArchivistNodeInfo = {
          url,
          alive: false,
          lastCheck: new Date(),
        };

        nodeHealth.value.set(url, info);
        results.push(info);
      }
    }),
  );

  lastHealthCheck.value = new Date();
  return results;
}

/**
 * Get nodes that store a specific CID
 * (For future use when Citadel integration is complete)
 */
async function getNodesForCid(cid: string): Promise<ArchivistNodeInfo[]> {
  // TODO: Query Citadel witness node for which Archivist nodes have this CID
  // For now, return all known healthy nodes
  const urls = getArchivistUrls();
  return urls.map(url => ({
    url,
    alive: nodeHealth.value.get(url)?.alive ?? true,
    lastCheck: nodeHealth.value.get(url)?.lastCheck ?? new Date(),
  }));
}

/**
 * Main composable export
 */
export function useArchivist() {
  const isUploading = ref(false);
  const uploadQueue = ref<{ file: File; progress: number }[]>([]);

  const archivistUrls = computed(() => getArchivistUrls());
  const healthyNodes = computed(() =>
    Array.from(nodeHealth.value.values()).filter(n => n.alive),
  );

  return {
    // State
    isUploading,
    uploadQueue,
    archivistUrls,
    healthyNodes,
    nodeHealth,
    lastHealthCheck,

    // Methods
    uploadFile,
    uploadFiles,
    uploadDirectory,
    checkNodeHealth,
    getNodesForCid,
  };
}

// Also export individual functions for direct use
export { uploadFile, uploadFiles, uploadDirectory, checkNodeHealth, getNodesForCid };
