import { createApp, type App as VueApp } from 'vue';
import App from './App.vue';
import { registerPlugins } from './plugins';

const app: VueApp = createApp(App);

registerPlugins(app);

app.mount('#app');
