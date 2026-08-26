/**
 * 多 Tab 请求 store
 *
 * 设计目标(浏览器风格):
 * - 可以同时打开多个请求 Tab,各自独立(method/url/headers/...)
 * - 切换 Tab 不丢失编辑内容
 * - 每个 Tab 有自己的响应历史
 * - 关闭 Tab 带"未保存"提示
 * - 支持从 DB 加载已保存请求 / 从历史快照加载
 * - 保存(create/update)回写 DB
 *
 * 数据结构:每个 Tab 持有完整的状态(Request + Response + UI 状态)
 * 生命周期:在 store 里集中管理,UI 通过 computed 读取
 */

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { invokeT } from '@/composables/useInvoke'
import { useAppStore } from '@/stores/app'
import type {
  ApiRequest,
  ApiResponse,
  HistoryEntry,
  HttpMethod,
  RequestItem,
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
  /** 有未保存的修改 */
  isDirty: boolean
  /** 正在发请求 */
  isLoading: boolean
  /** 当前请求内容 */
  request: ApiRequest
  /** 上次响应(可空) */
  response: ApiResponse | null
  /** 网络错误信息 */
  error: string | null
  /** DB 中的 request ID,null = 未保存 */
  requestId: string | null
  /** 保存到 DB 时的请求名称 */
  requestName: string
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
  const appStore = useAppStore()

  // ─── State ──────────────────────────────────────────────
  const tabs = ref<RequestTab[]>([])
  const activeId = ref<string | null>(null)

  // ─── Getters ────────────────────────────────────────────
  const activeTab = computed<RequestTab | null>(
    () => tabs.value.find((t) => t.id === activeId.value) ?? null,
  )

  const tabCount = computed(() => tabs.value.length)

  // ─── Actions ────────────────────────────────────────────

  /** 创建新 Tab,自动激活 */
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
      requestId: null,
      requestName: '',
    }
    tabs.value.push(tab)
    activeId.value = tab.id
    return tab
  }

  /** 从已保存的 RequestItem 加载到新 Tab */
  function loadRequest(item: RequestItem): RequestTab {
    const req: ApiRequest = {
      method: item.method,
      url: item.url,
      headers: item.headers,
      query: item.query,
      body: item.body,
      auth: item.auth,
    }
    const tab: RequestTab = {
      id: newId(),
      title: item.name,
      customTitle: true,
      isDirty: false,
      isLoading: false,
      request: req,
      response: null,
      error: null,
      requestId: item.id,
      requestName: item.name,
    }
    tabs.value.push(tab)
    activeId.value = tab.id
    return tab
  }

  /** 从历史快照加载到新 Tab */
  function loadHistory(entry: HistoryEntry): RequestTab {
    const req = entry.request_snapshot
    const tab: RequestTab = {
      id: newId(),
      title: autoTitle(req),
      customTitle: false,
      isDirty: false,
      isLoading: false,
      request: { ...req },
      response: entry.response,
      error: entry.error,
      requestId: entry.request_id,
      requestName: '',
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

    if (tabs.value.length === 1) {
      const tab = tabs.value[0]
      tab.request = emptyRequest()
      tab.response = null
      tab.error = null
      tab.title = autoTitle(tab.request)
      tab.customTitle = false
      tab.isDirty = false
      tab.requestId = null
      tab.requestName = ''
      return
    }

    const wasActive = activeId.value === id
    tabs.value.splice(idx, 1)

    if (wasActive) {
      const nextIdx = Math.min(idx, tabs.value.length - 1)
      activeId.value = tabs.value[nextIdx]?.id ?? null
    }
  }

  /** 关闭除指定 Tab 外的所有 Tab */
  function closeOthers(keepId: string): void {
    tabs.value = tabs.value.filter((t) => t.id === keepId)
    activeId.value = keepId
  }

  /** 关闭所有 Tab,保留一个空白 Tab */
  function closeAllTabs(): void {
    tabs.value = []
    ensureNonEmpty()
  }

  /** 关闭指定 Tab 左侧的所有 Tab */
  function closeLeft(targetId: string): void {
    const idx = tabs.value.findIndex((t) => t.id === targetId)
    if (idx <= 0) return
    const removed = tabs.value.slice(0, idx)
    tabs.value = tabs.value.slice(idx)
    if (removed.some((t) => t.id === activeId.value)) {
      activeId.value = targetId
    }
  }

  /** 关闭指定 Tab 右侧的所有 Tab */
  function closeRight(targetId: string): void {
    const idx = tabs.value.findIndex((t) => t.id === targetId)
    if (idx === -1 || idx === tabs.value.length - 1) return
    const removed = tabs.value.slice(idx + 1)
    tabs.value = tabs.value.slice(0, idx + 1)
    if (removed.some((t) => t.id === activeId.value)) {
      activeId.value = targetId
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

  /** 加载激活环境的变量(用于 {{var}} 插值) */
  async function loadEnvVars(): Promise<Record<string, string>> {
    try {
      const activeEnv = await invokeT('get_active_environment', undefined)
      if (!activeEnv) return {}
      const vars = await invokeT('list_variables', {
        environmentId: activeEnv.id,
      })
      const map: Record<string, string> = {}
      for (const v of vars) {
        if (v.enabled && v.key) {
          map[v.key] = v.value
        }
      }
      return map
    } catch {
      return {}
    }
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
      const vars = await loadEnvVars()
      tab.response = await invokeT('execute_request', {
        req: tab.request,
        vars,
        requestId: tab.requestId ?? undefined,
      })
    } catch (e) {
      tab.error = e instanceof Error ? e.message : String(e)
    } finally {
      tab.isLoading = false
      appStore.bumpSidebar()
    }
  }

  /**
   * 保存当前 active Tab 的请求到 DB
   * - 首次保存(requestId === null):需要 collectionId + name,调 create_request
   * - 后续保存(requestId !== null):调 update_request
   */
  async function saveActive(
    collectionId?: string,
    name?: string,
  ): Promise<RequestItem | null> {
    const tab = activeTab.value
    if (!tab) return null

    if (tab.requestId === null) {
      if (!collectionId || !name?.trim()) return null
      const item = await invokeT('create_request', {
        new: {
          collection_id: collectionId,
          name: name.trim(),
          method: tab.request.method,
          url: tab.request.url,
          headers: tab.request.headers,
          query: tab.request.query,
          body: tab.request.body,
          auth: tab.request.auth,
        },
      })
      tab.requestId = item.id
      tab.requestName = item.name
      tab.title = item.name
      tab.customTitle = true
      tab.isDirty = false
      return item
    } else {
      await invokeT('update_request', {
        id: tab.requestId,
        name: tab.requestName,
        method: tab.request.method,
        url: tab.request.url,
        headers: tab.request.headers,
        query: tab.request.query,
        body: tab.request.body,
        auth: tab.request.auth,
      })
      tab.isDirty = false
      return null
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
    loadRequest,
    loadHistory,
    activate,
    closeTab,
    closeOthers,
    closeAllTabs,
    closeLeft,
    closeRight,
    updateActiveRequest,
    setTitle,
    sendActive,
    saveActive,
    ensureNonEmpty,
  }
})

export type { HttpMethod }
