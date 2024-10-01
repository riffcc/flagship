import {TypedEmitter} from 'tiny-typed-emitter';

import {Lock} from 'semaphore-async-await';

import type {Constellation, bds, tableaux, types} from '@constl/ipa';
import {ignorerNonDéfinis, suivreBdDeFonction, uneFois} from '@constl/utils-ipa';

import type {JSONSchemaType} from 'ajv';

import {
  COLLECTIONS_AUTHOR_COLUMN,
  COLLECTIONS_DB_TABLE_KEY,
  COLLECTIONS_METADATA_COLUMN,
  COLLECTIONS_NAME_COLUMN,
  COLLECTIONS_RELEASES_COLUMN,
  COLLECTIONS_THUMBNAIL_COLUMN,
  COLLECTIONS_CATEGORY_COLUMN,
  RELEASES_AUTHOR_COLUMN,
  RELEASES_DB_TABLE_KEY,
  RELEASES_FILE_COLUMN,
  RELEASES_METADATA_COLUMN,
  RELEASES_NAME_COLUMN,
  RELEASES_THUMBNAIL_COLUMN,
  RELEASES_CATEGORY_COLUMN,
  TRUSTED_SITES_NAME_COL,
  TRUSTED_SITES_SITE_ID_COL,
  TRUSTED_SITES_TABLE_KEY,
  RELEASES_STATUS_COLUMN,
  COLLECTIONS_STATUS_COLUMN,
  FEATURED_RELEASES_END_TIME,
  FEATURED_RELEASES_RELEASE_ID,
  FEATURED_RELEASES_STAR_TIME,
  FEATURED_RELEASES_TABLE_KEY,
} from './consts.js';
import type {
  Collection,
  CollectionWithId,
  Release,
  ReleaseWithId,
  TrustedSite,
  VariableIds,
  possiblyIncompleteVariableIds,
} from './types.js';
import {variableIdKeys} from './types.js';
import {removeUndefined} from './utils.js';

type forgetFunction = () => Promise<void>;

interface OrbiterEvents {
  'site configured': (args: {siteId: string; variableIds: VariableIds}) => void;
}

type RootDbSchema = {
  swarmId: string;
  modDb: string;
};
const ROOT_DB_JSON_SCHEMA: JSONSchemaType<Partial<RootDbSchema>> = {
  type: 'object',
  properties: {
    modDb: {type: 'string', nullable: true},
    swarmId: {type: 'string', nullable: true},
  },
  required: [],
};

const OrbiterSiteDbSchema: JSONSchemaType<{modDb: string; swarmId: string}> = {
  type: 'object',
  properties: {
    modDb: {
      type: 'string',
    },
    swarmId: {
      type: 'string',
    },
  },
  required: ['modDb', 'swarmId'],
};

export default class Orbiter {
  siteId?: string;
  swarmId?: string;

  initialVariableIds: possiblyIncompleteVariableIds;
  variableIds?: VariableIds;

  constellation: Constellation;
  events: TypedEmitter<OrbiterEvents>;

  statusType = ['approved', 'deleted', 'pending', 'rejected'];
  contentCategories = ['tvShow', 'movie', 'audiobook', 'game', 'book', 'music', 'video', 'other'];

  constructor({
    siteId,
    swarmId,
    variableIds,
    constellation,
  }: {
    siteId?: string;
    swarmId?: string;
    variableIds: possiblyIncompleteVariableIds;
    constellation: Constellation;
  }) {
    this.events = new TypedEmitter<OrbiterEvents>();

    this.siteId = siteId;
    this.swarmId = swarmId;
    this.initialVariableIds = variableIds;

    if (this.checkVariableIdsComplete(variableIds)) {
      this.variableIds = variableIds;
    }

    this.constellation = constellation;
  }

  // Site setup functions

