import {createRouter, createWebHashHistory, type RouteRecordRaw} from 'vue-router';

// Keep HomePage as direct import since it's the landing page
import HomePage from '/@/views/homePage.vue';

// Lazy load all other routes
const AdminPage = () => import('../views/adminPage.vue');
const AboutPage = () => import('/@/views/aboutPage.vue');
const AccountPage = () => import('/@/views/accountPage.vue');
const PrivacyPolicyPage = () => import('/@/views/privacyPolicyPage.vue');
const ReleasePage = () => import('/@/views/releasePage.vue');
const TermsPage = () => import('/@/views/termsPage.vue');
const UploadPage = () => import('/@/views/uploadPage.vue');
const CategoryPage = () => import('../views/categoryPage.vue');

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
  {
    path: '/release/:id',
    name: 'Release',
    component: ReleasePage,
    props: true,
  },
  {
    path: '/featured/:category',
    component: CategoryPage,
    props: true,
  },
  {
    path: '/:category',
    component: CategoryPage,
    props: route => ({ ...route.params, showAll: true }),
  },
];

const routeur = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior() {
    return {top: 0};
  },
});

export default routeur;
