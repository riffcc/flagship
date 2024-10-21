/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import '@mdi/font/css/materialdesignicons.css';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import '/@/styles/main.scss';
import '/@/styles/settings.scss';
import colors from 'vuetify/util/colors';
// Composables
import {createVuetify} from 'vuetify';
import { aliases, mdi } from 'vuetify/iconsets/mdi';
// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
const vuetify = createVuetify({
  components,
  directives,
  icons: {
    defaultSet: 'mdi',
    aliases,
    sets: {
      mdi,
    },
  },
  defaults: {
    global: {
      elevation: 0,
    },
    VTextField: {
      variant: 'solo-filled',
    },
    VFileInput: {
      variant: 'solo-filled',
    },
    VSelect: {
      variant: 'solo-filled',
    },
    VAutocomplete: {
      variant: 'solo-filled',
    },
  },
  theme: {
    defaultTheme: 'dark',
    themes: {
      dark: {
        colors: {
          primary: colors.purple.base,
          'primary-lighten-1': colors.purple.lighten1,
          'primary-darken-1': colors.purple.darken1,
        },
      },
    },
  },
});

export default vuetify;
