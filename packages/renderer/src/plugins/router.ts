import {createRouter, createWebHashHistory, type RouteRecordRaw} from 'vue-router';
import AccountPage from '/@/views/accountPage.vue';
import UploadPage from '/@/views/uploadPage.vue';

import HomePage from '/@/views/homePage.vue';
import BuildingPage from '/@/views/buildingPage.vue';

import InitScreen from '/@/views/initScreen.vue';

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
    component: InitScreen,
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
];

const routeur = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default routeur;
