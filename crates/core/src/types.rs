//! 跨模块的共享类型(给前端用的)
//!
//! 主要放 `#[derive(Serialize, Deserialize)]` 的 DTO,
//! 方便 commands 把数据序列化给前端。

use serde::{Deserialize, Serialize};

/// 应用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    /// `"file"` / `"in-memory"` / `"mock"`(前端 mock 模式)
    pub db_status: String,
}