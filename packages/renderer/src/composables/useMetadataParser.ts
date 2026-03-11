// @ts-nocheck
/**
 * Metadata Parser for Audio Files
 *
 * Parses ID3, FLAC, Vorbis, and other audio metadata tags from files.
 * Supports bulk parsing of album folders with automatic metadata extraction.
 *
 * Uses music-metadata library for comprehensive format support:
 * - MP3 (ID3v1, ID3v2.2/v2.3/v2.4)
 * - FLAC (Vorbis comments)
 * - OGG (Vorbis comments)
 * - M4A/AAC (iTunes metadata)
 * - WAV, AIFF, WMA, APE, and more
 */

import { ref } from 'vue';
import * as musicMetadata from 'music-metadata';

/**
 * Parsed track metadata from a single audio file
 */
export interface ParsedTrack {
  fileName: string;
  title: string | null;
  artist: string | null;
  album: string | null;
  trackNumber: number | null;
  discNumber: number | null;
  duration: number | null; // seconds
  year: number | null;
  genre: string | null;
  codec: string | null; // 'mp3', 'flac', 'ogg', 'aac', etc.
  bitrate: number | null; // kbps
  sampleRate: number | null; // Hz
  channels: number | null;
  lossless: boolean;
}

/**
 * Parsed album metadata aggregated from multiple tracks
 */
export interface ParsedAlbumMetadata {
  artist: string | null;
  album: string | null;
  year: number | null;
  genre: string | null;
  label: string | null;
  tracks: ParsedTrack[];
  coverArt: Blob | null;
  coverArtMimeType: string | null;
  folderName: string;
  totalDuration: number; // seconds
  codec: string | null; // Dominant codec in album
  lossless: boolean; // True if all tracks are lossless
}

/**
 * Progress callback for bulk parsing operations
 */
export interface ParseProgress {
  current: number;
  total: number;
  currentFile: string;
  percent: number;
}

/**
 * Detect codec from file extension and format info
 */
function detectCodec(fileName: string, format: musicMetadata.IFormat): string {
  const ext = fileName.split('.').pop()?.toLowerCase();

  // Use container/codec info from metadata if available
  if (format.codec) {
    const codec = format.codec.toLowerCase();
    if (codec.includes('flac')) return 'flac';
    if (codec.includes('mp3') || codec.includes('mpeg')) return 'mp3';
    if (codec.includes('vorbis')) return 'ogg';
    if (codec.includes('aac') || codec.includes('m4a')) return 'aac';
    if (codec.includes('opus')) return 'opus';
    if (codec.includes('alac')) return 'alac';
    if (codec.includes('wav') || codec.includes('pcm')) return 'wav';
  }

  // Fallback to extension
  switch (ext) {
    case 'flac': return 'flac';
    case 'mp3': return 'mp3';
    case 'ogg': case 'oga': return 'ogg';
    case 'm4a': case 'aac': return 'aac';
    case 'opus': return 'opus';
    case 'wav': return 'wav';
    case 'aiff': case 'aif': return 'aiff';
    case 'wma': return 'wma';
    case 'ape': return 'ape';
    default: return ext || 'unknown';
  }
}

/**
 * Determine if a codec is lossless
 */
function isLosslessCodec(codec: string): boolean {
  const losslessCodecs = ['flac', 'alac', 'wav', 'aiff', 'ape', 'wavpack', 'tta'];
  return losslessCodecs.includes(codec.toLowerCase());
}

/**
 * Auto-detect category from genre tag
 */
export function detectCategoryFromGenre(genre: string | null): string | null {
  if (!genre) return null;

  const g = genre.toLowerCase();

  // Audiobooks and spoken word
  if (g.includes('audiobook') || g.includes('spoken') || g.includes('speech')) {
    return 'audiobooks';
  }

  // Podcasts
  if (g.includes('podcast')) {
    return 'podcasts';
  }

  // Comedy/Standup
  if (g.includes('comedy') || g.includes('standup') || g.includes('stand-up')) {
    return 'comedy';
  }

  // Classical
  if (g.includes('classical') || g.includes('orchestra') || g.includes('symphony') || g.includes('opera')) {
    return 'classical';
  }

  // Default to music for any recognized music genre
  return 'music';
}

/**
 * Parse metadata from a single audio file
 */
