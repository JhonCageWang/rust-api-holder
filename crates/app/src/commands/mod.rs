//! Tauri Commands 注册
//!
//! 业务模块,每个文件负责一类 Tauri Command:
//!
//! - [`request`]      — HTTP 请求执行
//! - [`collections`]  — 集合
//! - [`requests`]     — 请求
//! - [`environments`] — 环境
//! - [`variables`]    — 环境变量
//! - [`history`]      — 历史记录
//!
//! 所有命令在 `main.rs` 的 `tauri::generate_handler!` 里集中注册。

pub mod collections;
pub mod environments;
pub mod history;
pub mod request;
pub mod requests;
pub mod variables;

use api_holder_core::types::AppInfo;

use crate::AppState;

/// 心跳命令(测试 IPC 通不通)
#[tauri::command]
pub fn ping() -> String {
    "pong".into()
}

/// 应用信息(版本、db 状态等)
#[tauri::command]
pub fn app_info(state: tauri::State<'_, AppState>) -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        db_status: if state.db.path().to_str() == Some("<in-memory>") {
            "in-memory".into()
        } else {
            "file".into()
        },
    }
}