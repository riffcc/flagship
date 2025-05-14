import {createApp} from 'vue';
import App from './App.vue';
// import { registerPlugins } from './plugins/inscription/common'; // Removed

const app = createApp(App);

// registerPlugins(app); // Removed

app.mount('#app');
