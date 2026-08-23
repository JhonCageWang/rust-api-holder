//! 🖥️ api-holder — Tauri 桌面应用入口
//!
//! 职责:
//! 1. 启动 Tauri Builder,加载前端 webview
//! 2. 初始化 core 中的数据库 + 日志
//! 3. 注册 Tauri Commands(把 core 的能力暴露给前端)
//!
//! TODO(Week 4): 完整的 Commands 注册

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use std::sync::Mutex;

use tauri::Manager;

use api_holder_core::storage::Database;

/// 全局应用状态(由前端通过 Tauri Commands 访问)
pub struct AppState {
    /// SQLite 数据库连接(Week 3 才实装,目前只占位)
    pub db: Mutex<Option<Database>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db: Mutex::new(None),
        }
    }
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api_holder_core=debug".into()),
        )
        .init();

    tracing::info!("🦀 Rust API Holder starting up...");

    tauri::Builder::default()
        .setup(|app| {
            // 解析应用数据目录(后续 Week 3 用于放 SQLite 文件)
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            tracing::info!("app data dir: {}", data_dir.display());

            // TODO(Week 3): 在这里初始化 Database
            // let db_path = data_dir.join("api-holder.db");
            // let db = Database::open(&db_path)?;
            // app.manage(AppState { db: Mutex::new(Some(db)) });

            // 临时 manage 一个空的 AppState,前端 Week 4 才能跑起来
            app.manage(AppState::default());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::app_info,
            commands::request::execute_request,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}