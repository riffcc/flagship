import {
  consts,
} from '@riffcc/orbiter';
import type {Orbiter }from '@riffcc/orbiter';

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
    [consts.RELEASES_NAME_COLUMN]: 'Famous song',
    [consts.RELEASES_AUTHOR_COLUMN]: 'I. B. Astar',
    [consts.RELEASES_THUMBNAIL_COLUMN]: thumbnailCid,
    [consts.RELEASES_FILE_COLUMN]: audioCid,
    [consts.RELEASES_METADATA_COLUMN]: 'With an open-access licence, of course.',
    [consts.RELEASES_CATEGORY_COLUMN]: 'audio',
    [consts.RELEASES_STATUS_COLUMN]: 'pending',
    [consts.RELEASES_COVER_COLUMN]: 'todo',
  });
};