  async setUpSite(): Promise<{
    siteId: string;
    variableIds: VariableIds;
  }> {
    console.log('ici');
    // Variables for moderation database
    const trustedSitesSiteIdVariableId =
      this.initialVariableIds.trustedSitesSiteIdVariableId ||
      (await this.constellation.variables.créerVariable({catégorie: 'chaîneNonTraductible'}));
    const trustedSitesNameVariableId =
      this.initialVariableIds.trustedSitesNameVariableId ||
      (await this.constellation.variables.créerVariable({catégorie: 'chaîneNonTraductible'}));
    const featuredReleasesReleaseIdVar =
    this.initialVariableIds.featuredReleasesReleaseIdVar ||
    (await this.constellation.variables.créerVariable({catégorie: 'chaîneNonTraductible'}));
    const featuredReleasesStartTimeVar =
      this.initialVariableIds.featuredReleasesStartTimeVar ||
      (await this.constellation.variables.créerVariable({catégorie: 'horoDatage'}));
    const featuredReleasesEndTimeVar =
      this.initialVariableIds.featuredReleasesEndTimeVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'horoDatage',
      }));
    console.log('ici 1');
    // Variables for releases table
    const releasesFileVar =
      this.initialVariableIds.releasesFileVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'fichier',
      }));
    const releasesThumbnailVar =
      this.initialVariableIds.releasesThumbnailVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'fichier',
      }));
    const releasesAuthorVar =
      this.initialVariableIds.releasesAuthorVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      }));
    const releasesMetadataVar =
      this.initialVariableIds.releasesMetadataVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      }));
    const releasesContentNameVar =
      this.initialVariableIds.releasesContentNameVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      }));
    console.log('ici 2');
    // The release type variable is a bit more complicated, because we need to specify
    // allowed categories to enforce.
    let releasesCategoryVar: string;
    if (this.initialVariableIds.releasesCategoryVar) {
      releasesCategoryVar = this.initialVariableIds.releasesCategoryVar;
    } else {
      releasesCategoryVar = await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      });
      // Specify allowed categories
      await this.constellation.variables.ajouterRègleVariable({
        idVariable: releasesCategoryVar,
        règle: {
          typeRègle: 'valeurCatégorique',
          détails: {
            type: 'fixe',
            options: this.contentCategories,
          },
        },
      });
    }
    let releasesStatusVar: string;
    if (this.initialVariableIds.releasesStatusVar) {
      releasesStatusVar = this.initialVariableIds.releasesStatusVar;
    } else {
      releasesStatusVar = await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      });
      // Specify allowed categories
      await this.constellation.variables.ajouterRègleVariable({
        idVariable: releasesStatusVar,
        règle: {
          typeRègle: 'valeurCatégorique',
          détails: {
            type: 'fixe',
            options: this.statusType,
          },
        },
      });
    }
    console.log('ici 3');
    // Variables for collections table
    const collectionsNameVar =
      this.initialVariableIds.collectionsNameVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      }));
    const collectionsAuthorVar =
      this.initialVariableIds.collectionsAuthorVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      }));
    const collectionsMetadataVar =
      this.initialVariableIds.collectionsMetadataVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      }));
    const collectionsReleasesVar =
      this.initialVariableIds.collectionsReleasesVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      }));
    const collectionsThumbnailVar =
      this.initialVariableIds.collectionsThumbnailVar ||
      (await this.constellation.variables.créerVariable({
        catégorie: 'fichier',
      }));
    console.log('ici 4');
    // Same thing for collections type variable.
    let collectionsCategoryVar: string;
    if (this.initialVariableIds.collectionsCategoryVar) {
      collectionsCategoryVar = this.initialVariableIds.collectionsCategoryVar;
    } else {
      collectionsCategoryVar = await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      });
      // Specify allowed categories
      await this.constellation.variables.ajouterRègleVariable({
        idVariable: collectionsCategoryVar,
        règle: {
          typeRègle: 'valeurCatégorique',
          détails: {
            type: 'fixe',
            options: this.contentCategories,
          },
        },
      });
    }
    let collectionsStatusVar: string;
    if (this.initialVariableIds.collectionsStatusVar) {
      collectionsStatusVar = this.initialVariableIds.collectionsStatusVar;
    } else {
      collectionsStatusVar = await this.constellation.variables.créerVariable({
        catégorie: 'chaîneNonTraductible',
      });
      // Specify allowed categories
      await this.constellation.variables.ajouterRègleVariable({
        idVariable: collectionsStatusVar,
        règle: {
          typeRègle: 'valeurCatégorique',
          détails: {
            type: 'fixe',
            options: this.statusType,
          },
        },
      });
    }
    console.log('ici 5');
    // Swarm ID for site
    let swarmId: string;
    if (this.swarmId) {
      swarmId = this.swarmId;
    } else {
      swarmId = await this.constellation.nuées.créerNuée({});
      // Now we can specify the format for individual release dbs and collections
      const releasesDbFormat = this.getSwarmDbSchema({
        releasesFileVar,
        releasesCategoryVar,
        releasesThumbnailVar,
        releasesAuthorVar,
        releasesContentNameVar,
        releasesMetadataVar,
        releasesStatusVar,
        collectionsAuthorVar,
        collectionsMetadataVar,
        collectionsNameVar,
        collectionsReleasesVar,
        collectionsThumbnailVar,
        collectionsCategoryVar,
        collectionsStatusVar,
        swarmId: swarmId,
      });
      for (const table of releasesDbFormat.tableaux) {
        const tableKey = table.clef;
        const idTableau = await this.constellation.nuées.ajouterTableauNuée({
          idNuée: swarmId,
          clefTableau: tableKey,
        });
        for (const col of table.cols) {
          await this.constellation.nuées.ajouterColonneTableauNuée({
            idTableau,
            idVariable: col.idVariable,
            idColonne: col.idColonne,
          });
        }
      }
    }
    console.log('ici 6');
    const modDbId = await this.constellation.bds.créerBdDeSchéma({
      schéma: {
        licence: 'ODbl-1_0',
        tableaux: [
          {
            cols: [
              {
                idVariable: trustedSitesSiteIdVariableId,
                idColonne: TRUSTED_SITES_SITE_ID_COL,
              },
              {
                idVariable: trustedSitesNameVariableId,
                idColonne: TRUSTED_SITES_NAME_COL,
              },
            ],
            clef: TRUSTED_SITES_TABLE_KEY,
          },
          {
            cols: [
              {
                idVariable: featuredReleasesReleaseIdVar,
                idColonne: FEATURED_RELEASES_RELEASE_ID,
              },
              {
                idVariable: featuredReleasesStartTimeVar,
                idColonne: FEATURED_RELEASES_STAR_TIME,
              },
              {
                idVariable: featuredReleasesEndTimeVar,
                idColonne: FEATURED_RELEASES_END_TIME,
              },
            ],
            clef: FEATURED_RELEASES_TABLE_KEY,
          },
        ],
      },
    });
    console.log('ici 7');
    const variableIds: VariableIds = {
      // Federation stuff
      trustedSitesSiteIdVariableId,
      trustedSitesNameVariableId,

      // featured relases
      featuredReleasesReleaseIdVar,
      featuredReleasesStartTimeVar,
      featuredReleasesEndTimeVar,

      // releases
      releasesFileVar,
      releasesAuthorVar,
      releasesContentNameVar,
      releasesThumbnailVar,
      releasesMetadataVar,
      releasesCategoryVar,
      releasesStatusVar,

      // collections
      collectionsAuthorVar,
      collectionsMetadataVar,
      collectionsNameVar,
      collectionsThumbnailVar,
      collectionsReleasesVar,
      collectionsCategoryVar,
      collectionsStatusVar,
    };

    const siteId = await this.constellation.créerBdIndépendante({
      type: 'keyvalue',
    });
    console.log('ici 8', {modDbId});
    await this.constellation.orbite.appliquerFonctionBdOrbite({
      idBd: siteId,
      fonction: 'put',
      args: ['modDb', modDbId],
    });
    console.log('ici 9');
    await this.constellation.orbite.appliquerFonctionBdOrbite({
      idBd: siteId,
      fonction: 'put',
      args: ['swarmId', swarmId],
    });
    console.log('ici 10');

    this.events.emit('site configured', {
      siteId,
      variableIds,
    });
    console.log('ici 11');
    return {
      siteId,
      variableIds,
    };
  }

  checkVariableIdsComplete(ids?: possiblyIncompleteVariableIds): ids is VariableIds {
    return !!ids && variableIdKeys.every(k => Object.keys(ids).includes(k) && ids[k]);
  }

  getSwarmDbSchema({
    releasesFileVar,
    releasesCategoryVar,
    releasesThumbnailVar,
    releasesAuthorVar,
    releasesContentNameVar,
    releasesMetadataVar,
    releasesStatusVar,
    collectionsNameVar,
    collectionsCategoryVar,
    collectionsReleasesVar,
    collectionsAuthorVar,
    collectionsMetadataVar,
    collectionsThumbnailVar,
    collectionsStatusVar,
    swarmId,
  }: {
    releasesFileVar: string;
    releasesCategoryVar: string;
    releasesThumbnailVar: string;
    releasesAuthorVar: string;
    releasesContentNameVar: string;
    releasesMetadataVar: string;
    releasesStatusVar: string;
    collectionsNameVar: string;
    collectionsCategoryVar: string;
    collectionsReleasesVar: string;
    collectionsAuthorVar: string;
    collectionsMetadataVar: string;
    collectionsThumbnailVar: string;
    collectionsStatusVar: string;
    swarmId: string;
  }): bds.schémaSpécificationBd {
    return {
      licence: 'ODbl-1_0',
      nuées: [swarmId],
      tableaux: [
        {
          cols: [
            {
              idVariable: releasesFileVar,
              idColonne: RELEASES_FILE_COLUMN,
            },
            {
              idVariable: releasesCategoryVar,
              idColonne: RELEASES_CATEGORY_COLUMN,
            },
            {
              idVariable: releasesThumbnailVar,
              idColonne: RELEASES_THUMBNAIL_COLUMN,
            },
            {
              idVariable: releasesAuthorVar,
              idColonne: RELEASES_AUTHOR_COLUMN,
            },
            {
              idVariable: releasesContentNameVar,
              idColonne: RELEASES_NAME_COLUMN,
            },
            {
              idVariable: releasesMetadataVar,
              idColonne: RELEASES_METADATA_COLUMN,
            },
            {
              idVariable: releasesStatusVar,
              idColonne: RELEASES_STATUS_COLUMN,
            },
          ],
          clef: RELEASES_DB_TABLE_KEY,
        },
        {
          cols: [
            {
              idVariable: collectionsNameVar,
              idColonne: COLLECTIONS_NAME_COLUMN,
            },
            {
              idVariable: collectionsCategoryVar,
              idColonne: COLLECTIONS_CATEGORY_COLUMN,
            },
            {
              idVariable: collectionsReleasesVar,
              idColonne: COLLECTIONS_RELEASES_COLUMN,
            },
            {
              idVariable: collectionsAuthorVar,
              idColonne: COLLECTIONS_AUTHOR_COLUMN,
            },
            {
              idVariable: collectionsMetadataVar,
              idColonne: COLLECTIONS_METADATA_COLUMN,
            },
            {
              idVariable: collectionsThumbnailVar,
              idColonne: COLLECTIONS_THUMBNAIL_COLUMN,
            },
            {
              idVariable: collectionsStatusVar,
              idColonne: COLLECTIONS_STATUS_COLUMN,
            },
          ],
          clef: COLLECTIONS_DB_TABLE_KEY,
        },
      ],
    };
  }

  async listenForSiteConfigured({f}: {f: (x: boolean) => void}): Promise<forgetFunction> {
    const configured = () => {
      return !!(this.siteId && this.checkVariableIdsComplete(this.variableIds));
    };
    f(configured());
    const fFinal = () => f(configured());
    this.events.on('site configured', fFinal);
    return async () => {
      this.events.off('site configured', fFinal);
    };
  }

  async siteConfigured(): Promise<{siteId: string; variableIds: VariableIds}> {
    if (this.siteId && this.checkVariableIdsComplete(this.variableIds)) {
      return {siteId: this.siteId, variableIds: this.variableIds};
    }
    return new Promise(resolve => {
      this.events.once('site configured', resolve);
    });
  }

  async orbiterConfig(): Promise<{
    modDbId: string;
    swarmId: string;
    swarmSchema: bds.schémaSpécificationBd;
  }> {
    const {siteId, variableIds} = await this.siteConfigured();

    const modDbId = (await uneFois(
      async (fSuivi: types.schémaFonctionSuivi<string | undefined>) => {
        return await this.constellation.suivreBd({
          id: siteId,
          type: 'keyvalue',
          f: async x => fSuivi(await x.get('modDb')),
          schéma: OrbiterSiteDbSchema,
        });
      },
    )) as string;

    const swarmId = (await uneFois(
      async (fSuivi: types.schémaFonctionSuivi<string | undefined>) => {
        return await this.constellation.suivreBd({
          id: siteId,
          type: 'keyvalue',
          f: async x => fSuivi(await x.get('swarmId')),
          schéma: OrbiterSiteDbSchema,
        });
      },
      x => typeof x === 'string',
    )) as string;

    const swarmSchema = this.getSwarmDbSchema({
      ...variableIds,
      swarmId: swarmId,
    });

    return {
      modDbId,
      swarmId,
      swarmSchema,
    };
  }

  async followSiteSwarmId({
    f,
    siteId,
  }: {
    f: (x: string) => void;
    siteId?: string;
  }): Promise<forgetFunction> {
    // Use this site's id if none is given
    if (!siteId) ({siteId} = await this.siteConfigured());

    return await this.constellation.suivreBdDic({
      id: siteId,
      schéma: ROOT_DB_JSON_SCHEMA,
      f: x => {
        const swarmId = x['swarmId'];
        if (typeof swarmId === 'string') f(swarmId);
      },
    });
  }

  async followSiteModDbId({
    f,
    siteId,
  }: {
    f: (x: string) => void;
    siteId?: string;
  }): Promise<forgetFunction> {
    // Use this site's id if none is given
    if (!siteId) ({siteId} = await this.siteConfigured());

    return await this.constellation.suivreBdDic({
      id: siteId,
      schéma: ROOT_DB_JSON_SCHEMA,
      f: x => {
        const swarmId = x['modDb'];
        if (typeof swarmId === 'string') f(swarmId);
      },
    });
  }

  // Accessing network data
  async followTrustedSites({
    f,
  }: {
    f: (sites?: tableaux.élémentDonnées<TrustedSite>[]) => void;
  }): Promise<forgetFunction> {
    const {siteId} = await this.siteConfigured();

    return await suivreBdDeFonction({
      fRacine: async ({
        fSuivreRacine,
      }: {
        fSuivreRacine: (nouvelIdBdCible?: string | undefined) => Promise<void>;
      }): Promise<forgetFunction> => {
        return await this.constellation.suivreBd({
          id: siteId,
          f: async x => await fSuivreRacine(await x.get('modDb')),
          type: 'keyvalue',
          schéma: OrbiterSiteDbSchema,
        });
      },
      f,
      fSuivre: async ({
        id,
        fSuivreBd,
      }: {
        id: string;
        fSuivreBd: types.schémaFonctionSuivi<tableaux.élémentDonnées<TrustedSite>[] | undefined>;
      }) => {
        return this.constellation.bds.suivreDonnéesDeTableauParClef<TrustedSite>({
          idBd: id,
          clefTableau: TRUSTED_SITES_TABLE_KEY,
          f: fSuivreBd,
        });
      },
    });
  }

  // async listenForSiteBlockedReleases({
  //   f,
  //   siteId,
  // }: {
  //   f: (releases?: {cid: string; id: string}[]) => void;
  //   siteId?: string;
  // }): Promise<forgetFunction> {
  //   return await suivreBdDeFonction({
  //     fRacine: async ({
  //       fSuivreRacine,
  //     }: {
  //       fSuivreRacine: (nouvelIdBdCible?: string) => Promise<void>;
  //     }): Promise<forgetFunction> => {
  //       return await this.followSiteModDbId({
  //         f: fSuivreRacine,
  //         siteId,
  //       });
  //     },
  //     f: ignorerNonDéfinis(f),
  //     fSuivre: async ({
  //       id,
  //       fSuivreBd,
  //     }: {
  //       id: string;
  //       fSuivreBd: types.schémaFonctionSuivi<{cid: string; id: string}[] | undefined>;
  //     }): Promise<forgetFunction> => {
  //       return await this.constellation.bds.suivreDonnéesDeTableauParClef<BlockedCid>({
  //         idBd: id,
  //         clefTableau: BLOCKED_RELEASES_TABLE_KEY,
  //         f: async blocked => {
  //           if (blocked)
  //             await fSuivreBd(
  //               blocked.map(b => {
  //                 return {
  //                   cid: b.données[BLOCKED_RELEASE_CID_COL],
  //                   id: b.id,
  //                 };
  //               }),
  //             );
  //         },
  //       });
  //     },
  //   });
  // }

  async listenForSiteReleases({
    f,
    siteId,
    desiredNResults = 1000,
  }: {
    f: types.schémaFonctionSuivi<{release: ReleaseWithId; contributor: string}[]>;
    siteId?: string;
    desiredNResults?: number;
  }): Promise<types.schémaFonctionOublier> {
    return await suivreBdDeFonction({
      fRacine: async ({
        fSuivreRacine,
      }: {
        fSuivreRacine: (nouvelIdBdCible?: string) => Promise<void>;
      }): Promise<forgetFunction> => {
        return await this.followSiteSwarmId({
          f: fSuivreRacine,
          siteId,
        });
      },
      f: ignorerNonDéfinis(f),
      fSuivre: async ({
        id,
        fSuivreBd,
      }: {
        id: string;
        fSuivreBd: types.schémaFonctionSuivi<{release: ReleaseWithId; contributor: string}[]>;
      }): Promise<forgetFunction> => {
        const {fOublier} = await this.constellation.nuées.suivreDonnéesTableauNuée<Release>({
          idNuée: id,
          clefTableau: RELEASES_DB_TABLE_KEY,
          f: releases =>
            fSuivreBd(
              releases.map(r => ({
                release: {
                  release: r.élément.données,
                  id: r.élément.id,
                },
                contributor: r.idCompte,
              })),
            ),
          nRésultatsDésirés: desiredNResults,
          clefsSelonVariables: false,
        });
        return fOublier;
      },
    });
  }

  async listenForSiteCollections({
    f,
    siteId,
    desiredNResults = 1000,
  }: {
    f: types.schémaFonctionSuivi<{collection: CollectionWithId; contributor: string}[]>;
    siteId?: string;
    desiredNResults?: number;
  }): Promise<types.schémaFonctionOublier> {
    return await suivreBdDeFonction({
      fRacine: async ({
        fSuivreRacine,
      }: {
        fSuivreRacine: (nouvelIdBdCible?: string) => Promise<void>;
      }): Promise<forgetFunction> => {
        return await this.followSiteSwarmId({
          f: fSuivreRacine,
          siteId,
        });
      },
      f: ignorerNonDéfinis(f),
      fSuivre: async ({
        id,
        fSuivreBd,
      }: {
        id: string;
        fSuivreBd: types.schémaFonctionSuivi<{collection: CollectionWithId; contributor: string}[]>;
      }): Promise<forgetFunction> => {
        const {fOublier} = await this.constellation.nuées.suivreDonnéesTableauNuée<Collection>({
          idNuée: id,
          clefTableau: COLLECTIONS_DB_TABLE_KEY,
          f: collections =>
            fSuivreBd(
              collections.map(c => ({
                collection: {
                  collection: c.élément.données,
                  id: c.élément.id,
                },
                contributor: c.idCompte,
              })),
            ),
          nRésultatsDésirés: desiredNResults,
          clefsSelonVariables: false,
        });
        return fOublier;
      },
    });
  }

  async listenForReleases({
    f,
  }: {
    f: types.schémaFonctionSuivi<{release: ReleaseWithId; contributor: string; site: string}[]>;
  }): Promise<types.schémaFonctionOublier> {
    const {siteId} = await this.siteConfigured();

    type SiteInfo = {
      blockedCids?: string[];
      entries?: {release: ReleaseWithId; contributor: string}[];
      fForget?: forgetFunction;
    };
    const siteInfos: {[site: string]: SiteInfo} = {};

    let cancelled = false;
    const lock = new Lock();

    const fFinal = async () => {
      const blockedCids = Object.values(siteInfos)
        .map(s => s.blockedCids || [])
        .flat();
      const releases = Object.entries(siteInfos)
        .map(s => (s[1].entries || []).map(r => ({...r, site: s[0]})))
        .flat()
        .filter(r => !blockedCids.includes(r.release.release.file));
      await f(releases);
    };

    const fFollowTrustedSites = async (sites?: tableaux.élémentDonnées<TrustedSite>[]) => {
      const sitesList = (sites || []).map(s => s.données);
      sitesList.push({
        [TRUSTED_SITES_SITE_ID_COL]: siteId,
        [TRUSTED_SITES_NAME_COL]: 'Me !',
      });

      await lock.acquire();
      if (cancelled) return;

      const newSites = sitesList.filter(s => !Object.keys(siteInfos).includes(s.siteName));
      const obsoleteSites = Object.keys(siteInfos).filter(
        s => !sitesList.some(x => x.siteName === s),
      );

      for (const site of newSites) {
        const fsForgetSite: types.schémaFonctionOublier[] = [];

        const {siteName} = site;
        siteInfos[siteName] = {};
        // this.listenForSiteBlockedReleases({
        //   f: async cids => {
        //     siteInfos[siteName].blockedCids = cids?.map(c => c.cid);
        //     await fFinal();
        //   },
        //   siteId: site.siteId,
        // }).then(fForget => fsForgetSite.push(fForget));

        this.listenForSiteReleases({
          f: async entries => {
            siteInfos[siteName].entries = entries;
            await fFinal();
          },
          siteId: site.siteId,
        }).then(fOublier => fsForgetSite.push(fOublier));

        siteInfos[siteName].fForget = async () => {
          await Promise.all(fsForgetSite.map(f => f()));
        };
        await fFinal();
      }
      for (const site of obsoleteSites) {
        const {fForget} = siteInfos[site];
        if (fForget) await fForget();
        delete siteInfos[site];
      }

      await fFinal();
      lock.release();
    };

    // Need to call once manually to get the user's own entries to show even if user is offline or
    // the site's master databases are unreachable.
    await fFollowTrustedSites();

    let forgetTrustedSites: types.schémaFonctionOublier;
    this.followTrustedSites({f: fFollowTrustedSites}).then(
      fForget => (forgetTrustedSites = fForget),
    );

    const fForget = async () => {
      cancelled = true;
      if (forgetTrustedSites) await forgetTrustedSites();
      await Promise.all(
        Object.values(siteInfos).map(s => (s.fForget ? s.fForget() : Promise.resolve())),
      );
    };

    return fForget;
  }

  async listenForCollections({
    f,
  }: {
    f: types.schémaFonctionSuivi<
      {collection: CollectionWithId; contributor: string; site: string}[]
    >;
  }): Promise<types.schémaFonctionOublier> {
    const {siteId} = await this.siteConfigured();

    type SiteInfo = {
      entries?: {collection: CollectionWithId; contributor: string}[];
      fForget?: forgetFunction;
    };
    const siteInfos: {[site: string]: SiteInfo} = {};

    let cancelled = false;
    const lock = new Lock();

    const fFinal = async () => {
      const collections = Object.entries(siteInfos)
        .map(s => (s[1].entries || []).map(r => ({...r, site: s[0]})))
        .flat();
      await f(collections);
    };

    const fFollowTrustedSites = async (sites?: tableaux.élémentDonnées<TrustedSite>[]) => {
      const sitesList = (sites || []).map(s => s.données);
      sitesList.push({
        [TRUSTED_SITES_SITE_ID_COL]: siteId,
        [TRUSTED_SITES_NAME_COL]: 'Me !',
      });

      await lock.acquire();
      if (cancelled) return;

      const newSites = sitesList.filter(s => !Object.keys(siteInfos).includes(s.siteName));
      const obsoleteSites = Object.keys(siteInfos).filter(
        s => !sitesList.some(x => x.siteName === s),
      );

      for (const site of newSites) {
        const fsForgetSite: types.schémaFonctionOublier[] = [];

        const {siteName} = site;
        siteInfos[siteName] = {};

        this.listenForSiteCollections({
          f: async entries => {
            siteInfos[siteName].entries = entries;
            await fFinal();
          },
          siteId: site.siteId,
        }).then(fOublier => fsForgetSite.push(fOublier));

        siteInfos[siteName].fForget = async () => {
          await Promise.all(fsForgetSite.map(f => f()));
        };
        await fFinal();
      }
      for (const site of obsoleteSites) {
        const {fForget} = siteInfos[site];
        if (fForget) await fForget();
        delete siteInfos[site];
      }

      await fFinal();
      lock.release();
    };

    // Need to call once manually to get the user's own entries to show even if user is offline or
    // the site's master databases are unreachable.
    await fFollowTrustedSites();

    let forgetTrustedSites: types.schémaFonctionOublier;
    this.followTrustedSites({f: fFollowTrustedSites}).then(
      fForget => (forgetTrustedSites = fForget),
    );

    const fForget = async () => {
      cancelled = true;
      if (forgetTrustedSites) await forgetTrustedSites();
      await Promise.all(
        Object.values(siteInfos).map(s => (s.fForget ? s.fForget() : Promise.resolve())),
      );
    };

    return fForget;
  }

  // User functionalities - adding and editing content

  async addRelease(release: Release): Promise<void> {
    const {swarmId, swarmSchema} = await this.orbiterConfig();

    await this.constellation.bds.ajouterÉlémentÀTableauUnique({
      schémaBd: swarmSchema,
      idNuéeUnique: swarmId,
      clefTableau: RELEASES_DB_TABLE_KEY,
      vals: removeUndefined(release),
    });
  }

  async removeRelease(releaseId: string) {
    const {swarmId, swarmSchema} = await this.orbiterConfig();

    await this.constellation.bds.effacerÉlémentDeTableauUnique({
      schémaBd: swarmSchema,
      idNuéeUnique: swarmId,
      clefTableau: RELEASES_DB_TABLE_KEY,
      idÉlément: releaseId,
    });
  }

  async editRelease({
    release,
    releaseId,
  }: {
    release: Partial<Release>;
    releaseId: string;
  }): Promise<void> {
    const {swarmId, swarmSchema} = await this.orbiterConfig();

    return await this.constellation.bds.modifierÉlémentDeTableauUnique({
      vals: release,
      schémaBd: swarmSchema,
      idNuéeUnique: swarmId,
      clefTableau: RELEASES_DB_TABLE_KEY,
      idÉlément: releaseId,
    });
  }

  async addCollection(collection: Collection): Promise<void> {
    const {swarmId, swarmSchema} = await this.orbiterConfig();

    await this.constellation.bds.ajouterÉlémentÀTableauUnique({
      schémaBd: swarmSchema,
      idNuéeUnique: swarmId,
      clefTableau: COLLECTIONS_DB_TABLE_KEY,
      vals: removeUndefined(collection),
    });
  }

  async removeCollection(collectionId: string) {
    const {swarmId, swarmSchema} = await this.orbiterConfig();

    await this.constellation.bds.effacerÉlémentDeTableauUnique({
      schémaBd: swarmSchema,
      idNuéeUnique: swarmId,
      clefTableau: COLLECTIONS_DB_TABLE_KEY,
      idÉlément: collectionId,
    });
  }

  async editCollection({
    collection,
    collectionId,
  }: {
    collection: Partial<Collection>;
    collectionId: string;
  }): Promise<void> {
    const {swarmId, swarmSchema} = await this.orbiterConfig();

    return await this.constellation.bds.modifierÉlémentDeTableauUnique({
      vals: collection,
      schémaBd: swarmSchema,
      idNuéeUnique: swarmId,
      clefTableau: COLLECTIONS_DB_TABLE_KEY,
      idÉlément: collectionId,
    });
  }

  async getCollectionReleasesSetId({collectionId}: {collectionId: string}): Promise<string> {
    const collections = await uneFois(
      async (
        fSuivi: types.schémaFonctionSuivi<CollectionWithId[]>,
      ): Promise<types.schémaFonctionOublier> => {
        return await this.listenForCollections({
          f: async collections => fSuivi(collections.map(c => c.collection)),
        });
      },
    );
    const collection = collections.find(c => c.id === collectionId);
    if (!collection) throw new Error('Collection not found.');
    return collection.collection[COLLECTIONS_RELEASES_COLUMN];
  }

  async addReleaseToCollection({
    releaseId,
    collectionId,
  }: {
    releaseId: string;
    collectionId: string;
  }): Promise<void> {
    const collectionReleases = await this.getCollectionReleasesSetId({collectionId});
    await this.constellation.orbite.appliquerFonctionBdOrbite({
      idBd: collectionReleases,
      fonction: 'add',
      args: [releaseId],
    });
  }

  async removeReleaseFromCollection({
    releaseId,
    collectionId,
  }: {
    releaseId: string;
    collectionId: string;
  }): Promise<void> {
    const collectionReleases = await this.getCollectionReleasesSetId({collectionId});
    await this.constellation.orbite.appliquerFonctionBdOrbite({
      idBd: collectionReleases,
      fonction: 'remove',
      args: [releaseId],
    });
  }

  // User profile functions
  async changeName({name, language}: {name?: string; language: string}): Promise<void> {
    if (name) await this.constellation.profil.sauvegarderNom({langue: language, nom: name});
    else await this.constellation.profil.effacerNom({langue: language});
  }

  async changeProfilePhoto({
    image,
  }: {
    image?: {contenu: Uint8Array; nomFichier: string};
  }): Promise<void> {
    if (image) return await this.constellation.profil.sauvegarderImage({image});
    else return await this.constellation.profil.effacerImage();
  }

  async addContactInfo({type, contact}: {type: string; contact: string}): Promise<void> {
    return await this.constellation.profil.sauvegarderContact({type, contact});
  }

  async removeContactInfo({type, contact}: {type: string; contact?: string}): Promise<void> {
    return await this.constellation.profil.effacerContact({type, contact});
  }

  async deleteAccount(): Promise<void> {
    return await this.constellation.fermerCompte();
  }

  async listenForAccountId({f}: {f: (account?: string) => void}): Promise<forgetFunction> {
    return await this.constellation.suivreIdCompte({f});
  }

  async listenForAccountExists({f}: {f: (exists: boolean) => void}): Promise<forgetFunction> {
    return await this.constellation.profil.suivreInitialisé({f});
  }

  async listenForNameChange({
    f,
    accountId,
  }: {
    f: (name: {[language: string]: string}) => void;
    accountId?: string;
  }): Promise<forgetFunction> {
    return await this.constellation.profil.suivreNoms({
      f,
      idCompte: accountId,
    });
  }

  async listenForContactInfoChange({
    f,
    accountId,
  }: {
    f: types.schémaFonctionSuivi<{type: string; contact: string}[]>;
    accountId?: string;
  }): Promise<types.schémaFonctionOublier> {
    return await this.constellation.profil.suivreContacts({
      f,
      idCompte: accountId,
    });
  }

  async listenForProfilePhotoChange({
    f,
    accountId,
  }: {
    f: types.schémaFonctionSuivi<Uint8Array | null>;
    accountId?: string;
  }): Promise<types.schémaFonctionOublier> {
    return await this.constellation.profil.suivreImage({f, idCompte: accountId});
  }

  // Site moderator functions. Moderators can moderate content and exclude/invite other useres.
  // Admins can do all that and also invite other admins and moderators.

  async listenToIsModerator({
    f,
    userId,
  }: {
    f: (isMod: 'ADMIN' | 'MODERATOR' | undefined) => void;
    userId?: string;
  }): Promise<forgetFunction> {
    // User current user if none is specified.
    userId = userId || (await this.constellation.obtIdCompte());
    const {siteId} = await this.siteConfigured();

    const resolveModType = (x?: 'MODÉRATEUR' | 'MEMBRE'): 'ADMIN' | 'MODERATOR' | undefined => {
      return x === 'MODÉRATEUR' ? 'ADMIN' : x === 'MEMBRE' ? 'MODERATOR' : undefined;
    };

    return await this.constellation.suivreAccèsBd({
      id: siteId,
      f: x => f(resolveModType(x.find(y => y.idCompte === userId)?.rôle)),
    });
  }

  async inviteModerator({userId, admin = false}: {userId: string; admin?: boolean}): Promise<void> {
    // Invitations are not revocable ! They can, however, be upgraded (moderator => admin), though not downgraded.

    const {siteId} = await this.siteConfigured();
    const {modDbId, swarmId} = await this.orbiterConfig();

    await this.constellation.nuées.inviterAuteur({
      idNuée: swarmId,
      idCompteAuteur: userId,
      rôle: admin ? 'MODÉRATEUR' : 'MEMBRE',
    });
    await this.constellation.bds.inviterAuteur({
      idBd: modDbId,
      idCompteAuteur: userId,
      rôle: admin ? 'MODÉRATEUR' : 'MEMBRE',
    });
    if (admin) {
      await this.constellation.donnerAccès({idBd: siteId, identité: userId, rôle: 'MODÉRATEUR'});
    }
  }

  // async blockRelease({cid}: {cid: string}): Promise<string> {
  //   const {modDbId} = await this.orbiterConfig();

  //   return (
  //     await this.constellation.bds.ajouterÉlémentÀTableauParClef({
  //       idBd: modDbId,
  //       clefTableau: BLOCKED_RELEASES_TABLE_KEY,
  //       vals: {[BLOCKED_RELEASE_CID_COL]: cid},
  //     })
  //   )[0];
  // }

  // async unblockRelease({id}: {id: string}): Promise<void> {
  //   const {modDbId} = await this.orbiterConfig();

  //   await this.constellation.bds.effacerÉlémentDeTableauParClef({
  //     idBd: modDbId,
  //     clefTableau: BLOCKED_RELEASES_TABLE_KEY,
  //     idÉlément: id,
  //   });
  // }

  async makeSitePrivate(): Promise<void> {
    const {swarmId} = await this.orbiterConfig();

    // Both releases and collections swarms share the same swarm and authorisation rules, so changing one will update both
    const authId = await this.constellation.nuées.obtGestionnaireAutorisationsDeNuée({
      idNuée: swarmId,
    });
    await this.constellation.nuées.changerPhisolophieAutorisation({
      idAutorisation: authId,
      philosophie: 'CJPI',
    });
  }

  async makeSitePublic(): Promise<void> {
    const {swarmId} = await this.orbiterConfig();

    // Both releases and collections swarms share the same swarm and authorisation rules, so changing one will update both
    const authId = await this.constellation.nuées.obtGestionnaireAutorisationsDeNuée({
      idNuée: swarmId,
    });
    await this.constellation.nuées.changerPhisolophieAutorisation({
      idAutorisation: authId,
      philosophie: 'IJPC',
    });
  }

  async inviteUser({userId}: {userId: string}): Promise<void> {
    const {swarmId} = await this.orbiterConfig();

    // Both releases and collections swarms share the same swarm and authorisation rules, so changing one will update both
    await this.constellation.nuées.accepterMembreNuée({
      idNuée: swarmId,
      idCompte: userId,
    });
  }

  async blockUser({userId}: {userId: string}): Promise<void> {
    const {swarmId} = await this.orbiterConfig();

    // Both releases and collections swarms share the same swarm and authorisation rules, so changing one will update both
    await this.constellation.nuées.exclureMembreDeNuée({
      idNuée: swarmId,
      idCompte: userId,
    });
  }

  async trustSite({siteId, siteName}: {siteName: string; siteId: string}): Promise<string> {
    const {modDbId} = await this.orbiterConfig();

    const elementIds = await this.constellation.bds.ajouterÉlémentÀTableauParClef<TrustedSite>({
      idBd: modDbId,
      clefTableau: TRUSTED_SITES_TABLE_KEY,
      vals: {
        [TRUSTED_SITES_SITE_ID_COL]: siteId,
        [TRUSTED_SITES_NAME_COL]: siteName,
      },
    });
    return elementIds[0];
  }

  async editTrustedSite({elementId, site}: {elementId: string; site: Partial<TrustedSite>}) {
    const {modDbId} = await this.orbiterConfig();

    await this.constellation.bds.modifierÉlémentDeTableauParClef({
      idBd: modDbId,
      clefTableau: TRUSTED_SITES_TABLE_KEY,
      idÉlément: elementId,
      vals: site,
    });
  }

  async untrustSite({elementId}: {elementId: string}) {
    const {modDbId} = await this.orbiterConfig();
    await this.constellation.bds.effacerÉlémentDeTableauParClef({
      idBd: modDbId,
      clefTableau: TRUSTED_SITES_TABLE_KEY,
      idÉlément: elementId,
    });
  }
}
