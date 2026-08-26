/**
 * Vue Router 配置
 *
 * 路由:
 * - /  → HomeView(请求编辑器,主界面)
 *
 * Collections、History、Environments 已集成到 Sidebar/弹框,不再需要独立路由。
 */

import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/HomeView.vue'),
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
