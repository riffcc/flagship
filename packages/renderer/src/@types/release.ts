export type ReleaseItem = {
  id?: string;
  name: string;
  contentCID: string;
  category: string;
  author: string;
  thumbnail?: string;
  cover?: string;
  sourceSite?: string;
  metadata: Record<string, unknown>;
  // Fields for TV Show Episodes (when category === 'tvShow')
  seriesId?: string;
  seasonNumber?: number;
  episodeNumber?: number;
}

export type PartialReleaseItem = Partial<ReleaseItem>;

// Represents a single episode, which is essentially a ReleaseItem with TV context
export interface Episode extends ReleaseItem {
  category: 'tvShow';
  seriesId: string;
  seasonNumber: number;
  episodeNumber: number;
}

// Represents a collection of episodes within a series
export type Season = {
  number: number;
  episodes: Episode[];
};

// Represents the overall TV Series metadata
export type TvSeries = {
  id: string; // Orbiter record ID for the series
  name: string;
  description?: string;
  thumbnail?: string; // Optional thumbnail CID for the series
  cover?: string; // Optional cover CID for the series
  // Add any other series-specific metadata fields here
  // e.g., genre, releaseYear, etc.
  sourceSite?: string; // Site where the series definition originates
};

export type PartialTvSeries = Partial<TvSeries>;

export type FeaturedReleaseItem = {
  id: string;
  releaseId: string;
  startTime: string;
  endTime: string;
};
