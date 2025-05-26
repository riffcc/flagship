import type { FeaturedReleaseData, IdData, ReleaseData, AnyObject } from '@riffcc/lens-sdk';

export type { AnyObject };

export type ReleaseItem<T = string> = IdData & ReleaseData<T>;

export type PartialReleaseItem<T = string> = Partial<ReleaseItem<T>>;

export type FeaturedReleaseItem = IdData & FeaturedReleaseData;

export type PartialFeaturedReleaseItem = Partial<FeaturedReleaseItem>;
