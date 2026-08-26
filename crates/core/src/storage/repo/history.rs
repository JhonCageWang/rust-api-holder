//! 历史记录 Repository
//!
//! 每次发请求成功 / 失败都记一条,前端可以查、重发、清理。
//!
//! ## 字段设计
//!
//! - `request_id`:**可空**,指向原始请求;请求被删后变 NULL(ON DELETE SET NULL)
//! - `request_snapshot`:发起时的**完整请求 JSON** — 即使原请求改了/删了也能重放
//! - `status_code`, `response_headers`, `response_body`, `duration_ms`, `error`
//!   任一可能为空(网络错误时无 status_code,等等)
//!
//! ## 性能考虑
//!
//! - `sent_at DESC` 索引支持"最近 N 条"快速查询
//! - 数据量大时可以用 [`delete_older_than`](Self::delete_older_than) 定期清理

use rusqlite::{params, Row};

use crate::error::Error;
use crate::http::{KeyValue, Method, Request, Response};
use crate::history::HistoryEntry;
use crate::Result;

use super::{from_json, from_unix, parse_uuid, parse_uuid_opt, to_json, to_unix};

/// 历史 Repository
pub struct HistoryRepo<'a> {
    db: &'a crate::storage::Database,
}

impl<'a> HistoryRepo<'a> {
    pub(crate) fn new(db: &'a crate::storage::Database) -> Self {
        Self { db }
    }

    /// 记录一次请求
    ///
    /// `entry.response` 为 `None` 表示请求失败(网络错等);`entry.error` 有值。
    /// `method` 和 `url` 从 `request_snapshot` 里拿(快照自带),不单独存。
    pub fn record(&self, entry: &HistoryEntry) -> Result<()> {
        let id_str = entry.id.to_string();
        let req_id_str = entry.request_id.map(|u| u.to_string());
        let method_str = method_to_str(entry.request_snapshot.method);
        let url_str = entry.request_snapshot.url.clone();
        let req_snapshot = to_json(&entry.request_snapshot)?;
        let resp_headers = entry
            .response
            .as_ref()
            .map(|r| to_json(&r.headers))
            .transpose()?;
        let sent_at_ts = to_unix(entry.sent_at);

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO history
                 (id, request_id, method, url, request_snapshot,
                  status_code, response_headers, response_body, duration_ms,
                  error, sent_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    id_str,
                    req_id_str,
                    method_str,
                    url_str,
                    req_snapshot,
                    entry.response.as_ref().map(|r| r.status as i64),
                    resp_headers,
                    entry.response.as_ref().map(|r| r.body.clone()),
                    entry.response.as_ref().map(|r| r.duration_ms as i64),
                    entry.error,
                    sent_at_ts,
                ],
            )?;
            Ok(())
        })
    }

    /// 列出最近 N 条历史(分页)
    pub fn list(&self, limit: i64, offset: i64) -> Result<Vec<HistoryEntry>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, request_id, method, url, request_snapshot,
                        status_code, response_headers, response_body,
                        duration_ms, error, sent_at
                 FROM history
                 ORDER BY sent_at DESC
                 LIMIT ?1 OFFSET ?2",
            )?;
            let rows = stmt.query_map(params![limit, offset], row_to_history)?;
            rows.collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
                .map_err(Error::from)
        })
    }

    /// 找单条
    pub fn find_by_id(&self, id: uuid::Uuid) -> Result<HistoryEntry> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, request_id, method, url, request_snapshot,
                        status_code, response_headers, response_body,
                        duration_ms, error, sent_at
                 FROM history WHERE id = ?1",
                params![id_str],
                row_to_history,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("history {id}")),
                other => Error::Database(other),
            })
        })
    }

    /// 删除单条
    pub fn delete(&self, id: uuid::Uuid) -> Result<()> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "DELETE FROM history WHERE id = ?1",
                params![id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("history {id}")));
            }
            Ok(())
        })
    }

    /// 删除某个请求关联的所有历史(请求被删时自动调用)
    pub fn delete_by_request(&self, request_id: uuid::Uuid) -> Result<usize> {
        let id_str = request_id.to_string();
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "DELETE FROM history WHERE request_id = ?1",
                params![id_str],
            )?;
            Ok(n)
        })
    }

    /// 清理 N 天前的历史(返回删除的行数)
    pub fn delete_older_than(&self, days: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp() - days * 86400;
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "DELETE FROM history WHERE sent_at < ?1",
                params![cutoff],
            )?;
            Ok(n)
        })
    }

    /// 总数(用于 UI 显示 "(1234 total)")
    pub fn count(&self) -> Result<i64> {
        self.db.with_conn(|conn| {
            let n: i64 = conn.query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))?;
            Ok(n)
        })
    }
}

