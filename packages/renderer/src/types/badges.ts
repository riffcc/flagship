export interface AudioQuality {
  format?: 'flac' | 'mp3' | 'aac' | 'opus' | 'vorbis' | 'wav' | 'other';
  bitrate?: number;
  sampleRate?: number;
  bitDepth?: number;
  codec?: string;
}

/**
 * Quality ladder entry - maps tier name to directory CID
 * Used for switching between available audio qualities
 */
export type QualityLadder = Record<string, string>;

export interface LicenseInfo {
  type: 'cc0' | 'cc-by' | 'cc-by-sa' | 'cc-by-nd' | 'cc-by-nc' | 'cc-by-nc-sa' | 'cc-by-nc-nd';
  version?: string;
  url?: string;
  attribution?: string;
}
