import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import Login from '@/views/Login.vue'
import Accounts from '@/views/accounts/Accounts.vue'
import ViewAccount from '@/views/accounts/ViewAccount.vue'
import NewAccount from '@/views/accounts/NewAccount.vue'
export {}

declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth: boolean
    requiresAccessToUsers?: boolean
    requiresAccessToSystem?: boolean
  }
}
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
      meta: {
        requiresAuth: true
      }
    },
    {
      path: '/login',
      name: 'login',
      component: Login,
      meta: {
        requiresAuth: false
      }
    },
    {
      path: '/accounts',
      name: 'accounts',
      component: Accounts,
      meta: {
        requiresAuth: true,
        requiresAccessToUsers: true
      }
    },
    {
      path: '/account/view/:id',
      name: 'view-account',
      component: ViewAccount,
      meta: {
        requiresAuth: true,
        requiresAccessToUsers: true
      }
    },
    {
      path: '/account/create',
      name: 'create-account',
      component: NewAccount,
      meta: {
        requiresAuth: true,
        requiresAccessToUsers: true
      }
    }
  ]
})

export default router