// ─── 内部辅助 ────────────────────────────────────────────────────────

fn method_to_str(m: Method) -> &'static str {
    match m {
        Method::Get => "GET",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Patch => "PATCH",
        Method::Delete => "DELETE",
        Method::Head => "HEAD",
        Method::Options => "OPTIONS",
    }
}

fn method_from_str(s: &str) -> rusqlite::Result<Method> {
    match s {
        "GET" => Ok(Method::Get),
        "POST" => Ok(Method::Post),
        "PUT" => Ok(Method::Put),
        "PATCH" => Ok(Method::Patch),
        "DELETE" => Ok(Method::Delete),
        "HEAD" => Ok(Method::Head),
        "OPTIONS" => Ok(Method::Options),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown method '{other}'"),
            )),
        )),
    }
}

fn row_to_history(row: &Row<'_>) -> rusqlite::Result<HistoryEntry> {
    let id_str: String = row.get(0)?;
    let req_id_str: Option<String> = row.get(1)?;
    let method_str: String = row.get(2)?;
    let url: String = row.get(3)?;
    let req_snapshot: String = row.get(4)?;
    let status_code: Option<i64> = row.get(5)?;
    let resp_headers: Option<String> = row.get(6)?;
    let response_body: Option<String> = row.get(7)?;
    let duration_ms: Option<i64> = row.get(8)?;
    let error: Option<String> = row.get(9)?;
    let sent_at_ts: i64 = row.get(10)?;

    // 重构完整 Response(headers 反序列化失败就丢成空,不影响整行)
    let response = status_code.map(|status| {
        let status = status as u16;
        let headers: Vec<crate::http::KeyValue> = resp_headers
            .as_deref()
            .and_then(|s| from_json(s).ok())
            .unwrap_or_default();
        let body = response_body.unwrap_or_default();
        Response {
            status,
            status_text: reqwest::StatusCode::from_u16(status)
                .ok()
                .and_then(|s| s.canonical_reason())
                .unwrap_or("")
                .to_string(),
            headers,
            size_bytes: body.len(),
            body,
            duration_ms: duration_ms.unwrap_or(0) as u64,
        }
    });

    let mut request_snapshot: Request = from_json(&req_snapshot).map_err(serde_err)?;
    // 用数据库里存的方法和 URL 覆盖快照的(防 JSON 反序列化出错时还能查)
    let _ = method_from_str(&method_str)?;
    let _ = url;
    // 如果 JSON 里缺失,用行的字段填
    if request_snapshot.url.is_empty() {
        request_snapshot.url = url;
    }

    Ok(HistoryEntry {
        id: parse_uuid(&id_str).map_err(uuid_err)?,
        request_id: parse_uuid_opt(req_id_str).map_err(uuid_opt_err)?,
        request_snapshot,
        response,
        error,
        sent_at: from_unix(sent_at_ts),
    })
}

fn uuid_err(e: Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
    )
}

fn uuid_opt_err(e: Error) -> rusqlite::Error {
    uuid_err(e)
}

