type NavigationItem = {
  label: string;
  path: string;
};

type NavigationMap = {
  appBar: {
    normal: Array<NavigationItem>;
    authenticated: Array<NavigationItem>;
  };
  appFooter: {
    explore: Array<NavigationItem>;
    company: Array<NavigationItem>;
    help: Array<NavigationItem>;
  };
};

export const navigationMap: NavigationMap = {
  appBar: {
    normal: [
      {label: 'Home', path: '/'},
      {label: 'Music', path: '/music'},
      {label: 'Movies', path: '/movies'},
      {label: 'TV Shows', path: '/tv-shows'},
    ],
    authenticated: [
      {label: 'Upload', path: '/upload'},
      {label: 'Account', path: '/account'},
      { label: 'Admin', path: '/admin' },
    ],
  },
  appFooter: {
    explore: [
      {label: 'Home', path: '/'},
      {label: 'Movies', path: '/movies'},
      {label: 'TV Shows', path: '/tv-shows'},
      {label: 'Music', path: '/music'},
      {label: 'AudioBooks', path: '/audiobooks'},
    ],
    company: [
      {label: 'Terms of Use', path: '/terms'},
      {label: 'About the Riff.CC Project', path: '/about'},
    ],
    help: [
      {label: 'Privacy Policy', path: '/privacy-policy'},
      {label: 'Contact Us', path: '/contact'},
    ],
  },
};
