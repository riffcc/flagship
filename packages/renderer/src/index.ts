console.time('[Main] Total app initialization');

// Start lens service initialization ASAP
import('./preload-lens').then(() => {
  console.log('[Main] Lens service preload started');
});

// Load Vue and app in parallel with lens service
Promise.all([
  import('vue'),
  import('./App.vue'),
  import('./plugins')
]).then(([
  { createApp },
  VueApp,
  { registerPlugins }
]) => {
  console.log('[Main] Vue and app modules loaded');
  
  // Create and mount Vue app
  const app = createApp(VueApp.default);
  registerPlugins(app);
  app.mount('#app');
  
  console.timeEnd('[Main] Total app initialization');
});
