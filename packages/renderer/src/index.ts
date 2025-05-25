import { createApp, type App as VueApp } from 'vue';
import App from './App.vue';
import { registerPlugins } from './plugins';
import 'unfonts.css';

const app: VueApp = createApp(App);

registerPlugins(app);

app.mount('#app');