fn serde_err(e: Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_db() -> crate::storage::Database {
        crate::storage::Database::open_in_memory().unwrap()
    }

    fn entry_with_status(method: Method, url: &str, status: u16) -> HistoryEntry {
        let mut req = Request::default();
        req.method = method;
        req.url = url.into();
        HistoryEntry {
            id: uuid::Uuid::new_v4(),
            request_id: None,
            request_snapshot: req,
            response: Some(Response {
                status,
                status_text: "OK".into(),
                headers: vec![KeyValue {
                    key: "content-type".into(),
                    value: "application/json".into(),
                    enabled: true,
                }],
                body: "body".into(),
                duration_ms: 100,
                size_bytes: 4,
            }),
            error: None,
            sent_at: chrono::Utc::now(),
        }
    }

    fn entry_with_error() -> HistoryEntry {
        let mut req = Request::default();
        req.url = "https://example.com".into();
        HistoryEntry {
            id: uuid::Uuid::new_v4(),
            request_id: None,
            request_snapshot: req,
            response: None,
            error: Some("network timeout".into()),
            sent_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_record_success() {
        let db = fresh_db();
        let entry = entry_with_status(Method::Get, "https://example.com", 200);
        db.history().record(&entry).unwrap();
        assert_eq!(db.history().count().unwrap(), 1);
    }

    #[test]
    fn test_record_error() {
        let db = fresh_db();
        db.history().record(&entry_with_error()).unwrap();
        let found = db.history().list(10, 0).unwrap();
        assert_eq!(found[0].error.as_deref(), Some("network timeout"));
        assert!(found[0].response.is_none());
    }

    #[test]
    fn test_list_order_desc() {
        let db = fresh_db();
        // 插入 3 条,sent_at 递增
        let mut e1 = entry_with_status(Method::Get, "a", 200);
        e1.sent_at = chrono::Utc::now() - chrono::Duration::seconds(30);
        let mut e2 = entry_with_status(Method::Post, "b", 201);
        e2.sent_at = chrono::Utc::now() - chrono::Duration::seconds(10);
        let e3 = entry_with_status(Method::Put, "c", 200);
        for e in [&e1, &e2, &e3] {
            db.history().record(e).unwrap();
        }

        let list = db.history().list(10, 0).unwrap();
        assert_eq!(list.len(), 3);
        // 按 sent_at DESC,最新(e3) 排第一
        assert_eq!(list[0].request_snapshot.url, "c");
        assert_eq!(list[1].request_snapshot.url, "b");
        assert_eq!(list[2].request_snapshot.url, "a");
    }

    #[test]
    fn test_list_pagination() {
        let db = fresh_db();
        for i in 0..5 {
            let e = entry_with_status(Method::Get, &format!("u{i}"), 200);
            db.history().record(&e).unwrap();
        }
        let page1 = db.history().list(2, 0).unwrap();
        let page2 = db.history().list(2, 2).unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 2);
        // 不同页
        assert_ne!(page1[0].id, page2[0].id);
    }

    #[test]
    fn test_find_by_id() {
        let db = fresh_db();
        let e = entry_with_status(Method::Get, "x", 200);
        db.history().record(&e).unwrap();
        let found = db.history().find_by_id(e.id).unwrap();
        assert_eq!(found.request_snapshot.url, "x");
        let resp = found.response.as_ref().unwrap();
        assert_eq!(resp.status, 200);
        // 完整回显:headers / status_text / size_bytes 都要重建
        assert_eq!(resp.headers.len(), 1);
        assert_eq!(resp.headers[0].key, "content-type");
        assert_eq!(resp.status_text, "OK");
        assert_eq!(resp.size_bytes, 4);
    }

    #[test]
    fn test_delete() {
        let db = fresh_db();
        let e = entry_with_status(Method::Get, "x", 200);
        db.history().record(&e).unwrap();
        db.history().delete(e.id).unwrap();
        assert!(matches!(
            db.history().find_by_id(e.id),
            Err(Error::NotFound(_))
        ));
    }

    #[test]
    fn test_delete_older_than() {
        let db = fresh_db();
        let mut old = entry_with_status(Method::Get, "old", 200);
        old.sent_at = chrono::Utc::now() - chrono::Duration::days(10);
        db.history().record(&old).unwrap();
        let recent = entry_with_status(Method::Get, "recent", 200);
        db.history().record(&recent).unwrap();

        let deleted = db.history().delete_older_than(7).unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(db.history().count().unwrap(), 1);
    }
}