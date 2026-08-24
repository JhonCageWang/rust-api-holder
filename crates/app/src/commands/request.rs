//! Tauri Command: 执行一个 HTTP 请求
//!
//! ## 完整调用链(Webview ↔ Rust 主进程)
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │  Webview 进程(Chromium / Edge WebView2)                          │
//! │                                                                  │
//! │  HomeView.vue                                                    │
//! │    └─ → sendRequest()                                            │
//! │         └─ → invokeT('execute_request', { req, vars })           │
//! │              └─ → useInvoke.ts                                   │
//! │                   isTauri 模式:                                  │
//! │                     window.__TAURI_INTERNALS__.invoke(...)       │
//! │                   (Tauri 注入的 JS 桥)                          │
//! └──────────────────────────────────────────────────────────────────┘
//!                                  │
//!                                  │  IPC: JSON-RPC over stdin/stdout
//!                                  ▼                                  │
//! ┌──────────────────────────────────────────────────────────────────┐
//! │  Rust 主进程(Tauri 2)                                            │
//! │                                                                  │
//! │  Tauri 框架收到调用,匹配到 #[tauri::command]:                   │
//! │    execute_request(state, req, vars)                             │
//! │      │  serde 反序列化参数                                        │
//! │      │  state.http_client(共享的 reqwest::Client)               │
//! │      ▼                                                           │
//! │  api_holder_core::http::execute_with_vars(client, req, &vars)  │
//! │      │  变量插值 → 拼 RequestBuilder → reqwest 发送              │
//! │      ▼                                                           │
//! │  Response → serde 序列化为 JSON → IPC 回 Webview                 │
//! └──────────────────────────────────────────────────────────────────┘
//!                                  │
//!                                  ▼
//! 回到 HomeView.sendRequest() 的 await 后,response.value 被赋值,
//! Vue 响应式触发 ResponseViewer 重新渲染。
//!
//! ## 错误处理
//!
//! Tauri 2 要求 `Result<T, E>` 中的 `E` 实现 [`Serialize`]。
//! 我们用最简单的方案:把 [`api_holder_core::Error`] 兜底转成 `String`
//! (它已经实现了 Display,to_string() 拿到完整错误消息)。
//! 前端 `useInvoke.ts` catch 块已经知道 `e` 可能就是字符串。
//!
//! ## 为什么接收 `State<AppState>`
//!
//! reqwest::Client 内部有连接池,频繁新建会丢失池化。
//! AppState 里持有一个共享 Client,所有请求复用,生产环境必须这样做。

use std::collections::HashMap;

use api_holder_core::http::{self, Request, Response};

use crate::AppState;

/// 发送一个 HTTP 请求,返回响应。
///
/// ## 参数
///
/// - `state`: Tauri 注入的应用状态(持有共享的 `reqwest::Client`)
/// - `req`: 完整请求(method / url / headers / query / body / auth)
/// - `vars`: 环境变量插值表(`{{var}}` → 值),可选,没传就是空(不插值)
///
/// ## 返回
///
/// 成功 → `Ok(Response)`,失败 → `Err(String)`(已经 to_string 过的错误消息)
#[tauri::command]
pub async fn execute_request(
    state: tauri::State<'_, AppState>,
    req: Request,
    vars: Option<HashMap<String, String>>,
) -> Result<Response, String> {
    // vars 是 Option,JS 不传时就是 None → 默认空 HashMap → 不做插值
    let vars = vars.unwrap_or_default();

    // 实际干活在 core 库,这里只是薄壳。错误转成 string 给前端。
    http::execute_with_vars(&state.http_client, req, &vars)
        .await
        .map_err(|e| e.to_string())
}

// ───────────────────────────────────────────────────────────────────
// 用 `tauri::generate_handler!` 在 main.rs 里注册时,
// 这条命令的路径是: `commands::request::execute_request`
// ───────────────────────────────────────────────────────────────────
