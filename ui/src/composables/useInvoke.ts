/**
 * Tauri invoke 封装
 *
 * 提供:
 * 1. 类型安全的 invoke 调用(自动关联类型契约)
 * 2. 统一的错误处理(把后端错误转成 Promise reject)
 * 3. Loading 状态管理
 *
 * 使用示例:
 * ```ts
 * const info = await invokeT('app_info', undefined)
 * // info 类型自动推断为 AppInfo
 * ```
 */

import { invoke } from '@tauri-apps/api/core'

/** Tauri Command 签名映射 — 增删 command 时需要同步 */
export interface CommandSignatures {
  ping: { args: undefined; returns: string }
  app_info: { args: undefined; returns: import('@/types/api').AppInfo }
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
  try {
    return await invoke<CommandSignatures[C]['returns']>(command, args as any)
  } catch (e) {
    // Tauri 把 anyhow::Error 转成 string
    const message = typeof e === 'string' ? e : (e as Error).message
    throw new Error(`[${command}] ${message}`)
  }
}