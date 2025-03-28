export type ReleaseItem = {
  id?: string;
  name: string;
  contentCID: string;
  category: string;
  author: string;
  thumbnail?: string;
  cover?: string;
  sourceSite?: string;
  metadata: Record<string, unknown>
}

export type PartialReleaseItem = Partial<ReleaseItem>;

export type FeaturedReleaseItem = {
  id: string;
  releaseId: string;
  startTime: string;
  endTime: string;
};
