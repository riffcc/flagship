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
      {label: 'Terms of Use', path: 'https://riff.cc/terms'},
      {label: 'About the Riff.CC Project', path: 'https://riff.cc/about'},
    ],
    help: [
      {label: 'Privacy Policy', path: 'https://riff.cc/privacy'},
      {label: 'Contact Us', path: 'https://riff.cc/contact'},
    ],
  },
};
