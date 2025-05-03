import {createRouter, createWebHashHistory, type RouteRecordRaw} from 'vue-router';

import {useStaticStatus} from '../composables/staticStatus';
import AdminPage from '../views/adminPage.vue';
import AboutPage from '/@/views/aboutPage.vue';
import AccountPage from '/@/views/accountPage.vue';
import BuildingPage from '/@/views/buildingPage.vue';
import HomePage from '/@/views/homePage.vue';
import MusicPage from '/@/views/musicPage.vue'; // Import the new music page
import MoviesPage from '/@/views/moviesPage.vue'; // Import the new movies page
import TvShowsPage from '/@/views/tvShowsPage.vue'; // Import the new tv shows page
import PrivacyPolicyPage from '/@/views/privacyPolicyPage.vue';
import ReleasePage from '/@/views/releasePage.vue';
import TermsPage from '/@/views/termsPage.vue';
import UploadPage from '/@/views/uploadPage.vue';

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    component: HomePage,
  },
  {
    path: '/account',
    name: 'Account',
    component: AccountPage,
  },
  {
    path: '/upload',
    name: 'Upload',
    component: UploadPage,
  },
  {
    path: '/admin',
    name: 'Admin Website',
    component: AdminPage,
  },
  {
    path: '/about',
    component: AboutPage,
  },
  {
    path: '/privacy-policy',
    component: PrivacyPolicyPage,
  },
  {
    path: '/terms',
    component: TermsPage,
  },
  // Add routes for the new sections
  {
    path: '/music',
    name: 'Music',
    component: MusicPage, // Use MusicPage
  },
  {
    path: '/movies',
    name: 'Movies',
    component: MoviesPage, // Use MoviesPage
  },
  {
    path: '/tv-shows',
    name: 'TV Shows',
    component: TvShowsPage, // Use TvShowsPage
  },
  // End new section routes
  {
    path: '/release/:id',
    name: 'Release',
    component: ReleasePage,
    props: true,
  },
  {
    path: '/featured/:category',
    component: BuildingPage,
    props: true,

  },
];

const routeur = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior() {
    return {top: 0};
  },
});

routeur.afterEach(to => {
  const {stub} = to.query;
  const {staticStatus, alreadyChanged} = useStaticStatus();
  if (!alreadyChanged)
    staticStatus.value = stub !== undefined
      ? 'static'
      : import.meta.env.VITE_STATIC_MODE ? 'static' : 'live';
});

export default routeur;
