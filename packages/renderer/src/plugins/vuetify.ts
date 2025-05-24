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
// Map both kebab-case (used in templates) and camelCase to the same icon
// Also include mdi- prefixed versions for backward compatibility
const customAliases = {
  // Account icons
  account: mdiAccount,
  'mdi-account': mdiAccount,
  accountSupervisor: mdiAccountSupervisor,
  'account-supervisor': mdiAccountSupervisor,
  'mdi-account-supervisor': mdiAccountSupervisor,
  
  // Navigation icons
  arrowLeft: mdiArrowLeft,
  'arrow-left': mdiArrowLeft,
  'mdi-arrow-left': mdiArrowLeft,
  chevronDown: mdiChevronDown,
  'chevron-down': mdiChevronDown,
  'mdi-chevron-down': mdiChevronDown,
  chevronLeft: mdiChevronLeft,
  'chevron-left': mdiChevronLeft,
  'mdi-chevron-left': mdiChevronLeft,
  chevronRight: mdiChevronRight,
  'chevron-right': mdiChevronRight,
  'mdi-chevron-right': mdiChevronRight,
  chevronUp: mdiChevronUp,
  'chevron-up': mdiChevronUp,
  'mdi-chevron-up': mdiChevronUp,
  menu: mdiMenu,
  'mdi-menu': mdiMenu,
  menuDown: mdiMenuDown,
  'menu-down': mdiMenuDown,
  'mdi-menu-down': mdiMenuDown,
  menuLeft: mdiMenuLeft,
  'menu-left': mdiMenuLeft,
  'mdi-menu-left': mdiMenuLeft,
  menuRight: mdiMenuRight,
  'menu-right': mdiMenuRight,
  'mdi-menu-right': mdiMenuRight,
  
  // Player controls
  play: mdiPlay,
  'mdi-play': mdiPlay,
  playCircle: mdiPlayCircle,
  'play-circle': mdiPlayCircle,
  'mdi-play-circle': mdiPlayCircle,
  pause: mdiPause,
  'mdi-pause': mdiPause,
  pauseCircle: mdiPauseCircle,
  'pause-circle': mdiPauseCircle,
  'mdi-pause-circle': mdiPauseCircle,
  skipNext: mdiSkipNext,
  'skip-next': mdiSkipNext,
  'mdi-skip-next': mdiSkipNext,
  skipPrevious: mdiSkipPrevious,
  'skip-previous': mdiSkipPrevious,
  'mdi-skip-previous': mdiSkipPrevious,
  shuffle: mdiShuffle,
  'mdi-shuffle': mdiShuffle,
  volumeHigh: mdiVolumeHigh,
  'volume-high': mdiVolumeHigh,
  'mdi-volume-high': mdiVolumeHigh,
  volumeOff: mdiVolumeOff,
  'volume-off': mdiVolumeOff,
  'mdi-volume-off': mdiVolumeOff,
  
  // Actions
  check: mdiCheck,
  'mdi-check': mdiCheck,
  close: mdiClose,
  'mdi-close': mdiClose,
  delete: mdiDelete,
  'mdi-delete': mdiDelete,
  download: mdiDownload,
  'mdi-download': mdiDownload,
  fullscreen: mdiFullscreen,
  'mdi-fullscreen': mdiFullscreen,
  pencil: mdiPencil,
  'mdi-pencil': mdiPencil,
  plus: mdiPlus,
  'mdi-plus': mdiPlus,
  plusCircle: mdiPlusCircle,
  'plus-circle': mdiPlusCircle,
  'mdi-plus-circle': mdiPlusCircle,
  rotateLeft: mdiRotateLeft,
  'rotate-left': mdiRotateLeft,
  'mdi-rotate-left': mdiRotateLeft,
  shareVariant: mdiShareVariant,
  'share-variant': mdiShareVariant,
  'mdi-share-variant': mdiShareVariant,
  
  // UI elements
  blockHelper: mdiBlockHelper,
  'block-helper': mdiBlockHelper,
  'mdi-block-helper': mdiBlockHelper,
  circle: mdiCircle,
  'mdi-circle': mdiCircle,
  clipboardCheckMultipleOutline: mdiClipboardCheckMultipleOutline,
  'clipboard-check-multiple-outline': mdiClipboardCheckMultipleOutline,
  'mdi-clipboard-check-multiple-outline': mdiClipboardCheckMultipleOutline,
  clipboardMultipleOutline: mdiClipboardMultipleOutline,
  'clipboard-multiple-outline': mdiClipboardMultipleOutline,
  'mdi-clipboard-multiple-outline': mdiClipboardMultipleOutline,
  dotsVertical: mdiDotsVertical,
  'dots-vertical': mdiDotsVertical,
  'mdi-dots-vertical': mdiDotsVertical,
  heart: mdiHeart,
  'mdi-heart': mdiHeart,
  helpCircleOutline: mdiHelpCircleOutline,
  'help-circle-outline': mdiHelpCircleOutline,
  'mdi-help-circle-outline': mdiHelpCircleOutline,
  star: mdiStar,
  'mdi-star': mdiStar,
  starPlusOutline: mdiStarPlusOutline,
  'star-plus-outline': mdiStarPlusOutline,
  'mdi-star-plus-outline': mdiStarPlusOutline,
  
  // Pagination
  pageFirst: mdiPageFirst,
  'page-first': mdiPageFirst,
  'mdi-page-first': mdiPageFirst,
  pageLast: mdiPageLast,
  'page-last': mdiPageLast,
  'mdi-page-last': mdiPageLast,
  
  // Other
  svg: mdiSvg,
  'mdi-svg': mdiSvg,
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
