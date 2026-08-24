//! 🖥️ api-holder — Tauri 桌面应用入口
//!
//! 职责:
//! 1. 启动 Tauri Builder,加载前端 webview
//! 2. 初始化 SQLite 数据库(`api-holder.db`,在 AppData 目录里)
//! 3. 初始化共享的 reqwest::Client
//! 4. 注册 Tauri Commands(把 core 的能力暴露给前端)

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use std::path::PathBuf;

use tauri::Manager;

use api_holder_core::http::default_client;
use api_holder_core::storage::Database;

/// 全局应用状态(由前端通过 Tauri Commands 访问)
///
/// 持有:
/// - `db`: SQLite 数据库(`Database` 内部已包 Mutex,线程安全)
/// - `http_client`: 共享的 reqwest::Client(所有 HTTP 请求复用,享受连接池)
///
/// ## ⚠️ 为什么 http_client **不**加 Mutex?
///
/// `reqwest::Client` 本身是 Send + Sync + 内部 Arc-like 的:
///
/// - 它的方法签名都是 `&self`,多个任务可以**并发持有共享引用**
/// - 连接池是它的内部状态,自动并发管理
/// - 加 `Mutex<Client>` 会强制串行 + 锁跨 await,反而**失去并发的优势**
///
/// 错误的写法(千万别这么改):
/// ```ignore
/// pub http_client: Mutex<reqwest::Client>,  // ← 串行!
/// ```
///
/// 真的需要修改配置(如换 cookie store) 时,用 `Arc<Client>` 或 `arc-swap`
/// 原子替换,不要阻塞所有读端。
pub struct AppState {
    /// SQLite 数据库(内部已包 Mutex,直接放)
    pub db: Database,
    /// 共享的 HTTP 客户端(连接池复用)。
    ///
    /// 直接放裸 `Client`,**不要**包 Mutex,见上面 struct-level 注释。
    pub http_client: reqwest::Client,
}

/// 解析 app 数据目录,创建子目录,返回 db 路径
fn resolve_db_path(handle: &tauri::AppHandle) -> std::io::Result<PathBuf> {
    let dir = handle
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("api-holder.db"))
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
            let db_path = resolve_db_path(app.handle())
                .expect("failed to resolve db path");
            tracing::info!("db path: {}", db_path.display());

            // 打开 SQLite(会自动跑 migrations)
            let db = Database::open(&db_path).expect("failed to open database");
            tracing::info!("database opened, schema migrated");

            // 共享的 reqwest::Client(30 秒超时)
            let http_client = default_client().expect("failed to build http client");

            app.manage(AppState { db, http_client });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 通用
            commands::ping,
            commands::app_info,
            // HTTP 请求执行
            commands::request::execute_request,
            // 集合
            commands::collections::list_collections,
            commands::collections::create_collection,
            commands::collections::rename_collection,
            commands::collections::delete_collection,
            commands::collections::count_collection_requests,
            // 请求
            commands::requests::list_requests,
            commands::requests::get_request,
            commands::requests::create_request,
            commands::requests::rename_request,
            commands::requests::update_request_url,
            commands::requests::update_request_method,
            commands::requests::update_request_headers,
            commands::requests::update_request_query,
            commands::requests::update_request_body,
            commands::requests::update_request_auth,
            commands::requests::delete_request,
            commands::requests::search_requests,
            // 环境
            commands::environments::list_environments,
            commands::environments::get_active_environment,
            commands::environments::create_environment,
            commands::environments::rename_environment,
            commands::environments::set_active_environment,
            commands::environments::delete_environment,
            // 变量
            commands::variables::list_variables,
            commands::variables::create_variable,
            commands::variables::update_variable,
            commands::variables::delete_variable,
            commands::variables::bulk_replace_variables,
            // 历史
            commands::history::list_history,
            commands::history::delete_history,
            commands::history::delete_old_history,
            commands::history::count_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}