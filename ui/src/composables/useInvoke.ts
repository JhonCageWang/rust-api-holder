/**
 * Tauri invoke 封装
 *
 * 支持两种模式:
 * 1. **Tauri 模式**(生产 / `cargo tauri dev`):真正调 Rust 后端
 * 2. **浏览器模式**(`pnpm dev`):用 mock 数据替代,前端独立开发
 *
 * 自动检测:用 `window.__TAURI_INTERNALS__` 判断是否在 Tauri 环境。
 *
 * 使用示例:
 * ```ts
 * const info = await invokeT('app_info', undefined)
 * // info 类型自动推断为 AppInfo
 * ```
 */

import { invoke } from '@tauri-apps/api/core'

import type {
  ApiRequest,
  ApiResponse,
  AppInfo,
  Collection,
  Environment,
  HistoryEntry,
  NewRequest,
  RequestItem,
  Variable,
} from '@/types/api'

/** 是否在 Tauri 环境(运行时检测) */
const isTauri =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

if (!isTauri) {
  // 仅在浏览器开发模式提示一次
  console.info(
    '%c[mock mode] 前端在浏览器中运行,invoke 调用走 mock 数据',
    'color: orange; font-weight: bold',
  )
}

/** Tauri Command 签名映射 — 增删 command 时需要同步 */
export interface CommandSignatures {
  ping: { args: undefined; returns: string }
  app_info: { args: undefined; returns: AppInfo }

  // ===== HTTP =====
  execute_request: {
    args: { req: ApiRequest; vars?: Record<string, string> }
    returns: ApiResponse
  }

  // ===== 集合 =====
  list_collections: { args: undefined; returns: Collection[] }
  create_collection: {
    args: { name: string; description: string | null; parent_id: string | null }
    returns: Collection
  }
  rename_collection: { args: { id: string; new_name: string }; returns: null }
  delete_collection: { args: { id: string }; returns: null }
  count_collection_requests: { args: { id: string }; returns: number }

  // ===== 请求 =====
  list_requests: { args: { collection_id: string }; returns: RequestItem[] }
  get_request: { args: { id: string }; returns: RequestItem }
  create_request: { args: { new: NewRequest }; returns: RequestItem }
  rename_request: { args: { id: string; new_name: string }; returns: null }
  update_request_url: { args: { id: string; new_url: string }; returns: null }
  update_request_method: {
    args: { id: string; new_method: ApiRequest['method'] }
    returns: null
  }
  update_request_headers: { args: { id: string; headers: any[] }; returns: null }
  update_request_query: { args: { id: string; query: any[] }; returns: null }
  update_request_body: { args: { id: string; body: any }; returns: null }
  update_request_auth: { args: { id: string; auth: any }; returns: null }
  delete_request: { args: { id: string }; returns: null }
  search_requests: { args: { keyword: string }; returns: RequestItem[] }

  // ===== 环境 =====
  list_environments: { args: undefined; returns: Environment[] }
  get_active_environment: { args: undefined; returns: Environment | null }
  create_environment: { args: { name: string }; returns: Environment }
  rename_environment: { args: { id: string; new_name: string }; returns: null }
  set_active_environment: { args: { id: string }; returns: null }
  delete_environment: { args: { id: string }; returns: null }

  // ===== 变量 =====
  list_variables: { args: { environment_id: string }; returns: Variable[] }
  create_variable: {
    args: { environment_id: string; key: string; value: string }
    returns: Variable
  }
  update_variable: {
    args: { id: string; new_value: string; enabled: boolean }
    returns: null
  }
  delete_variable: { args: { id: string }; returns: null }
  bulk_replace_variables: {
    args: { environment_id: string; variables: Variable[] }
    returns: null
  }

  // ===== 历史 =====
  list_history: { args: { limit: number; offset: number }; returns: HistoryEntry[] }
  delete_history: { args: { id: string }; returns: null }
  delete_old_history: { args: { days: number }; returns: number }
  count_history: { args: undefined; returns: number }
}

export type CommandName = keyof CommandSignatures

/**
 * 类型安全的 Tauri invoke
 */
export async function invokeT<C extends CommandName>(
  command: C,
  args?: CommandSignatures[C]['args'],
): Promise<CommandSignatures[C]['returns']> {
  if (!isTauri) {
    return mockInvoke(command, args)
  }
  try {
    return await invoke<CommandSignatures[C]['returns']>(
      command,
      args as Record<string, unknown> | undefined,
    )
  } catch (e) {
    // Tauri 把 anyhow::Error 转成 string
    const message = typeof e === 'string' ? e : (e as Error).message
    throw new Error(`[${command}] ${message}`)
  }
}

