console.log('[Preload Minimal] Script successfully loaded and executing.');

import {contextBridge} from 'electron';

contextBridge.exposeInMainWorld('__verySimpleAPI', {
  ping: () => 'pong',
});

console.log('[Preload Minimal] __verySimpleAPI exposed.');
