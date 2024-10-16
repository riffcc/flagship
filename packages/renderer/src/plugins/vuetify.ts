/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import '@fortawesome/fontawesome-free/css/all.css';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import '/@/styles/main.scss';
import '/@/styles/settings.scss';

// Composables
import {createVuetify} from 'vuetify';

import {aliases, fa} from 'vuetify/iconsets/fa';

// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
const vuetify = createVuetify({
  components,
  directives,
  icons: {
    defaultSet: 'fa',
    aliases,
    sets: {
      fa,
    },
  },
  defaults: {
    global: {
      elevation: 0,
    },
    VTextField: {
      variant: 'solo-filled',
      'bg-color': 'background',
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
    VTooltip: {
      location: 'start',
    },
  },
  theme: {
    defaultTheme: 'default',
    themes: {
      default: {
        dark: false,
        colors: {
          background: '#221F1F',
          'background-lighten-1': '#141414',
          'background-lighten-2': '#363B65',
          'background-darken-1': '#191919',
          'background-darken-2': '#191919',
          primary: '#A020F0',
          'primary-lighten-1': '#BA52FB',
          'primary-darken-1': '#7918B5',
          secondary: '#D027C1',
          'secondary-darken-1': '#2B2029',
          'secondary-lighten-1': '#AD18A0',
          surface: '#191919',
          error: '#F44336',
          info: '#2986CC',
          success: '#51BF32',
          warning: '#F1C232',
        },
        variables: {
          'border-color': '#A020F0',
          'border-opacity': '0.5',
          'high-emphasis-opacity': '0.87',
          'medium-emphasis-opacity': ' 0.6',
          'disabled-opacity': '0.38',
          'idle-opacity': '0.04',
          'hover-opacity': '0.04',
          'focus-opacity': '0.12',
          'selected-opacity': '0.08',
          'activated-opacity': '0.12',
          'pressed-opacity': '0.12',
          'dragged-opacity': '0.08',
          'theme-kbd': '#212529',
          'theme-on-kbd': '#FFFFFF',
          'theme-code': '#F5F5F5',
          'theme-on-code': '#000000',
        },
      },
    },
  },
});

export default vuetify;
