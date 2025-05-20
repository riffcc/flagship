import { createApp, type App as VueApp } from 'vue';
import { VueQueryPlugin } from '@tanstack/vue-query';
import App from './App.vue';
import routerPlugin from './plugins/router';
import vuetifyPlugin from './plugins/vuetify';
import lensServicePlugin from './plugins/lensService';

const app: VueApp = createApp(App);

app.use(routerPlugin);
app.use(vuetifyPlugin);
app.use(VueQueryPlugin);
app.use(lensServicePlugin);

app.mount('#app');
