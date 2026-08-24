/**
 * 多 Tab 请求 store
 *
 * 设计目标(浏览器风格):
 * - 可以同时打开多个请求 Tab,各自独立(method/url/headers/...)
 * - 切换 Tab 不丢失编辑内容
 * - 每个 Tab 有自己的响应历史
 * - 关闭 Tab 带"未保存"提示
 *
 * 数据结构:每个 Tab 持有完整的状态(Request + Response + UI 状态)
 * 生命周期:在 store 里集中管理,UI 通过 computed 读取
 */

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { invokeT } from '@/composables/useInvoke'
import type {
  ApiRequest,
  ApiResponse,
  HttpMethod,
} from '@/types/api'

/**
  * 单个请求 Tab 的完整状态
  */
export interface RequestTab {
  /** 唯一标识 */
  id: string
  /** 显示标题(用户可改,默认 "GET /path") */
  title: string
  /** 用户自定义标题(改了就不自动生成) */
  customTitle: boolean
  /** 有未保存的修改(未来持久化用) */
  isDirty: boolean
  /** 正在发请求 */
  isLoading: boolean
  /** 当前请求内容 */
  request: ApiRequest
  /** 上次响应(可空) */
  response: ApiResponse | null
  /** 网络错误信息 */
  error: string | null
}

/** 简单 uuid(不依赖 uuid npm 包) */
function newId(): string {
  return crypto.randomUUID()
}

/** 默认空请求 */
function emptyRequest(url = 'https://httpbin.org/get'): ApiRequest {
  return {
    method: 'GET',
    url,
    headers: [],
    query: [],
    body: { type: 'none' },
    auth: { type: 'none' },
  }
}

/** 根据 request 自动生成 title */
function autoTitle(req: ApiRequest): string {
  let path = '(空)'
  try {
    const u = new URL(req.url)
    path = u.pathname + u.search
    if (path === '/' || path === '') path = u.host
  } catch {
    path = req.url || '(空)'
  }
  return `${req.method} ${path}`
}

export const useTabsStore = defineStore('tabs', () => {
  // ─── State ──────────────────────────────────────────────
  const tabs = ref<RequestTab[]>([])
  const activeId = ref<string | null>(null)

  // ─── Getters ────────────────────────────────────────────
  const activeTab = computed<RequestTab | null>(
    () => tabs.value.find((t) => t.id === activeId.value) ?? null,
  )

  const tabCount = computed(() => tabs.value.length)

  // ─── Actions ────────────────────────────────────────────

  /** 创建新 Tab,自动激活;如果没有 Tab 则第一个不能关闭 */
  function createTab(overrides: Partial<ApiRequest> = {}): RequestTab {
    const req = { ...emptyRequest(), ...overrides }
    const tab: RequestTab = {
      id: newId(),
      title: autoTitle(req),
      customTitle: false,
      isDirty: false,
      isLoading: false,
      request: req,
      response: null,
      error: null,
    }
    tabs.value.push(tab)
    activeId.value = tab.id
    return tab
  }

  /** 激活指定 Tab */
  function activate(id: string): void {
    if (tabs.value.some((t) => t.id === id)) {
      activeId.value = id
    }
  }

  /**
   * 关闭 Tab
   * - 最后一个 Tab 不被真正移除 — 而是**清空它**(像 Postman 一样)
   *   保证至少有一个 Tab 存在,避免 UI 上看到空状态
   * - 关掉中间 Tab 后,激活相邻的
   */
  function closeTab(id: string): void {
    const idx = tabs.value.findIndex((t) => t.id === id)
    if (idx === -1) return

    // 最后一个 Tab → 清空它(不创建新 Tab,不删除)
    if (tabs.value.length === 1) {
      const tab = tabs.value[0]
      tab.request = emptyRequest()
      tab.response = null
      tab.error = null
      tab.title = autoTitle(tab.request)
      tab.customTitle = false
      tab.isDirty = false
      return
    }

    const wasActive = activeId.value === id
    tabs.value.splice(idx, 1)

    if (wasActive) {
      // 激活相邻 Tab:优先选被关闭的那个位置的下一个,否则上一个
      const nextIdx = Math.min(idx, tabs.value.length - 1)
      activeId.value = tabs.value[nextIdx]?.id ?? null
    }
  }

  /** 更新当前激活 Tab 的 request */
  function updateActiveRequest(patch: Partial<ApiRequest>): void {
    const tab = activeTab.value
    if (!tab) return
    tab.request = { ...tab.request, ...patch }
    if (!tab.customTitle) {
      tab.title = autoTitle(tab.request)
    }
    tab.isDirty = true
  }

  /** 设置自定义 title */
  function setTitle(id: string, title: string): void {
    const tab = tabs.value.find((t) => t.id === id)
    if (!tab) return
    tab.title = title
    tab.customTitle = true
  }

  /** 发送当前 active Tab 的请求 */
  async function sendActive(): Promise<void> {
    const tab = activeTab.value
    if (!tab) return

    const url = tab.request.url.trim()
    if (!url) {
      tab.error = 'URL 不能为空'
      return
    }
    if (!/^https?:\/\//i.test(url)) {
      tab.error = 'URL 必须以 http:// 或 https:// 开头'
      return
    }

    tab.isLoading = true
    tab.error = null
    tab.response = null

    try {
      tab.response = await invokeT('execute_request', {
        req: tab.request,
        vars: {},
      })
    } catch (e) {
      tab.error = e instanceof Error ? e.message : String(e)
    } finally {
      tab.isLoading = false
    }
  }

  /** 初始化:确保至少有 1 个 Tab */
  function ensureNonEmpty(): void {
    if (tabs.value.length === 0) {
      createTab()
    }
  }

  return {
    // state
    tabs,
    activeId,
    // getters
    activeTab,
    tabCount,
    // actions
    createTab,
    activate,
    closeTab,
    updateActiveRequest,
    setTitle,
    sendActive,
    ensureNonEmpty,
  }
})

// Re-export 类型,方便外部 import
export type { HttpMethod }