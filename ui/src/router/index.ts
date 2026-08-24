/**
 * Vue Router 配置
 *
 * 四个一级路由:
 * - /  → HomeView(请求编辑器,主界面)
 * - /collections → CollectionsView(集合管理)
 * - /environments → EnvironmentsView
 * - /history → HistoryView
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
    path: '/collections',
    name: 'collections',
    component: () => import('@/views/CollectionsView.vue'),
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