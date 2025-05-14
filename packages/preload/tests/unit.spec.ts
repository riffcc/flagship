import {expect, test, vi} from 'vitest';
import type {IpcRendererEvent} from 'electron';
import EventEmitter from 'events';

// Minimal mock for ipcRenderer if other preload functions use it.
// This can be expanded if other tests require more specific IPC behavior.
vi.mock('electron', () => {
  return {
    contextBridge: {
      exposeInMainWorld: vi.fn(),
    },
    ipcRenderer: {
      on: vi.fn(),
      once: vi.fn(),
      send: vi.fn(),
      invoke: vi.fn(), // Added invoke
      // Add other methods if needed by non-constl preload functions
    },
  };
});

import {plateforme, surLinux, surMac, surWindows} from '../src'; // RE-ENABLED import

test('plateforme', async () => { // RE-ENABLED test (no longer .skip)
  // The following lines would cause errors as plateforme, etc. are undefined.
  expect(plateforme).toBe(process.platform); // RE-ENABLED
  
  const plateformes: Partial<Record<NodeJS.Platform, boolean>> = { // RE-ENABLED
    darwin: surMac, // RE-ENABLED
    linux: surLinux, // RE-ENABLED
    win32: surWindows, // RE-ENABLED
  }; // RE-ENABLED
  
  expect(plateformes[process.platform]).toBe(true); // RE-ENABLED
  
  for (const p of Object.keys(plateformes).filter(p => p !== process.platform)) { // RE-ENABLED
    expect(plateformes[p]).toBe(false); // RE-ENABLED
  } // RE-ENABLED
});
