import type {types as orbiterTypes} from '@riffcc/orbiter';
import {Orbiter, configIsComplete} from '@riffcc/orbiter';
import type {App} from 'vue';

export default {
  install: (app: App) => {
    const orbiterConfig = {
      siteId: import.meta.env.VITE_SITE_ID,
      variableIds: getVariableIds(),
    };
    let orbiterApp: Orbiter | undefined = undefined;
    if (configIsComplete(orbiterConfig)) {
      orbiterApp = new Orbiter({
        ...orbiterConfig,
      });
    } else {
      throw new Error('Orbiter config is invalid, please check the .env or generate a new one with orb export-config');
    }

    app.config.globalProperties.$orbiter = orbiterApp;
    app.provide('orbiter', orbiterApp);
  },
};

const getVariableIds = (): orbiterTypes.PossiblyIncompleteVariableIds => {
  const {
    VITE_TRUSTED_SITES_SITE_ID_VAR_ID,
    VITE_TRUSTED_SITES_NAME_VAR_ID,
    VITE_RELEASES_FILE_VAR_ID,
    VITE_RELEASES_CATEGORY_VAR_ID,
    VITE_RELEASES_AUTHOR_VAR_ID,
    VITE_RELEASES_CONTENT_NAME_VAR_ID,
    VITE_RELEASES_COVER_VAR_ID,
    VITE_RELEASES_METADATA_VAR_ID,
    VITE_RELEASES_THUMBNAIL_VAR_ID,
    VITE_COLLECTIONS_AUTHOR_VAR_ID,
    VITE_COLLECTIONS_METADATA_VAR_ID,
    VITE_COLLECTIONS_NAME_VAR_ID,
    VITE_COLLECTIONS_RELEASES_VAR_ID,
    VITE_COLLECTIONS_THUMBNAIL_VAR_ID,
    VITE_COLLECTIONS_CATEGORY_VAR_ID,
    VITE_FEATURED_RELEASES_RELEASE_ID_VAR_ID,
    VITE_FEATURED_RELEASES_START_TIME_VAR_ID,
    VITE_FEATURED_RELEASES_END_TIME_VAR_ID,
    VITE_FEATURED_RELEASES_PROMOTED_VAR_ID,
    VITE_BLOCKED_RELEASES_RELEASE_ID_VAR_ID,
    VITE_CONTENT_CATEGORIES_CATEGORY_ID_VAR_ID,
    VITE_CONTENT_CATEGORIES_DISPLAY_NAME_VAR_ID,
    VITE_CONTENT_CATEGORIES_FEATURED_VAR_ID,
    VITE_CONTENT_CATEGORIES_METADATA_SCHEMA_VAR_ID,
  } = import.meta.env;

  const variableIds: orbiterTypes.PossiblyIncompleteVariableIds = {
    trustedSitesSiteIdVar: VITE_TRUSTED_SITES_SITE_ID_VAR_ID,
    trustedSitesNameVar: VITE_TRUSTED_SITES_NAME_VAR_ID,

    releasesFileVar: VITE_RELEASES_FILE_VAR_ID,
    releasesCategoryVar: VITE_RELEASES_CATEGORY_VAR_ID,
    releasesAuthorVar: VITE_RELEASES_AUTHOR_VAR_ID,
    releasesContentNameVar: VITE_RELEASES_CONTENT_NAME_VAR_ID,
    releasesCoverVar: VITE_RELEASES_COVER_VAR_ID,
    releasesMetadataVar: VITE_RELEASES_METADATA_VAR_ID,
    releasesThumbnailVar: VITE_RELEASES_THUMBNAIL_VAR_ID,
    // releasesStatusVar: VITE_RELEASES_STATUS_VAR_ID,

    collectionsAuthorVar: VITE_COLLECTIONS_AUTHOR_VAR_ID,
    collectionsMetadataVar: VITE_COLLECTIONS_METADATA_VAR_ID,
    collectionsNameVar: VITE_COLLECTIONS_NAME_VAR_ID,
    collectionsReleasesVar: VITE_COLLECTIONS_RELEASES_VAR_ID,
    collectionsThumbnailVar: VITE_COLLECTIONS_THUMBNAIL_VAR_ID,
    collectionsCategoryVar: VITE_COLLECTIONS_CATEGORY_VAR_ID,
    // collectionsStatusVar: VITE_COLLECTIONS_STATUS_VAR_ID,

    featuredReleasesReleaseIdVar: VITE_FEATURED_RELEASES_RELEASE_ID_VAR_ID,
    featuredReleasesStartTimeVar: VITE_FEATURED_RELEASES_START_TIME_VAR_ID,
    featuredReleasesEndTimeVar: VITE_FEATURED_RELEASES_END_TIME_VAR_ID,
    featuredReleasesPromotedVar: VITE_FEATURED_RELEASES_PROMOTED_VAR_ID,

    blockedReleasesReleaseIdVar: VITE_BLOCKED_RELEASES_RELEASE_ID_VAR_ID,

    contentCategoriesCategoryIdVar: VITE_CONTENT_CATEGORIES_CATEGORY_ID_VAR_ID,
    contentCategoriesDisplayNameVar: VITE_CONTENT_CATEGORIES_DISPLAY_NAME_VAR_ID,
    contentCategoriesFeaturedVar: VITE_CONTENT_CATEGORIES_FEATURED_VAR_ID,
    contentCategoriesMetadataSchemaVar: VITE_CONTENT_CATEGORIES_METADATA_SCHEMA_VAR_ID,
  };

  return variableIds;
};
