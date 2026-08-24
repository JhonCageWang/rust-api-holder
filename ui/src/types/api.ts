/**
 * Tauri Command 接口类型定义
 *
 * 这里定义的类型必须和 `crates/app/src/commands/*.rs` 里
 * `#[tauri::command]` 函数的参数 + 返回值严格对应。
 *
 * 任何新增/修改 Tauri Command 时,需要同步更新这里。
 */

// ===== 通用类型 =====

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS'

export interface KeyValue {
  key: string
  value: string
  enabled: boolean
}

export interface RequestBody {
  type: 'none' | 'json' | 'form' | 'raw'
  content?: string
  content_type?: string
  fields?: KeyValue[]
}

export type Auth =
  | { type: 'none' }
  | { type: 'bearer'; token: string }
  | { type: 'basic'; username: string; password: string }
  | { type: 'api_key'; key: string; value: string; in_header: boolean }

export interface ApiRequest {
  method: HttpMethod
  url: string
  headers: KeyValue[]
  query: KeyValue[]
  body: RequestBody
  auth: Auth
}

export interface ApiResponse {
  status: number
  status_text: string
  headers: KeyValue[]
  body: string
  duration_ms: number
  size_bytes: number
}

// ===== 集合 / 请求 =====

export interface Collection {
  id: string
  name: string
  description: string | null
  parent_id: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

/** 创建集合时的输入参数(对应 Rust 的 `NewCollection`) */
export interface NewCollection {
  name: string
  description: string | null
  parent_id: string | null
}

export interface RequestItem {
  id: string
  collection_id: string
  name: string
  method: HttpMethod
  url: string
  headers: KeyValue[]
  query: KeyValue[]
  body: RequestBody
  auth: Auth
  sort_order: number
  created_at: string
  updated_at: string
}

/** 创建请求时的输入参数(对应 Rust 的 `NewRequest`) */
export interface NewRequest {
  collection_id: string
  name: string
  method: HttpMethod
  url: string
  headers: KeyValue[]
  query: KeyValue[]
  body: RequestBody
  auth: Auth
}

// ===== 环境变量 =====

export interface Environment {
  id: string
  name: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface Variable {
  id: string
  environment_id: string
  key: string
  value: string
  enabled: boolean
}

// ===== 历史 =====

export interface HistoryEntry {
  id: string
  request_id: string | null
  request_snapshot: ApiRequest
  response: ApiResponse | null
  error: string | null
  sent_at: string
}

// ===== 应用信息 =====

export interface AppInfo {
  name: string
  version: string
  db_status: string
}