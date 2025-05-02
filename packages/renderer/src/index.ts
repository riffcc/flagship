import {createApp} from 'vue';
import App from './App.vue';
import { registerPlugins } from './plugins/inscription/common';

const app = createApp(App);

registerPlugins(app);

app.mount('#app');
