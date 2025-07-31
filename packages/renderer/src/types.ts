import type {
  ImmutableProps,
  FeaturedReleaseData,
  ReleaseData,
  AnyObject,
  ContentCategoryData,
  ContentCategoryMetadataField,
} from '@riffcc/lens-sdk';

export type ReleaseItem= ImmutableProps & ReleaseData<AnyObject>;

export type FeaturedReleaseItem = ImmutableProps & FeaturedReleaseData;

export type ContentCategoryItem = ImmutableProps & ContentCategoryData<ContentCategoryMetadataField>;
