/**
 * Vue Router 配置
 *
 * 三个一级路由:
 * - /  → HomeView(请求编辑器,主界面)
 * - /environments → EnvironmentsView
 * - /history → HistoryView
 *
 * 后续会加入嵌套路由(比如 HomeView 下有 collection 子路由)。
 */

import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/HomeView.vue'),
  },
  {
    path: '/environments',
    name: 'environments',
    component: () => import('@/views/EnvironmentsView.vue'),
  },
  {
    path: '/history',
    name: 'history',
    component: () => import('@/views/HistoryView.vue'),
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router