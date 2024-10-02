import {createRouter, createWebHashHistory, type RouteRecordRaw} from 'vue-router';

import AboutPage from '/@/views/aboutPage.vue';
import AccountPage from '/@/views/accountPage.vue';
import BuildingPage from '/@/views/buildingPage.vue';
import HomePage from '/@/views/homePage.vue';
import InitPage from '/@/views/initPage.vue';
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
    path: '/init',
    name: 'Create Website',
    component: InitPage,
  },
  {
    path: '/admin',
    name: 'Admin Website',
    component: BuildingPage,
  },
  {
    path: '/music',
    component: BuildingPage,
  },
  {
    path: '/movies',
    component: BuildingPage,
  },
  {
    path: '/tv-shows',
    component: BuildingPage,
  },
  {
    path: '/about',
    component: AboutPage,
  },
];

const routeur = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default routeur;
