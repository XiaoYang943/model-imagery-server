import routes from '~pages';
import { createRouter, createWebHashHistory } from 'vue-router';
export const router = createRouter({
  routes: [
    { path: '/', redirect: { name: "main" } },
    ...routes,
  ],
  history: createWebHashHistory(),
});
