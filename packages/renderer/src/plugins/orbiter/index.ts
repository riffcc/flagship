import type {App} from 'vue';
import {loadStubData} from './dev/index.js';
import OrbiterApp from './orbiter';
import type {possiblyIncompleteVariableIds} from './types';

export default {
  install: (app: App) => {
    const variableIds = getVariableIds();
    const constellation = app.config.globalProperties.$constl;

    const orbiterApp = new OrbiterApp({
      constellation,
      siteId: variableIds ? import.meta.env.VITE_SITE_ID : undefined,
      swarmId: variableIds ? import.meta.env.VITE_SWARM_ID : undefined,
      variableIds,
    });
    // For now, only admins can add content.
    orbiterApp.makeSitePrivate();

    app.config.globalProperties.$orbiter = orbiterApp;

    if (import.meta.env.VITE_STUB_DATA) {
      loadStubData(orbiterApp);
    }

    app.provide('orbiter', orbiterApp);
  },
};

const getVariableIds = (): possiblyIncompleteVariableIds => {
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
    VITE_RELEASES_STATUS_VAR_ID,
    VITE_COLLECTIONS_AUTHOR_VAR_ID,
    VITE_COLLECTIONS_METADATA_VAR_ID,
    VITE_COLLECTIONS_NAME_VAR_ID,
    VITE_COLLECTIONS_RELEASES_VAR_ID,
    VITE_COLLECTIONS_THUMBNAIL_VAR_ID,
    VITE_COLLECTIONS_CATEGORY_VAR_ID,
    VITE_COLLECTIONS_STATUS_VAR_ID,
    VITE_FEATURED_RELEASES_RELEASE_ID_VAR_ID,
    VITE_FEATURED_RELEASES_START_TIME_VAR_ID,
    VITE_FEATURED_RELEASES_END_TIME_VAR_ID,
    VITE_BLOCKED_RELEASES_RELEASE_ID_VAR_ID,
  } = import.meta.env;

  const variableIds: possiblyIncompleteVariableIds = {
    trustedSitesSiteIdVar: VITE_TRUSTED_SITES_SITE_ID_VAR_ID,
    trustedSitesNameVar: VITE_TRUSTED_SITES_NAME_VAR_ID,

    releasesFileVar: VITE_RELEASES_FILE_VAR_ID,
    releasesCategoryVar: VITE_RELEASES_CATEGORY_VAR_ID,
    releasesAuthorVar: VITE_RELEASES_AUTHOR_VAR_ID,
    releasesContentNameVar: VITE_RELEASES_CONTENT_NAME_VAR_ID,
    releasesCoverVar: VITE_RELEASES_COVER_VAR_ID,
    releasesMetadataVar: VITE_RELEASES_METADATA_VAR_ID,
    releasesThumbnailVar: VITE_RELEASES_THUMBNAIL_VAR_ID,
    releasesStatusVar: VITE_RELEASES_STATUS_VAR_ID,

    collectionsAuthorVar: VITE_COLLECTIONS_AUTHOR_VAR_ID,
    collectionsMetadataVar: VITE_COLLECTIONS_METADATA_VAR_ID,
    collectionsNameVar: VITE_COLLECTIONS_NAME_VAR_ID,
    collectionsReleasesVar: VITE_COLLECTIONS_RELEASES_VAR_ID,
    collectionsThumbnailVar: VITE_COLLECTIONS_THUMBNAIL_VAR_ID,
    collectionsCategoryVar: VITE_COLLECTIONS_CATEGORY_VAR_ID,
    collectionsStatusVar: VITE_COLLECTIONS_STATUS_VAR_ID,

    featuredReleasesReleaseIdVar: VITE_FEATURED_RELEASES_RELEASE_ID_VAR_ID,
    featuredReleasesStartTimeVar: VITE_FEATURED_RELEASES_START_TIME_VAR_ID,
    featuredReleasesEndTimeVar: VITE_FEATURED_RELEASES_END_TIME_VAR_ID,

    blockedReleasesReleaseIdVar: VITE_BLOCKED_RELEASES_RELEASE_ID_VAR_ID,
  };

  return variableIds;
};
