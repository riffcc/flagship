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

// Mock electron with simplified stubs
vi.mock('electron', () => {
  // Simple stubs for ipcRenderer methods
  const mockedIpcRenderer = {
    on: vi.fn((_channel, _listener) => mockedIpcRenderer), // Return self for chaining if needed
    off: vi.fn((_channel, _listener) => mockedIpcRenderer), // Return self
    once: vi.fn((_channel, listener) => { // Basic 'once' simulation if needed
      // Immediately call listener for specific channels if required by tests
      // if (channel === 'clientPrêt') {
      //   listener({} as IpcRendererEvent);
      // }
      return mockedIpcRenderer;
    }),
    send: vi.fn((_channel, ..._args) => {}), // Simple send stub
  };

  // Basic stubs for app and ipcMain
  const mockedApp = {
    getAppPath: vi.fn(() => 'mock/app/path'),
    getPath: vi.fn(() => 'mock/path'),
  };

  const mockedIpcMain = {
    on: vi.fn(),
    handle: vi.fn(),
  };

  // Return the structure expected by imports
  return {
    ipcRenderer: mockedIpcRenderer,
    default: { // Provide the default export needed by dependencies
      app: mockedApp,
      ipcMain: mockedIpcMain,
    },
    // Also provide named exports if they are imported directly
    app: mockedApp,
    ipcMain: mockedIpcMain,
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
