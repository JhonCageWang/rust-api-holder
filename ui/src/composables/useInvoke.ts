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

import type { ApiRequest, ApiResponse, AppInfo } from '@/types/api'

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
  /** 发送一个 HTTP 请求,返回响应。vars 是环境变量插值表。 */
  execute_request: {
    args: { req: ApiRequest; vars?: Record<string, string> }
    returns: ApiResponse
  }
  // TODO(Week 4+): 在这里添加更多 command 签名
  // create_collection: { args: { name: string }, returns: Collection }
  // ...
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

/**
 * Mock 实现 — 浏览器开发模式用
 *
 * Week X 实现某个 command 时,在这里加对应的 mock。
 * 这样前端不依赖 Rust 也能跑通流程。
 */
async function mockInvoke<C extends CommandName>(
  command: C,
  args?: CommandSignatures[C]['args'],
): Promise<CommandSignatures[C]['returns']> {
  // 模拟网络延迟,让 loading 状态真实一点
  await new Promise((r) => setTimeout(r, 200))

  // 在 mock 模式下打印调试日志(仅开发环境)
  if (import.meta.env.DEV) {
    console.debug(`[mock] ${command}(${JSON.stringify(args) ?? 'undefined'})`)
  }

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
      // 浏览器开发模式下,不要真的发请求(会跨域、还会被服务器拦)
      // 把请求原样回显成 JSON,这样能看到编辑器的组装是否正确
      const a = args as { req: ApiRequest; vars?: Record<string, string> }
      const echoBody = JSON.stringify(
        {
          mock: true,
          tip: '这是浏览器 mock 响应。在 Tauri 环境里会真的发出去。',
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
          { key: 'x-mock-server', value: 'api-holder-mock/0.1', enabled: true },
        ],
        body: echoBody,
        duration_ms: 123,
        size_bytes: echoBody.length,
      } as CommandSignatures[C]['returns']
    }

    // TODO(Week 4+): 加更多 mock
    // case 'list_collections':
    //   return [{ id: 'mock-1', name: 'Mock Collection', ... }]

    default: {
      // 未实现的 mock,显式报错而不是静默
      const _exhaustive: never = command
      throw new Error(
        `Mock not implemented for command: ${String(_exhaustive)}`,
      )
    }
  }
}