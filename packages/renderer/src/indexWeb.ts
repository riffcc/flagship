import {createApp} from 'vue';
import App from './App.vue';
import {inscrireExtentions} from './plugins/inscription/browser.js';
import routeur from './plugins/router';

const app = createApp(App);

inscrireExtentions(app);
app.use(routeur);

app.mount('#app');
