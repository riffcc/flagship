import {expect, test, vi} from 'vitest';

import type {
  CODE_CLIENT_PRÊT,
  CODE_MESSAGE_DE_SERVEUR,
  CODE_MESSAGE_D_IPA,
  CODE_MESSAGE_POUR_IPA,
  CODE_MESSAGE_POUR_SERVEUR,
  messageDeServeur,
  messageInitServeur,
} from '@constl/mandataire-electron-principal';
import type {IpcRendererEvent} from 'electron';

import {attente} from '@constl/utils-tests';
import EventEmitter from 'events';
import type TypedEmitter from 'typed-emitter';
import {v4 as uuidv4} from 'uuid';

import type {MessageDIpa, MessagePourIpa} from '@constl/mandataire';
import {
  envoyerMessageÀConstellation,
  envoyerMessageÀServeurConstellation,
  plateforme,
  surLinux,
  surMac,
  surWindows,
  écouterMessagesDeConstellation,
  écouterMessagesDeServeurConstellation,
} from '../src';


vi.mock('electron', async importOriginal => {
  const actual = await importOriginal<typeof import('electron')>();

  type ÉvénementsCoquille = {
    [CODE_MESSAGE_D_IPA]: (x: [IpcRendererEvent, MessageDIpa[]]) => void;
    [CODE_MESSAGE_DE_SERVEUR]: (x: [IpcRendererEvent, messageDeServeur[]]) => void;
    [CODE_CLIENT_PRÊT]: (e: IpcRendererEvent, args: unknown[]) => void;
  };
  const événements = new EventEmitter() as TypedEmitter<ÉvénementsCoquille>;

  const ipcRenderer: Pick<Electron.IpcRenderer, 'on' | 'once' | 'send'> = {
    on(
      channel: typeof CODE_MESSAGE_D_IPA | typeof CODE_MESSAGE_DE_SERVEUR | typeof CODE_CLIENT_PRÊT,
      listener: (event: IpcRendererEvent, ...args: unknown[]) => void,
    ) {
      if (channel === 'dIPA') {
        événements.on(channel, (x: [IpcRendererEvent, MessageDIpa[]]) => listener(...x));
      } else if (channel === 'deServeur') {
        événements.on(channel, (x: [IpcRendererEvent, messageDeServeur[]]) => listener(...x));
      }
      return this;
    },
    once(
      channel: typeof CODE_MESSAGE_D_IPA | typeof CODE_MESSAGE_DE_SERVEUR | typeof CODE_CLIENT_PRÊT,
      listener: (event: IpcRendererEvent, ...args: unknown[]) => void,
    ) {
      if (channel === 'clientPrêt') {
        listener({} as IpcRendererEvent);
      }
      événements.once(channel, listener);
      return this;
    },
    send(
      channel: typeof CODE_MESSAGE_POUR_IPA | typeof CODE_MESSAGE_POUR_SERVEUR,
      ...args: unknown[]
    ) {
      if (channel === 'pourIpa') {
        événements.emit('dIPA', [{} as IpcRendererEvent, args[0] as MessageDIpa[]]);
      } else if (channel === 'pourServeur') {
        événements.emit('deServeur', [{} as IpcRendererEvent, args[0] as messageDeServeur[]]);
      }
      return this;
    },
  };

  return {
    ...actual, // Include the original exports
    ipcRenderer, // Override with our mock ipcRenderer
  };
});

test('plateforme', async () => {
  expect(plateforme).toBe(process.platform);

  const plateformes: Partial<Record<NodeJS.Platform, boolean>> = {
    darwin: surMac,
    linux: surLinux,
    win32: surWindows,
  };

  expect(plateformes[process.platform]).toBe(true);

  for (const p of Object.keys(plateformes).filter(p => p !== process.platform)) {
    expect(plateformes[p]).toBe(false);
  }
});

test('messages ipa constellation', async () => {
  const résultat = new attente.AttendreRésultat<MessageDIpa>();

  écouterMessagesDeConstellation(message => résultat.mettreÀJour(message));

  const message: MessagePourIpa = {
    type: 'action',
    id: uuidv4(),
    fonction: ['on', 'test', 'une', 'fonction'],
    args: {qui: 'nexiste', pas: 'vraiment'},
  };
  await envoyerMessageÀConstellation(message);

  const val = await résultat.attendreExiste();
  expect(val).to.deep.equal(message);
});

test('messages serveur constellation', async () => {
  const résultat = new attente.AttendreRésultat<messageDeServeur>();

  écouterMessagesDeServeurConstellation(message => résultat.mettreÀJour(message));

  const message: messageInitServeur = {
    type: 'init',
    port: 1234,
  };
  envoyerMessageÀServeurConstellation(message);

  const val = await résultat.attendreExiste();
  expect(val).to.deep.equal(message);
});
