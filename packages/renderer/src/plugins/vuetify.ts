/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import colors from 'vuetify/util/colors';
import '/@/styles/main.scss';
import '/@/styles/settings.scss';
import { createVuetify } from 'vuetify';
import { mdi } from 'vuetify/iconsets/mdi-svg';
import {
  mdiClipboardMultipleOutline,
  mdiClipboardCheckMultipleOutline,
  mdiPlay,
  mdiShareVariant,
  mdiHeart,
  mdiPlus,
  mdiMenuLeft,
  mdiMenuRight,
  mdiClose,
  mdiSkipPrevious,
  mdiPauseCircle,
  mdiPlayCircle,
  mdiSkipNext,
  mdiDotsVertical,
  mdiVolumeOff,
  mdiVolumeHigh,
  mdiRotateLeft,
  mdiShuffle,
  mdiPlusCircle,
  mdiHelpCircleOutline,
  mdiPencil,
  mdiDelete,
  mdiDownload,
  mdiArrowLeft,
  mdiPause,
  mdiFullscreen,
  mdiMenu,
  mdiChevronUp,
  mdiChevronLeft,
  mdiChevronRight,
  mdiCircle,
  mdiCheck,
  mdiAccountSupervisor,
  mdiAccount,
  mdiStar,
  mdiStarPlusOutline,
  mdiBlockHelper,
  mdiGamepad,
  mdiCursorDefaultOutline,
  mdiTelevision,
  mdiMusic,
  mdiTag,
  mdiFolder,
  mdiEye,
  mdiCursorDefaultClick,
  mdiCloseCircle,
  mdiMagnify,
  mdiMenuDown,
  mdiDragVertical,
  mdiCloudCheck,
  mdiPageFirst,
  mdiPageLast,
  mdiWeatherSunny,
  mdiWeatherNight,
} from '@mdi/js';

const iconsAliasesMapping = {
  'clipboard-multiple-outline': mdiClipboardMultipleOutline,
  'clipboard-check-multiple-outline': mdiClipboardCheckMultipleOutline,
  'play': mdiPlay,
  'share-variant': mdiShareVariant,
  'heart': mdiHeart,
  'plus': mdiPlus,
  'menu-left': mdiMenuLeft,
  'menu-right': mdiMenuRight,
  'close': mdiClose,
  'skip-previous': mdiSkipPrevious,
  'pause-circle': mdiPauseCircle,
  'play-circle': mdiPlayCircle,
  'skip-next': mdiSkipNext,
  'dots-vertical': mdiDotsVertical,
  'volume-off': mdiVolumeOff,
  'volume-high': mdiVolumeHigh,
  'rotate-left': mdiRotateLeft,
  'shuffle': mdiShuffle,
  'plus-circle': mdiPlusCircle,
  'help-circle-outline': mdiHelpCircleOutline,
  'pencil': mdiPencil,
  'delete': mdiDelete,
  'download': mdiDownload,
  'arrow-left': mdiArrowLeft,
  'pause': mdiPause,
  'fullscreen': mdiFullscreen,
  'menu': mdiMenu,
  'chevron-up': mdiChevronUp,
  prev: mdiChevronLeft,
  next: mdiChevronRight,
  'circle': mdiCircle,
  'check': mdiCheck,
  'account-supervisor': mdiAccountSupervisor,
  'account': mdiAccount,
  'star': mdiStar,
  'star-plus-outline': mdiStarPlusOutline,
  'block-helper': mdiBlockHelper,
  'gamepad': mdiGamepad,
  'cursor-default-outline': mdiCursorDefaultOutline,
  'television': mdiTelevision,
  'music': mdiMusic,
  'tag': mdiTag,
  'folder': mdiFolder,
  'eye': mdiEye,
  'cursor-default-click': mdiCursorDefaultClick,
  'close-circle': mdiCloseCircle,
  'magnify': mdiMagnify,
  'menu-down': mdiMenuDown,
  'drag-vertical': mdiDragVertical,
  'cloud-check': mdiCloudCheck,
  'page-first': mdiPageFirst,
  'page-last': mdiPageLast,
  'weather-sunny': mdiWeatherSunny,
  'weather-night': mdiWeatherNight,
};

const vuetify = createVuetify({
  icons: {
    defaultSet: 'mdi',
    aliases: iconsAliasesMapping,
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
          background: '#000000',
          surface: '#000000',
          'surface-variant': '#0a0a0a',
          // Network visualization colors
          'server-node': colors.purple.base,
          'browser-node': '#546e7a',  // Light grey-blue for dark mode
        },
      },
      light: {
        colors: {
          primary: colors.purple.base,
          'primary-lighten-1': colors.purple.lighten1,
          'primary-darken-1': colors.purple.darken1,
          'primary-accent': colors.purple.accent2,
          background: '#ffffff',
          surface: '#ffffff',
          'surface-variant': '#f5f5f5',
          // Network visualization colors
          'server-node': colors.purple.base,
          'browser-node': '#80deea',  // Light cyan for light mode
        },
      },
    },
  },
});

export default vuetify;
