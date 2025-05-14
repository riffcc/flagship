import {createApp} from 'vue';
import App from './App.vue';
import routeur from './plugins/router';
import vuetify from './plugins/vuetify';
import {pinia} from './plugins/pinia';
import peerbitPlugin from './plugins/peerbit';
// import { registerPlugins } from './plugins/inscription/common'; // Removed

const app = createApp(App);

// registerPlugins(app); // Removed
app.use(routeur);
app.use(vuetify);
app.use(pinia);
app.use(peerbitPlugin);

app.mount('#app');