export async function parseAudioFile(file: File): Promise<ParsedTrack> {
  try {
    const buffer = await file.arrayBuffer();
    const metadata = await musicMetadata.parseBuffer(
      new Uint8Array(buffer),
      { mimeType: file.type || undefined },
      { duration: true }
    );

    const { common, format } = metadata;
    const codec = detectCodec(file.name, format);

    return {
      fileName: file.name,
      title: common.title || null,
      artist: common.artist || common.albumartist || null,
      album: common.album || null,
      trackNumber: common.track?.no || null,
      discNumber: common.disk?.no || null,
      duration: format.duration || null,
      year: common.year || null,
      genre: common.genre?.[0] || null,
      codec,
      bitrate: format.bitrate ? Math.round(format.bitrate / 1000) : null,
      sampleRate: format.sampleRate || null,
      channels: format.numberOfChannels || null,
      lossless: format.lossless ?? isLosslessCodec(codec),
    };
  } catch (error) {
    console.warn(`Failed to parse metadata for ${file.name}:`, error);
    return {
      fileName: file.name,
      title: null,
      artist: null,
      album: null,
      trackNumber: null,
      discNumber: null,
      duration: null,
      year: null,
      genre: null,
      codec: detectCodec(file.name, {} as musicMetadata.IFormat),
      bitrate: null,
      sampleRate: null,
      channels: null,
      lossless: false,
    };
  }
}

/**
 * Extract cover art from audio file metadata
 */
