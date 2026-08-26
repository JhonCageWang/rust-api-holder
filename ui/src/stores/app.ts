/**
 * 全局 app store
 *
 * 持有:
 * - 后端 ping 结果(确认连接)
 * - 应用信息
 * - 当前激活的环境
 *
 * 后续会拆分出 collection / environment / history / request 等子 store。
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'

import { invokeT } from '@/composables/useInvoke'
import type { AppInfo } from '@/types/api'

export const useAppStore = defineStore('app', () => {
  const isBackendReady = ref(false)
  const appInfo = ref<AppInfo | null>(null)
  const activeEnvironmentId = ref<string | null>(null)
  const sidebarVersion = ref(0)

  /** 启动时调用,确认后端可用 */
  async function checkBackend(): Promise<void> {
    try {
      await invokeT('ping')
      isBackendReady.value = true
      appInfo.value = await invokeT('app_info', undefined)
    } catch (e) {
      isBackendReady.value = false
      console.error('Backend check failed:', e)
    }
  }

  /** 通知 sidebar 刷新(collections/history 等数据变更后调用) */
  function bumpSidebar(): void {
    sidebarVersion.value++
  }

  return {
    isBackendReady,
    appInfo,
    activeEnvironmentId,
    sidebarVersion,
    checkBackend,
    bumpSidebar,
  }
})