// ─── Mock 数据存储(浏览器开发模式) ──────────────────────────────────

/** 持久化到 localStorage,刷新不丢 */
const LS_KEY = 'api-holder-mock-db'

interface MockDb {
  collections: Collection[]
  requests: RequestItem[]
  environments: Environment[]
  variables: Variable[]
  history: HistoryEntry[]
}

function emptyDb(): MockDb {
  return {
    collections: [
      {
        id: 'mock-coll-1',
        name: '示例集合',
        description: '欢迎使用 Rust API Holder',
        parent_id: null,
        sort_order: 0,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    ],
    requests: [
      {
        id: 'mock-req-1',
        collection_id: 'mock-coll-1',
        name: 'Get user',
        method: 'GET',
        url: 'https://httpbin.org/get',
        headers: [],
        query: [],
        body: { type: 'none' },
        auth: { type: 'none' },
        sort_order: 0,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    ],
    environments: [
      {
        id: 'mock-env-dev',
        name: 'Dev',
        is_active: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
      {
        id: 'mock-env-prod',
        name: 'Prod',
        is_active: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    ],
    variables: [
      {
        id: 'mock-var-1',
        environment_id: 'mock-env-dev',
        key: 'host',
        value: 'api.dev.example.com',
        enabled: true,
      },
      {
        id: 'mock-var-2',
        environment_id: 'mock-env-dev',
        key: 'token',
        value: 'dev-token-xxx',
        enabled: true,
      },
    ],
    history: [],
  }
}

function loadMockDb(): MockDb {
  if (typeof localStorage === 'undefined') return emptyDb()
  try {
    const raw = localStorage.getItem(LS_KEY)
    if (!raw) {
      const fresh = emptyDb()
      localStorage.setItem(LS_KEY, JSON.stringify(fresh))
      return fresh
    }
    return JSON.parse(raw) as MockDb
  } catch {
    return emptyDb()
  }
}

function saveMockDb(db: MockDb): void {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(LS_KEY, JSON.stringify(db))
  }
}

function mockId(prefix: string): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

/**
 * Mock 实现 — 浏览器开发模式用
 */
async function mockInvoke<C extends CommandName>(
  command: C,
  args?: CommandSignatures[C]['args'],
): Promise<CommandSignatures[C]['returns']> {
  await new Promise((r) => setTimeout(r, 50)) // 模拟一点点延迟

  if (import.meta.env.DEV) {
    console.debug(`[mock] ${command}(${JSON.stringify(args) ?? 'undefined'})`)
  }

  const db = loadMockDb()

  switch (command) {
    case 'ping':
      return 'pong' as CommandSignatures[C]['returns']

    case 'app_info':
      return {
        name: 'api-holder',
        version: '0.1.0-dev',
        db_status: 'mock',
      } as CommandSignatures[C]['returns']

    case 'execute_request': {
      const a = args as { req: ApiRequest; vars?: Record<string, string> }
      const echoBody = JSON.stringify(
        {
          mock: true,
          tip: '这是浏览器 mock 响应。',
          sent: {
            method: a.req.method,
            url: a.req.url,
            headers: a.req.headers,
            query: a.req.query,
            body: a.req.body,
            auth: a.req.auth,
          },
          vars: a.vars ?? {},
        },
        null,
        2,
      )
      return {
        status: 200,
        status_text: 'OK (mock)',
        headers: [
          { key: 'content-type', value: 'application/json', enabled: true },
        ],
        body: echoBody,
        duration_ms: 123,
        size_bytes: echoBody.length,
      } as CommandSignatures[C]['returns']
    }

    case 'list_collections':
      return db.collections as CommandSignatures[C]['returns']

    case 'create_collection': {
      const a = args as { name: string; description: string | null; parent_id: string | null }
      const c: Collection = {
        id: mockId('mock-coll'),
        name: a.name,
        description: a.description,
        parent_id: a.parent_id,
        sort_order: db.collections.length,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }
      db.collections.push(c)
      saveMockDb(db)
      return c as CommandSignatures[C]['returns']
    }

    case 'rename_collection': {
      const a = args as { id: string; new_name: string }
      const c = db.collections.find((x) => x.id === a.id)
      if (!c) throw new Error('collection not found')
      c.name = a.new_name
      c.updated_at = new Date().toISOString()
      saveMockDb(db)
      return null as CommandSignatures[C]['returns']
    }

    case 'delete_collection': {
      const a = args as { id: string }
      db.collections = db.collections.filter((c) => c.id !== a.id)
      db.requests = db.requests.filter((r) => r.collection_id !== a.id)
      saveMockDb(db)
      return null as CommandSignatures[C]['returns']
    }

    case 'count_collection_requests': {
      const a = args as { id: string }
      return db.requests.filter((r) => r.collection_id === a.id)
        .length as CommandSignatures[C]['returns']
    }

    case 'list_requests': {
      const a = args as { collection_id: string }
      return db.requests.filter((r) => r.collection_id === a.collection_id) as CommandSignatures[C]['returns']
    }

    case 'get_request': {
      const a = args as { id: string }
      const r = db.requests.find((x) => x.id === a.id)
      if (!r) throw new Error('request not found')
      return r as CommandSignatures[C]['returns']
    }

    case 'create_request': {
      const a = args as { new: NewRequest }
      const r: RequestItem = {
        id: mockId('mock-req'),
        collection_id: a.new.collection_id,
        name: a.new.name,
        method: a.new.method,
        url: a.new.url,
        headers: a.new.headers,
        query: a.new.query,
        body: a.new.body,
        auth: a.new.auth,
        sort_order: db.requests.length,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }
      db.requests.push(r)
      saveMockDb(db)
      return r as CommandSignatures[C]['returns']
    }

    case 'rename_request':
    case 'update_request_url':
    case 'update_request_method':
    case 'update_request_headers':
    case 'update_request_query':
    case 'update_request_body':
    case 'update_request_auth': {
      const a = args as { id: string }
      const r = db.requests.find((x) => x.id === a.id)
      if (!r) throw new Error('request not found')
      r.updated_at = new Date().toISOString()
      saveMockDb(db)
      return null as CommandSignatures[C]['returns']
    }

    case 'delete_request': {
      const a = args as { id: string }
      db.requests = db.requests.filter((r) => r.id !== a.id)
      saveMockDb(db)
      return null as CommandSignatures[C]['returns']
    }

    case 'search_requests': {
      const a = args as { keyword: string }
      const kw = a.keyword.toLowerCase()
      return db.requests.filter(
        (r) => r.name.toLowerCase().includes(kw) || r.url.toLowerCase().includes(kw),
      ) as CommandSignatures[C]['returns']
    }

    case 'list_environments':
      return db.environments as CommandSignatures[C]['returns']

    case 'get_active_environment':
      return (db.environments.find((e) => e.is_active) ?? null) as CommandSignatures[C]['returns']

    case 'create_environment': {
      const a = args as { name: string }
      const e: Environment = {
        id: mockId('mock-env'),
        name: a.name,
        is_active: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }
      db.environments.push(e)
      saveMockDb(db)
      return e as CommandSignatures[C]['returns']
    }

    case 'rename_environment':
    case 'set_active_environment':
    case 'delete_environment': {
      saveMockDb(db)
      return null as CommandSignatures[C]['returns']
    }

    case 'list_variables': {
      const a = args as { environment_id: string }
      return db.variables.filter((v) => v.environment_id === a.environment_id) as CommandSignatures[C]['returns']
    }

    case 'create_variable': {
      const a = args as { environment_id: string; key: string; value: string }
      const v: Variable = {
        id: mockId('mock-var'),
        environment_id: a.environment_id,
        key: a.key,
        value: a.value,
        enabled: true,
      }
      db.variables.push(v)
      saveMockDb(db)
      return v as CommandSignatures[C]['returns']
    }

    case 'update_variable':
    case 'delete_variable':
    case 'bulk_replace_variables': {
      saveMockDb(db)
      return null as CommandSignatures[C]['returns']
    }

    case 'list_history': {
      const a = args as { limit: number; offset: number }
      return db.history.slice(a.offset, a.offset + a.limit) as CommandSignatures[C]['returns']
    }

    case 'delete_history': {
      const a = args as { id: string }
      db.history = db.history.filter((h) => h.id !== a.id)
      saveMockDb(db)
      return null as CommandSignatures[C]['returns']
    }

    case 'delete_old_history':
    case 'count_history': {
      return (db.history.length) as CommandSignatures[C]['returns']
    }

    default: {
      const _exhaustive: never = command
      throw new Error(
        `Mock not implemented for command: ${String(_exhaustive)}`,
      )
    }
  }
}