/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import colors from 'vuetify/util/colors';
import '/@/styles/main.scss';
import '/@/styles/settings.scss';
import '@mdi/font/css/materialdesignicons.css';

// Composables
import {createVuetify} from 'vuetify';
// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
const vuetify = createVuetify({
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
          'primary-accent': colors.purple.accent2,
        },
      },
    },
  },
});

export default vuetify;
