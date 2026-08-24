//! 各实体的 Repository
//!
//! ## 文件结构
//!
//! - [`collections`]  — 集合(文件夹),`parent_id` 支持嵌套
//! - [`requests`]     — 请求,归属于 `collection_id`
//! - [`environments`] — 环境,只有 1 个 `is_active = true`
//! - [`variables`]    — 环境变量,归属于 `environment_id`
//! - [`history`]      — 请求历史记录
//!
//! ## 公共辅助
//!
//! - 时间戳 ↔ `chrono::DateTime<Utc>` 的转换(SQLite 用 `INTEGER` 存)
//! - `bool` ↔ `i64` 转换(SQLite 没有原生 bool,用 0/1)
//! - `T` ↔ JSON `String` 转换(复杂字段如 `headers`、`auth` 存为 `TEXT`)
//!
//! 这些转换在每个 Repository 文件的 `row_to_xxx` / `params_from_xxx` 里用到。

pub mod collections;
pub mod environments;
pub mod history;
pub mod requests;
pub mod variables;

pub use collections::CollectionRepo;
pub use environments::EnvironmentRepo;
pub use history::HistoryRepo;
pub use requests::RequestRepo;
pub use variables::VariableRepo;

// ─── 类型转换辅助 ───────────────────────────────────────────────────

/// `chrono::DateTime<Utc>` → unix 秒(i64)
pub(crate) fn to_unix(dt: chrono::DateTime<chrono::Utc>) -> i64 {
    dt.timestamp()
}

/// unix 秒(i64) → `chrono::DateTime<Utc>`
///
/// 如果时间戳无效(范围外),fallback 到 `Utc::now()`,避免 panic。
pub(crate) fn from_unix(ts: i64) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
        .unwrap_or_else(chrono::Utc::now)
}

/// `bool` → `i64`(SQLite 用 0/1)
pub(crate) fn to_sqlite_bool(b: bool) -> i64 {
    if b {
        1
    } else {
        0
    }
}

/// `i64` → `bool`(SQLite 的 0/1)
pub(crate) fn from_sqlite_bool(v: i64) -> bool {
    v != 0
}

/// 任意 `Serialize` 类型 → JSON 字符串(存 TEXT 字段)
pub(crate) fn to_json<T: serde::Serialize>(v: &T) -> crate::Result<String> {
    Ok(serde_json::to_string(v)?)
}

/// JSON 字符串 → 任意 `Deserialize` 类型
pub(crate) fn from_json<T: serde::de::DeserializeOwned>(s: &str) -> crate::Result<T> {
    Ok(serde_json::from_str(s)?)
}

/// `uuid::Uuid` → 字符串(存 TEXT 字段)
pub(crate) fn uuid_to_str(id: uuid::Uuid) -> String {
    id.to_string()
}

/// `Option<uuid::Uuid>` → `Option<String>`
pub(crate) fn uuid_opt_to_str(id: Option<uuid::Uuid>) -> Option<String> {
    id.map(uuid_to_str)
}

/// 字符串 → `uuid::Uuid`,错误包装为 `Error::NotFound`(语义:数据不合法 → "查不到")
///
/// 实际上数据库里出现非法 UUID 几乎一定是 bug,我们用 `Error::Other` 更准确。
pub(crate) fn parse_uuid(s: &str) -> crate::Result<uuid::Uuid> {
    uuid::Uuid::parse_str(s).map_err(|e| crate::Error::Other(format!("invalid UUID '{s}': {e}")))
}

/// `Option<String>` → `Option<uuid::Uuid>`
pub(crate) fn parse_uuid_opt(s: Option<String>) -> crate::Result<Option<uuid::Uuid>> {
    s.map(|s| parse_uuid(&s)).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_roundtrip() {
        assert_eq!(to_sqlite_bool(true), 1);
        assert_eq!(to_sqlite_bool(false), 0);
        assert!(from_sqlite_bool(1));
        assert!(!from_sqlite_bool(0));
    }

    #[test]
    fn test_unix_roundtrip() {
        let now = chrono::Utc::now();
        let ts = to_unix(now);
        let back = from_unix(ts);
        assert_eq!(to_unix(back), ts);
    }

    #[test]
    fn test_json_roundtrip() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct V {
            x: i32,
            y: String,
        }
        let v = V { x: 42, y: "hi".into() };
        let s = to_json(&v).unwrap();
        assert!(s.contains("42"));
        let back: V = from_json(&s).unwrap();
        assert_eq!(back, v);
    }
}