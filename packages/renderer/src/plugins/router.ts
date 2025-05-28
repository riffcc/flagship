import {createRouter, createWebHashHistory, type RouteRecordRaw} from 'vue-router';

// Check if we should use federation index
const USE_FEDERATION_INDEX = true; // Enable federation index by default

// Keep HomePage as direct import since it's the landing page
import HomePageLegacy from '/@/views/homePage.vue';
import HomePageFederated from '/@/views/homePageFederated.vue';

const HomePage = USE_FEDERATION_INDEX ? HomePageFederated : HomePageLegacy;

// Lazy load all other routes
const AdminPage = () => import('../views/adminPage.vue');
const AboutPage = () => import('/@/views/aboutPage.vue');
const AccountPage = () => import('/@/views/accountPage.vue');
const PrivacyPolicyPage = () => import('/@/views/privacyPolicyPage.vue');
const TermsPage = () => import('/@/views/termsPage.vue');
const UploadPage = () => import('/@/views/uploadPage.vue');

// Release page - use federation index version if enabled
const ReleasePageLegacy = () => import('/@/views/releasePage.vue');
const ReleasePageFederated = () => import('/@/views/releasePageFederated.vue');
const ReleasePage = USE_FEDERATION_INDEX ? ReleasePageFederated : ReleasePageLegacy;

// Category pages - use federation index version if enabled
const CategoryPageLegacy = () => import('../views/categoryPage.vue');
const CategoryPageFederated = () => import('../views/categoryPageFederated.vue');
const CategoryPage = USE_FEDERATION_INDEX ? CategoryPageFederated : CategoryPageLegacy;

// Test pages
const HomePageFederatedTest = () => import('/@/views/homePageFederatedTest.vue');
const AddTestReleases = () => import('/@/views/addTestReleases.vue');
const TestFederationIndex = () => import('/@/views/testFederationIndex.vue');
const DebugReleases = () => import('/@/views/debugReleases.vue');

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    component: HomePage,
  },
  {
    path: '/federation-test',
    name: 'FederationTest',
    component: HomePageFederatedTest,
  },
  {
    path: '/add-test-releases',
    name: 'AddTestReleases',
    component: AddTestReleases,
  },
  {
    path: '/test-federation-index',
    name: 'TestFederationIndex',
    component: TestFederationIndex,
  },
  {
    path: '/debug-releases',
    name: 'DebugReleases',
    component: DebugReleases,
  },
  {
    path: '/federation-stats',
    name: 'FederationStats',
    component: () => import('/@/views/federationStats.vue'),
  },
  {
    path: '/check-export',
    name: 'CheckExport',
    component: () => import('/@/views/checkExportFile.vue'),
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
