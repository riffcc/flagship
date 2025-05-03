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

// Mock only the ipcRenderer from electron
vi.mock('electron', () => {
  // Define event types for the mock emitter
  type ÉvénementsCoquille = {
    [key: string]: (...args: any[]) => void; // Generic signature
  };
  const événements = new EventEmitter() as TypedEmitter<ÉvénementsCoquille>;

  type ListenerFunc = (event: IpcRendererEvent, ...args: unknown[]) => void;
  // Store mapping from original listener to the wrapped one used with EventEmitter
  const listenerMap = new Map<ListenerFunc, ListenerFunc>();

  const mockedIpcRenderer: Pick<Electron.IpcRenderer, 'on' | 'once' | 'send' | 'off'> = {
    on(channel: string, listener: ListenerFunc) {
      // Wrap the listener to ensure consistent handling and mapping
      const wrappedListener = (event: IpcRendererEvent, ...args: unknown[]) => {
        listener(event, ...args);
      };
      listenerMap.set(listener, wrappedListener); // Store the mapping
      événements.on(channel, wrappedListener);
      return mockedIpcRenderer; // Return the mock object itself
    },
    once(channel: string, listener: ListenerFunc) {
      // Wrap, map, and ensure cleanup for 'once'
      const wrappedListener = (event: IpcRendererEvent, ...args: unknown[]) => {
        listenerMap.delete(listener); // Clean up map after execution
        listener(event, ...args);
      };
      listenerMap.set(listener, wrappedListener);
      événements.once(channel, wrappedListener);
      return mockedIpcRenderer; // Return the mock object itself
    },
    off(channel: string, listener: ListenerFunc) {
      // Retrieve the wrapped listener using the original listener as the key
      const wrappedListener = listenerMap.get(listener);
      if (wrappedListener) {
        événements.off(channel, wrappedListener);
        listenerMap.delete(listener); // Clean up map
      } else {
        // Fallback or warning if no mapping found
        // console.warn(`ipcRenderer.off: No mapped listener found for channel "${channel}"`);
        // événements.off(channel, listener); // Attempt to remove original directly (less reliable)
      }
      return mockedIpcRenderer; // Return the mock object itself
    },
    send(channel: string, ...args: unknown[]) {
      // Simulate receiving the event back via the EventEmitter
      // Emit on the channel the 'on' listener is listening to
      if (channel === 'pourIpa') {
        // Pass event object and the actual message data (args[0])
        événements.emit('dIPA', {} as IpcRendererEvent, args[0]);
      } else if (channel === 'pourServeur') {
        événements.emit('deServeur', {} as IpcRendererEvent, args[0]);
      }
      // Add other channel mappings if needed
      return mockedIpcRenderer; // Return the mock object itself
    },
  };

  // Return an object containing only the mocked ipcRenderer
  return {
    ipcRenderer: mockedIpcRenderer,
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
