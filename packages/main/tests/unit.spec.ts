import type {MockedClass, MockedObject} from 'vitest';
import {beforeEach, expect, test, vi} from 'vitest';
import {restoreOrCreateWindow} from '../src/mainWindow';

import {BrowserWindow} from 'electron';

// Mock import.meta.env to ensure stability for Vite-specific environment variables
vi.mock('import.meta', () => ({
  env: {
    DEV: false, // Simulates a production or test build environment
    VITE_DEV_SERVER_URL: undefined,
    // Add any other VITE_ variables used by mainWindow.ts or its dependencies here
  },
}));

// Mock for @constl/utils-ipa to provide the missing 'suivreBdDeFonction'
vi.mock('@constl/utils-ipa', () => ({
  suivreBdDeFonction: vi.fn(),
  // If other exports from this module are needed by the code under test,
  // they can be added here as well, e.g., anotherFunction: vi.fn()
}));

// Mock for Peerbit node functions
vi.mock('../src/peerbitNode', () => ({
  startPeerbitNode: vi.fn().mockResolvedValue({}), // Mocked peerbitClient, adjust if tests need specific properties
  stopPeerbitNode: vi.fn().mockResolvedValue(undefined),
  // Add mocks for other exported functions from peerbitNode.ts if they are called by the code under test
}));

/**
 * Mock real electron BrowserWindow API
 */
vi.mock('electron', () => {
  // Use "as unknown as" because vi.fn() does not have static methods
  const bw = vi.fn() as unknown as MockedClass<typeof BrowserWindow>;
  bw.getAllWindows = vi.fn(() => bw.mock.instances);
  bw.prototype.loadURL = vi.fn((_: string, __?: Electron.LoadURLOptions) => Promise.resolve());
  bw.prototype.loadFile = vi.fn((_: string, __?: Electron.LoadFileOptions) => Promise.resolve());
  // Use "any" because the on function is overloaded
  // @ts-expect-error I have no idea why
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  bw.prototype.on = vi.fn<any>();
  bw.prototype.destroy = vi.fn();
  bw.prototype.isDestroyed = vi.fn();
  bw.prototype.isMinimized = vi.fn();
  bw.prototype.focus = vi.fn();
  bw.prototype.restore = vi.fn();

  // @ts-expect-error webContents est une propriété en lecture seule
  bw.prototype.webContents = {
    send: vi.fn(),
    setWindowOpenHandler: vi.fn(),
  };

  const app: Pick<Electron.App, 'getAppPath' | 'getPath'> = {
    getAppPath(): string {
      return '';
    },
    getPath(): string {
      return '';
    },
  };

  const ipcMain: Pick<Electron.IpcMain, 'on' | 'handle'> = {
    on(..._args) {
      return this;
    },
    handle(..._args) {
      return this;
    },
  };

  return {BrowserWindow: bw, app, ipcMain};
});

beforeEach(() => {
  vi.clearAllMocks();
});

test('Devrait créer une nouvelle fenêtre', async () => {
  const {mock} = vi.mocked(BrowserWindow);
  expect(mock.instances).toHaveLength(0);

  await restoreOrCreateWindow();
  expect(mock.instances).toHaveLength(1);
  const instance = mock.instances[0] as MockedObject<BrowserWindow>;
  const loadURLCalls = instance.loadURL.mock.calls.length;
  const loadFileCalls = instance.loadFile.mock.calls.length;
  expect(loadURLCalls + loadFileCalls).toBe(1);
  if (loadURLCalls === 1) {
    expect(instance.loadURL).toHaveBeenCalledWith(expect.stringMatching(/index\.html$/));
  } else {
    expect(instance.loadFile).toHaveBeenCalledWith(expect.stringMatching(/index\.html$/));
  }
});

test('Devrait restaurer une fenêtre existante', async () => {
  const {mock} = vi.mocked(BrowserWindow);

  // Create a window and minimize it.
  await restoreOrCreateWindow();
  expect(mock.instances).toHaveLength(1);
  const appWindow = vi.mocked(mock.instances[0]);
  appWindow.isMinimized.mockReturnValueOnce(true);

  await restoreOrCreateWindow();
  expect(mock.instances).toHaveLength(1);
  expect(appWindow.restore).toHaveBeenCalledOnce();
});

test("Devrait créer une nouvelle fenêtre si l'ancienne fenêtre a été détruite", async () => {
  const {mock} = vi.mocked(BrowserWindow);

  // Create a window and destroy it.
  await restoreOrCreateWindow();
  expect(mock.instances).toHaveLength(1);
  const appWindow = vi.mocked(mock.instances[0]);
  appWindow.isDestroyed.mockReturnValueOnce(true);

  await restoreOrCreateWindow();
  expect(mock.instances).toHaveLength(2);
});
