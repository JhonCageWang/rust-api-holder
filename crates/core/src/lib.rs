//! 🧠 api-holder-core
//!
//! 核心业务逻辑库,**纯 Rust,不依赖任何 GUI 框架**。
//! 这样可以:
//! - 单独跑单元测试,无需启动 Tauri
//! - 未来想做 CLI 版本(`api-holder send xxx.json`)很容易
//! - 编译隔离:改 GUI 不会重编核心逻辑
//!
//! ## 模块组织
//!
//! - [`http`]        — HTTP 请求执行(基于 reqwest)
//! - [`collection`]  — 集合 / 请求模型
//! - [`environment`] — 环境变量管理
//! - [`history`]     — 请求历史
//! - [`import`]      — 导入(Postman JSON 等)
//! - [`storage`]     — SQLite 持久化(Repository 模式)
//! - [`types`]       — 共享 DTO 类型
//! - [`error`]       — 统一错误类型

#![warn(missing_docs)]
#![warn(rust_2021_compatibility)]

pub mod collection;
pub mod environment;
pub mod error;
pub mod history;
pub mod http;
pub mod import;
pub mod storage;
pub mod types;

pub use error::{Error, Result};

/// 当前库的语义化版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
