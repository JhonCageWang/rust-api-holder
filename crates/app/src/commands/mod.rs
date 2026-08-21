//! Tauri Commands(给前端调用的函数)
//!
//! 每个 `#[tauri::command]` 函数都自动生成前端 invoke 接口:
//! ```ts
//! import { invoke } from '@tauri-apps/api/core'
//! const result = await invoke('ping', { /* args */ })
//! ```
//!
//! 设计原则:
//! 1. **参数和返回值都用 serde 序列化**(前端拿到的是 JSON)
//! 2. **业务逻辑放 core**,Commands 只是薄壳
//! 3. **错误用 anyhow::Result**(链到前端是 string)
//!
//! TODO(Week 4-6): 完整的 Commands 实现(collection / request / environment / history)

use serde::Serialize;

use crate::AppState;

/// 健康检查(给前端确认后端已就绪)
#[tauri::command]
pub fn ping() -> &'static str {
    "pong"
}

/// 返回应用元信息(版本、数据库路径等)
#[tauri::command]
pub fn app_info(state: tauri::State<'_, AppState>) -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        // TODO(Week 3): db_status 根据 state.db 实际状态返回
        db_status: if state.db.lock().unwrap().is_some() {
            "ready".to_string()
        } else {
            "not initialized".to_string()
        },
    }
}

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub db_status: String,
}