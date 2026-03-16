// IMPORTANT: Import WebGPU shim FIRST before any other imports
// This provides fallback constants for browsers without WebGPU support (e.g., Firefox)
// Required for 3d-force-graph and Three.js to work in browsers without WebGPU
import './lib/webgpu-shim';

import './styles/fonts.css'; // Self-hosted fonts - load early
import { createApp, type App as VueApp } from 'vue';
import App from './App.vue';
import { registerPlugins } from './plugins';

if (typeof window !== 'undefined' && 'serviceWorker' in navigator) {
  void navigator.serviceWorker.getRegistrations().then(registrations => {
    for (const registration of registrations) {
      void registration.unregister();
    }
  });
}

const app: VueApp = createApp(App);

registerPlugins(app);

app.mount('#app');
