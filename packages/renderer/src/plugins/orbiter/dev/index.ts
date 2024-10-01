import {
  RELEASES_AUTHOR_COLUMN,
  RELEASES_FILE_COLUMN,
  RELEASES_METADATA_COLUMN,
  RELEASES_NAME_COLUMN,
  RELEASES_THUMBNAIL_COLUMN,
  RELEASES_CATEGORY_COLUMN,
  RELEASES_STATUS_COLUMN,
} from '../consts';
import type Orbiter from '../orbiter';

const clearData = async () => {
  localStorage.clear();
  if (indexedDB.databases) {
    const indexedDbDatabases = await indexedDB.databases();
    await Promise.all(
      indexedDbDatabases.map(db => {
        if (db.name) indexedDB.deleteDatabase(db.name);
      }),
    );
  }
};

export const loadStubData = async (app: Orbiter) => {
  await clearData();

  await app.changeName({name: 'Me !', language: 'en'});

  const audioFile = (await import('../../../../../../tests/devData/06-yy_ch01_l01_d01.mp3'))
    .default;

  const audioCid = await app.constellation.ajouterÀSFIP({
    contenu: new Uint8Array(new TextEncoder().encode(audioFile)),
    nomFichier: '06-yy_ch01_l01_d01.mp3',
  });

  const thumbnailFile = (await import('../../../assets/logo.svg')).default;

  const thumbnailCid = await app.constellation.ajouterÀSFIP({
    contenu: new Uint8Array(new TextEncoder().encode(thumbnailFile)),
    nomFichier: 'logo.svg',
  });
  await app.addRelease({
    [RELEASES_NAME_COLUMN]: 'Famous song',
    [RELEASES_AUTHOR_COLUMN]: 'I. B. Astar',
    [RELEASES_THUMBNAIL_COLUMN]: thumbnailCid,
    [RELEASES_FILE_COLUMN]: audioCid,
    [RELEASES_METADATA_COLUMN]: 'With an open-access licence, of course.',
    [RELEASES_CATEGORY_COLUMN]: 'audio',
    [RELEASES_STATUS_COLUMN]: 'pending',
  });
};
