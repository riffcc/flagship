export interface AudioQuality {
  format?: 'flac' | 'mp3' | 'aac' | 'opus' | 'other';
  bitrate?: number;
  sampleRate?: number;
  bitDepth?: number;
  codec?: string;
}

export interface LicenseInfo {
  type: 'cc0' | 'cc-by' | 'cc-by-sa' | 'cc-by-nd' | 'cc-by-nc' | 'cc-by-nc-sa' | 'cc-by-nc-nd';
  version?: string;
  url?: string;
  attribution?: string;
}
