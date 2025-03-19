import type { types as orbiterTypes } from '@riffcc/orbiter';

export interface ReleaseItem {
  id?: string;
  name: string;
  contentCID: string;
  category: string;
  author: string;
  thumbnail?: string;
  cover?: string;
  sourceSite?: string;
  metadata: orbiterTypes.ReleaseMetadata | orbiterTypes.MusicReleaseMetadata | orbiterTypes.MovieReleaseMetadata;
}

export type PartialReleaseItem = Partial<ReleaseItem>;
