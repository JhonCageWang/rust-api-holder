//! 🖥️ api-holder — Tauri 桌面应用入口
//!
//! 职责:
//! 1. 启动 Tauri Builder,加载前端 webview
//! 2. 初始化 core 中的数据库 + 共享的 reqwest::Client
//! 3. 注册 Tauri Commands(把 core 的能力暴露给前端)
//!
//! TODO(Week 3): 完整 Database 初始化

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use std::sync::Mutex;

use tauri::Manager;

use api_holder_core::http::default_client;
use api_holder_core::storage::Database;

/// 全局应用状态(由前端通过 Tauri Commands 访问)
///
/// 持有:
/// - `db`: SQLite 数据库连接(Week 3 才实装,目前只占位)
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
/// 真的需要修改配置(如换 cookie store)时,用 `Arc<Client>` 或 `arc-swap`
/// 原子替换,不要阻塞所有读端。
pub struct AppState {
    /// SQLite 数据库连接(Week 3 才实装,目前只占位)
    ///
    /// SQLite 连接是**真的需要 Mutex**(rusqlite::Connection 不是线程安全)。
    pub db: Mutex<Option<Database>>,
    /// 共享的 HTTP 客户端(连接池复用)。
    ///
    /// 直接放裸 `Client`,**不要**包 Mutex,见上面 struct-level 注释。
    pub http_client: reqwest::Client,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db: Mutex::new(None),
            // 30 秒超时,跟 core 里的 default_client() 保持一致
            http_client: default_client().expect("failed to build default http client"),
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
            // app.manage(AppState { db: Mutex::new(Some(db)), http_client });

            // 临时 manage 一个带共享 Client 的 AppState,前端 Week 4 才能跑起来
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
