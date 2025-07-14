import { createApp, type App as VueApp } from 'vue';
import App from './App.vue';
import { registerPlugins } from './plugins';

// WebOS debugging
if (navigator.userAgent.includes('Web0S') || navigator.userAgent.includes('webOS')) {
  console.log('[WebOS] Running on webOS TV');
  console.log('[WebOS] User Agent:', navigator.userAgent);
  console.log('[WebOS] IndexedDB available:', 'indexedDB' in window);
  console.log('[WebOS] WebRTC available:', 'RTCPeerConnection' in window);
  console.log('[WebOS] Crypto available:', 'crypto' in window);
  console.log('[WebOS] localStorage available:', 'localStorage' in window);
}

const app: VueApp = createApp(App);

registerPlugins(app);

app.mount('#app');
