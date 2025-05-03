export type ReleaseItem = {
  id?: string;
  name: string;
  contentCID: string;
  category: string;
  author: string;
  thumbnail?: string;
  cover?: string;
  sourceSite?: string;
  tmdbId?: string | number; // Add field for TMDB ID
  metadata: Record<string, unknown>; // Can store season/episode info here later
}

export type PartialReleaseItem = Partial<ReleaseItem>;

// Optional: Define more specific types for TV Shows later if needed
// export type Episode = { number: number; title: string; contentCID: string; /* ... */ };
// export type Season = { number: number; episodes: Episode[]; /* ... */ };
// export interface TvShowReleaseItem extends ReleaseItem {
//   category: 'tvShow';
//   seasons: Season[];
// }

export type FeaturedReleaseItem = {
  id: string;
  releaseId: string;
  startTime: string;
  endTime: string;
};
