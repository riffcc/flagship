/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import colors from 'vuetify/util/colors';
import '/@/styles/main.scss';
import '/@/styles/settings.scss';
// Composables
import {createVuetify} from 'vuetify';
import {mdi} from 'vuetify/iconsets/mdi-svg';
// Import only the icons we use
import {
  mdiAccount,
  mdiAccountSupervisor,
  mdiArrowLeft,
  mdiBlockHelper,
  mdiCheck,
  mdiChevronDown,
  mdiChevronLeft,
  mdiChevronRight,
  mdiChevronUp,
  mdiCircle,
  mdiClipboardCheckMultipleOutline,
  mdiClipboardMultipleOutline,
  mdiClose,
  mdiDelete,
  mdiDotsVertical,
  mdiDownload,
  mdiFullscreen,
  mdiHeart,
  mdiHelpCircleOutline,
  mdiMenu,
  mdiMenuDown,
  mdiMenuLeft,
  mdiMenuRight,
  mdiPageFirst,
  mdiPageLast,
  mdiPause,
  mdiPauseCircle,
  mdiPencil,
  mdiPlay,
  mdiPlayCircle,
  mdiPlus,
  mdiPlusCircle,
  mdiRotateLeft,
  mdiShareVariant,
  mdiShuffle,
  mdiSkipNext,
  mdiSkipPrevious,
  mdiStar,
  mdiStarPlusOutline,
  mdiSvg,
  mdiVolumeHigh,
  mdiVolumeOff,
} from '@mdi/js';

// Create custom aliases with only the icons we use
const customAliases = {
  account: mdiAccount,
  accountSupervisor: mdiAccountSupervisor,
  arrowLeft: mdiArrowLeft,
  blockHelper: mdiBlockHelper,
  check: mdiCheck,
  chevronDown: mdiChevronDown,
  chevronLeft: mdiChevronLeft,
  chevronRight: mdiChevronRight,
  chevronUp: mdiChevronUp,
  circle: mdiCircle,
  clipboardCheckMultipleOutline: mdiClipboardCheckMultipleOutline,
  clipboardMultipleOutline: mdiClipboardMultipleOutline,
  close: mdiClose,
  delete: mdiDelete,
  dotsVertical: mdiDotsVertical,
  download: mdiDownload,
  fullscreen: mdiFullscreen,
  heart: mdiHeart,
  helpCircleOutline: mdiHelpCircleOutline,
  menu: mdiMenu,
  menuDown: mdiMenuDown,
  menuLeft: mdiMenuLeft,
  menuRight: mdiMenuRight,
  pageFirst: mdiPageFirst,
  pageLast: mdiPageLast,
  pause: mdiPause,
  pauseCircle: mdiPauseCircle,
  pencil: mdiPencil,
  play: mdiPlay,
  playCircle: mdiPlayCircle,
  plus: mdiPlus,
  plusCircle: mdiPlusCircle,
  rotateLeft: mdiRotateLeft,
  shareVariant: mdiShareVariant,
  shuffle: mdiShuffle,
  skipNext: mdiSkipNext,
  skipPrevious: mdiSkipPrevious,
  star: mdiStar,
  starPlusOutline: mdiStarPlusOutline,
  svg: mdiSvg,
  volumeHigh: mdiVolumeHigh,
  volumeOff: mdiVolumeOff,
};
// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
const vuetify = createVuetify({
  icons: {
    defaultSet: 'mdi',
    aliases: customAliases,
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
          'primary-accent': colors.purple.accent2,
        },
      },
    },
  },
});

export default vuetify;
