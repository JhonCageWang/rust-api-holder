//! 统一错误类型
//!
//! 库内部用 [`Error`] + [`Result`],把所有底层错误(reqwest / serde_json /
//! rusqlite 等)统一包装。调用方只需关心 [`Error`] 这一个类型。
//!
//! ## 错误设计原则
//!
//! 1. **每个错误变体携带上下文**(用 `String` 描述)
//! 2. **`#[from]` 派生让 `?` 自动转换**
//! 3. **业务错误显式定义**(NotFound / InvalidInput),不滥用 `Other`

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// HTTP 请求执行失败(网络问题、超时等)
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON 序列化 / 反序列化失败
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// SQLite 操作失败
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// 资源不存在(如 collection_id 找不到)
    #[error("Not found: {0}")]
    NotFound(String),

    /// 用户输入不合法(空字符串、格式错误等)
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// IO 操作失败(读文件、写文件等)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 导入第三方格式时解析失败
    #[error("Import error: {0}")]
    Import(String),

    /// 兜底错误
    #[error("{0}")]
    Other(String),
}

/// 库的 Result 类型别名,等价于 `std::result::Result<T, Error>`
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::NotFound("collection-123".to_string());
        assert_eq!(err.to_string(), "Not found: collection-123");
    }

    #[test]
    fn test_error_from_io() {
        // 演示 ? 操作符自动转换 IO 错误
        fn read_file() -> Result<String> {
            std::fs::read_to_string("/nonexistent/path")?;
            Ok("never".to_string())
        }
        let result = read_file();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Io(_)));
    }
}