export async function extractCoverArt(file: File): Promise<{ data: Blob; mimeType: string } | null> {
  try {
    const buffer = await file.arrayBuffer();
    const metadata = await musicMetadata.parseBuffer(
      new Uint8Array(buffer),
      { mimeType: file.type || undefined }
    );

    const picture = metadata.common.picture?.[0];
    if (picture) {
      const blob = new Blob([picture.data], { type: picture.format });
      return { data: blob, mimeType: picture.format };
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * Find the highest resolution cover art from a set of files
 */
async function findBestCoverArt(files: File[]): Promise<{ data: Blob; mimeType: string } | null> {
  let bestCover: { data: Blob; mimeType: string; size: number } | null = null;

  // Check for image files first (folder.jpg, cover.jpg, etc.)
  const imageFiles = files.filter(f =>
    /\.(jpg|jpeg|png|webp)$/i.test(f.name) &&
    /^(cover|folder|front|album|artwork)/i.test(f.name)
  );

  for (const imageFile of imageFiles) {
    const blob = imageFile.slice();
    if (!bestCover || imageFile.size > bestCover.size) {
      bestCover = {
        data: blob,
        mimeType: imageFile.type || 'image/jpeg',
        size: imageFile.size
      };
    }
  }

  // If no cover image file, extract from audio files
  if (!bestCover) {
    const audioFiles = files.filter(f => isAudioFile(f.name));
    for (const audioFile of audioFiles) {
      const cover = await extractCoverArt(audioFile);
      if (cover && (!bestCover || cover.data.size > bestCover.size)) {
        bestCover = { ...cover, size: cover.data.size };
      }
    }
  }

  return bestCover ? { data: bestCover.data, mimeType: bestCover.mimeType } : null;
}

/**
 * Check if a file/folder is macOS/system junk that should be ignored
 */
export function isJunkFile(fileName: string): boolean {
  const name = fileName.split('/').pop() || fileName;
  const junkPatterns = [
    '.DS_Store',
    '.AppleDouble',
    '.LSOverride',
    '._',  // macOS resource fork files
    'Thumbs.db',
    'desktop.ini',
    '.Spotlight-V100',
    '.Trashes',
    '__MACOSX',
    '.fseventsd',
  ];

  // Check exact matches and prefixes
  for (const pattern of junkPatterns) {
    if (name === pattern || name.startsWith(pattern)) {
      return true;
    }
  }

  return false;
}

/**
 * Filter out junk files from a file list
 */
export function filterJunkFiles(files: File[]): File[] {
  return files.filter(f => {
    const path = (f as any).webkitRelativePath || f.name;
    // Check each path segment for junk
    const segments = path.split('/');
    return !segments.some((seg: string) => isJunkFile(seg));
  });
}

/**
 * Check if a file is an audio file by extension
 */
export function isAudioFile(fileName: string): boolean {
  const audioExtensions = [
    'mp3', 'flac', 'ogg', 'oga', 'm4a', 'aac', 'wav', 'aiff', 'aif',
    'wma', 'ape', 'opus', 'wv', 'tta', 'alac'
  ];
  const ext = fileName.split('.').pop()?.toLowerCase();
  return ext ? audioExtensions.includes(ext) : false;
}

/**
 * Check if a file is a video file by extension
 */
export function isVideoFile(fileName: string): boolean {
  const videoExtensions = [
    'mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v',
    'mpg', 'mpeg', 'ts', 'mts', 'm2ts', 'vob', 'ogv', '3gp'
  ];
  const ext = fileName.split('.').pop()?.toLowerCase();
  return ext ? videoExtensions.includes(ext) : false;
}

/**
 * Check if a file is a media file (audio or video)
 */
export function isMediaFile(fileName: string): boolean {
  return isAudioFile(fileName) || isVideoFile(fileName);
}

/**
 * Count media files in a file list
 */
export function countMediaFiles(files: File[]): { audio: number; video: number; total: number } {
  let audio = 0;
  let video = 0;
  for (const file of files) {
    if (isAudioFile(file.name)) audio++;
    else if (isVideoFile(file.name)) video++;
  }
  return { audio, video, total: audio + video };
}

/**
 * Count relevant files based on category (excludes junk files)
 * - Music/audiobooks/podcasts → audio files
 * - Movies/TV → video files
 * - Unknown → all media files
 */
export function countRelevantFiles(files: File[], categoryId: string): number {
  // Filter out junk files first
  const cleanFiles = filterJunkFiles(files);
  const counts = countMediaFiles(cleanFiles);

  // Audio-based categories
  if (categoryId === 'music' || categoryId === 'audiobooks' || categoryId === 'podcasts' ||
      categoryId?.includes('music') || categoryId?.includes('audio') || categoryId?.includes('podcast')) {
    return counts.audio;
  }

  // Video-based categories
  if (categoryId === 'movies' || categoryId === 'tv-shows' || categoryId === 'tv' ||
      categoryId?.includes('movie') || categoryId?.includes('video') || categoryId?.includes('tv')) {
    return counts.video;
  }

  // Default: count all media
  return counts.total;
}

/**
 * Get most common value from an array (mode)
 */
function getMostCommon<T>(values: T[]): T | null {
  if (values.length === 0) return null;

  const counts = new Map<T, number>();
  for (const v of values) {
    counts.set(v, (counts.get(v) || 0) + 1);
  }

  let maxCount = 0;
  let result: T | null = null;
  for (const [value, count] of counts) {
    if (count > maxCount) {
      maxCount = count;
      result = value;
    }
  }
  return result;
}

/**
 * Parse an album folder and aggregate metadata from all audio files
 */
export async function parseAlbumFolder(
  files: File[],
  folderName: string,
  onProgress?: (progress: ParseProgress) => void
): Promise<ParsedAlbumMetadata> {
  const audioFiles = files.filter(f => isAudioFile(f.name));
  const tracks: ParsedTrack[] = [];

  // Parse each audio file
  for (let i = 0; i < audioFiles.length; i++) {
    const file = audioFiles[i];

    onProgress?.({
      current: i + 1,
      total: audioFiles.length,
      currentFile: file.name,
      percent: Math.round(((i + 1) / audioFiles.length) * 100),
    });

    const track = await parseAudioFile(file);
    tracks.push(track);
  }

  // Sort tracks by disc and track number
  tracks.sort((a, b) => {
    const discA = a.discNumber || 1;
    const discB = b.discNumber || 1;
    if (discA !== discB) return discA - discB;

    const trackA = a.trackNumber || 999;
    const trackB = b.trackNumber || 999;
    return trackA - trackB;
  });

  // Aggregate metadata from tracks (use most common values)
  const artists = tracks.map(t => t.artist).filter((a): a is string => a !== null);
  const albums = tracks.map(t => t.album).filter((a): a is string => a !== null);
  const years = tracks.map(t => t.year).filter((y): y is number => y !== null);
  const genres = tracks.map(t => t.genre).filter((g): g is string => g !== null);
  const codecs = tracks.map(t => t.codec).filter((c): c is string => c !== null);

  // Calculate total duration
  const totalDuration = tracks.reduce((sum, t) => sum + (t.duration || 0), 0);

  // Check if all tracks are lossless
  const lossless = tracks.length > 0 && tracks.every(t => t.lossless);

  // Extract best cover art
  const coverArt = await findBestCoverArt(files);

  return {
    artist: getMostCommon(artists),
    album: getMostCommon(albums),
    year: getMostCommon(years),
    genre: getMostCommon(genres),
    label: null, // Would need to extract from metadata.common.label if available
    tracks,
    coverArt: coverArt?.data || null,
    coverArtMimeType: coverArt?.mimeType || null,
    folderName,
    totalDuration,
    codec: getMostCommon(codecs),
    lossless,
  };
}

/**
 * Group files by their parent folder path
 */
export function groupFilesByFolder(files: FileList | File[]): Map<string, File[]> {
  const folders = new Map<string, File[]>();

  for (const file of files) {
    // Get folder path from webkitRelativePath or use root
    const relativePath = (file as any).webkitRelativePath as string || file.name;
    const parts = relativePath.split('/');

    // Get the immediate parent folder (or root if single file)
    let folderPath: string;
    if (parts.length > 1) {
      // For nested files, use the first-level folder as the album folder
      folderPath = parts[0];
    } else {
      // Single file without folder structure
      folderPath = '/';
    }

    if (!folders.has(folderPath)) {
      folders.set(folderPath, []);
    }
    folders.get(folderPath)!.push(file);
  }

  return folders;
}

/**
 * Parse multiple album folders from a file list
 */
export async function parseMultipleFolders(
  files: FileList | File[],
  onProgress?: (folderIndex: number, totalFolders: number, albumProgress: ParseProgress) => void
): Promise<ParsedAlbumMetadata[]> {
  const folders = groupFilesByFolder(files);
  const albums: ParsedAlbumMetadata[] = [];

  let folderIndex = 0;
  for (const [folderName, folderFiles] of folders) {
    const audioFiles = folderFiles.filter(f => isAudioFile(f.name));

    // Skip folders with no audio files
    if (audioFiles.length === 0) {
      folderIndex++;
      continue;
    }

    const album = await parseAlbumFolder(
      folderFiles,
      folderName,
      (progress) => onProgress?.(folderIndex, folders.size, progress)
    );

    albums.push(album);
    folderIndex++;
  }

  return albums;
}

/**
 * Vue composable for metadata parsing with reactive state
 */
export function useMetadataParser() {
  const isProcessing = ref(false);
  const progress = ref<ParseProgress | null>(null);
  const currentFolder = ref<number>(0);
  const totalFolders = ref<number>(0);

  /**
   * Parse a single audio file
   */
  async function parseFile(file: File): Promise<ParsedTrack> {
    isProcessing.value = true;
    try {
      return await parseAudioFile(file);
    } finally {
      isProcessing.value = false;
    }
  }

  /**
   * Parse an album folder
   */
  async function parseFolder(files: File[], folderName: string): Promise<ParsedAlbumMetadata> {
    isProcessing.value = true;
    progress.value = null;

    try {
      return await parseAlbumFolder(files, folderName, (p) => {
        progress.value = p;
      });
    } finally {
      isProcessing.value = false;
      progress.value = null;
    }
  }

  /**
   * Parse multiple folders (bulk upload)
   */
  async function parseFolders(files: FileList | File[]): Promise<ParsedAlbumMetadata[]> {
    isProcessing.value = true;
    progress.value = null;

    const folders = groupFilesByFolder(files);
    totalFolders.value = folders.size;
    currentFolder.value = 0;

    try {
      return await parseMultipleFolders(files, (folderIdx, total, albumProgress) => {
        currentFolder.value = folderIdx + 1;
        totalFolders.value = total;
        progress.value = albumProgress;
      });
    } finally {
      isProcessing.value = false;
      progress.value = null;
      currentFolder.value = 0;
      totalFolders.value = 0;
    }
  }

  return {
    // State
    isProcessing,
    progress,
    currentFolder,
    totalFolders,

    // Methods
    parseFile,
    parseFolder,
    parseFolders,

    // Utilities
    isAudioFile,
    detectCategoryFromGenre,
    groupFilesByFolder,
  };
}

export default useMetadataParser;
