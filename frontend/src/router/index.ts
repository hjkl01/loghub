import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('../views/Login.vue'),
  },
  {
    path: '/',
    component: () => import('../views/Layout.vue'),
    redirect: '/logs',
    children: [
      {
        path: 'logs',
        name: 'LogQuery',
        component: () => import('../views/LogQuery.vue'),
      },
      {
        path: 'logs/realtime',
        name: 'LogRealtime',
        component: () => import('../views/LogRealtime.vue'),
      },
      {
        path: 'swagger',
        name: 'Swagger',
        component: () => import('../views/SwaggerView.vue'),
      },
    ],
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to, _from, next) => {
  const token = localStorage.getItem('token')
  if (to.path !== '/login' && !token) {
    next('/login')
  } else if (to.path === '/login' && token) {
    next('/')
  } else {
    next()
  }
})

export default router
