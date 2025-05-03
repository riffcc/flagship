import {createRouter, createWebHashHistory, type RouteRecordRaw} from 'vue-router';

import {useStaticStatus} from '../composables/staticStatus';
import {createRouter, createWebHashHistory, type RouteRecordRaw} from 'vue-router';

import {useStaticStatus} from '../composables/staticStatus';
import AdminPage from '../views/adminPage.vue';
import AboutPage from '/@/views/aboutPage.vue';
import AccountPage from '/@/views/accountPage.vue';
import BuildingPage from '/@/views/buildingPage.vue'; // Placeholder/generic page
import HomePage from '/@/views/homePage.vue';
import MusicPage from '/@/views/musicPage.vue';
import MoviesPage from '/@/views/moviesPage.vue'; // Import the actual movies page
import TvShowsPage from '/@/views/tvShowsPage.vue'; // Import the actual tv shows page
import PrivacyPolicyPage from '/@/views/privacyPolicyPage.vue';
import ReleasePage from '/@/views/releasePage.vue'; // Generic release detail page
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
    component: MusicPage,
  },
  {
    path: '/movies',
    name: 'Movies',
    component: MoviesPage, // Use the new MoviesPage
  },
  {
    path: '/tv-shows',
    name: 'TV Shows',
    component: TvShowsPage, // Use the new TvShowsPage
  },
  // End new section routes
  { // Route for generic release detail (used by movies, music, etc.)
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
 { // Route for TV Show specific detail page
   path: '/tv-show/:id',
   name: 'TV Show Detail',
   component: () => import('/@/views/tvShowDetailPage.vue'), // Lazy load the new component
   props: true,
 },
 { // Keep featured route if needed
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
