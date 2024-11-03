import {writeFileSync} from 'fs';
import {constantCase} from 'change-case';

import { créerConstellation } from '@constl/ipa';
import {Orbiter} from '@riffcc/orbiter';

// Todo: refactor into a single @riffcc/orbiter-vue package to avoid duplication 
// with renderer plugin code
const getVariableIds = () => {
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
    } = process.env;
  
    const variableIds = {
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


const setUpSite = async () => {
    // Todo: refactor as part of orbiter's API so that we can do this though the command line even when not compiling a site?
    const siteId = process.env.VITE_SITE_ID;
    let swarmId = process.env.VITE_SWARM_ID;

    const  orbiter = new Orbiter({
        siteId,
        swarmId,
        variableIds: getVariableIds(),
        constellation: créerConstellation({ dossier: './.config'}),
    });
    const vars = await orbiter.setUpSite();
    ({ swarmId } = await orbiter.orbiterConfig());

    await orbiter.constellation.fermer();

    generateLocalEnvFile({siteId: vars.siteId});
    generateEnvFile({swarmId, variableIds: vars.variableIds});
};

const generateLocalEnvFile = ({siteId}) => {
  const fileContent = 
    '# The address below should be regenerated for each Orbiter site. If you are setting up an independent site, erase the value below and run the site in development mode (`pnpm dev`) to automatically regenerate. \n' +
    'VITE_SITE_ID=' + siteId +
    '\n';
  writeFileSync('.env.local', fileContent);
};

const generateEnvFile = ({swarmId, variableIds}) => {
    const variableIdsList = Object.keys(variableIds).map(
      k => `VITE_${constantCase(k)}_ID=${variableIds[k]}`,
    );
  
    const fileContent =
      '# These should ideally stay the same for all Orbiter sites for optimal performance. Only change if you know what you are doing.\n' +
      'VITE_SWARM_ID=' + swarmId +
      '\n' + 
      '\n' +
      variableIdsList.join('\n') +
      '\n'
    ;
  writeFileSync('.env', fileContent);
};

(setUpSite().then(()=>process.exit(0)));