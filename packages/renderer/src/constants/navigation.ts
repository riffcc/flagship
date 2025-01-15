type NavigationItem = {
  label: string;
  path: string;
};

type NavigationMap = {
  appFooter: {
    explore: Array<NavigationItem>;
    company: Array<NavigationItem>;
    help: Array<NavigationItem>;
  };
};

export const navigationMap: NavigationMap = {
  appFooter: {
    explore: [
      {label: 'Home', path: '/'},
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
