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
  mdiCircle,
  mdiCheck,
  mdiAccountSupervisor,
  mdiAccount,
  mdiStar,
  mdiStarPlusOutline,
  mdiBlockHelper,
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
  'circle': mdiCircle,
  'check': mdiCheck,
  'account-supervisor': mdiAccountSupervisor,
  'account': mdiAccount,
  'star': mdiStar,
  'star-plus-outline': mdiStarPlusOutline,
  'block-helper': mdiBlockHelper,
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
        },
      },
    },
  },
});

export default vuetify;
