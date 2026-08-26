//! 请求 Repository
//!
//! ## 字段存储
//!
//! | 字段 | SQLite 类型 | 序列化 |
//! |------|------------|--------|
//! | `id` | TEXT | UUID 字符串 |
//! | `name` | TEXT | 字符串 |
//! | `method` | TEXT | `"GET"` / `"POST"` 等 |
//! | `url` | TEXT | 字符串 |
//! | `headers` | TEXT | **JSON 字符串**(`Vec<KeyValue>`) |
//! | `query` | TEXT | **JSON 字符串**(`Vec<KeyValue>`) |
//! | `body_type` | TEXT | `"none"` / `"json"` / `"form"` / `"raw"` |
//! | `body_content` | TEXT | 字符串(配合 body_type 解释) |
//! | `auth_type` | TEXT | `"none"` / `"bearer"` / `"basic"` / `"apikey"` |
//! | `auth_config` | TEXT | **JSON 字符串**(`Bearer {token}` 等) |
//!
//! JSON 字段的好处:不用为每个变体建单独的列,加新变体时 schema 不用变。
//! 坏处:不能 SQL 搜索(比如 "所有 bearer auth 的请求"),我们目前不需要。
//!
//! ## 用法
//!
//! ```no_run
//! # use api_holder_core::storage::Database;
//! # use api_holder_core::collection::NewRequest;
//! # use api_holder_core::http::{Method, Auth, Body};
//! # use uuid::Uuid;
//! # let db = Database::open("app.db").unwrap();
//! # let collection_id = Uuid::new_v4();
//! let req = db.requests().create(NewRequest {
//!     collection_id,
//!     name: "Get user".into(),
//!     method: Method::Get,
//!     url: "https://api.example.com/users/1".into(),
//!     headers: vec![],
//!     query: vec![],
//!     body: Body::None,
//!     auth: Auth::None,
//! }).unwrap();
//!
//! // 修改 URL
//! db.requests().update_url(req.id, "https://api.example.com/v2/users/1".into()).unwrap();
//! ```

use rusqlite::{params, Row};

use crate::collection::{NewRequest, RequestItem};
use crate::error::Error;
use crate::http::{Auth, Body, KeyValue, Method};
use crate::Result;

use super::{
    from_json, from_unix, parse_uuid, parse_uuid_opt, to_json, to_unix,
};

/// 请求 Repository
pub struct RequestRepo<'a> {
    db: &'a crate::storage::Database,
}

impl<'a> RequestRepo<'a> {
    pub(crate) fn new(db: &'a crate::storage::Database) -> Self {
        Self { db }
    }

    /// 列出指定集合下的所有请求,按 `sort_order` → `name` 排序。
    pub fn list_by_collection(&self, collection_id: uuid::Uuid) -> Result<Vec<RequestItem>> {
        let cid = collection_id.to_string();
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, collection_id, name, method, url, headers, query_params,
                        body_type, body_content, auth_type, auth_config,
                        sort_order, created_at, updated_at
                 FROM requests
                 WHERE collection_id = ?1
                 ORDER BY sort_order ASC, name ASC",
            )?;
            let rows = stmt.query_map(params![cid], row_to_request)?;
            rows.collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
                .map_err(Error::from)
        })
    }

    /// 查找单个请求
    pub fn find_by_id(&self, id: uuid::Uuid) -> Result<RequestItem> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, collection_id, name, method, url, headers, query_params,
                        body_type, body_content, auth_type, auth_config,
                        sort_order, created_at, updated_at
                 FROM requests WHERE id = ?1",
                params![id_str],
                row_to_request,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    Error::NotFound(format!("request {id}"))
                }
                other => Error::Database(other),
            })
        })
    }

    /// 创建新请求。
    ///
    /// 必填:`collection_id`, `name`, `method`, `url`。
    /// 其他字段给默认值(空 headers / 空 query / `Body::None` / `Auth::None`)。
    pub fn create(&self, new: NewRequest) -> Result<RequestItem> {
        if new.name.trim().is_empty() {
            return Err(Error::InvalidInput("request name 不能为空".into()));
        }
        if new.url.trim().is_empty() {
            return Err(Error::InvalidInput("request url 不能为空".into()));
        }

        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();
        let now_ts = to_unix(now);

        let method_str = method_to_str(new.method);
        let body_type_str = body_type_str(&new.body);
        let auth_type_str = auth_type_str(&new.auth);
        let headers_json = to_json(&new.headers)?;
        let query_json = to_json(&new.query)?;
        let auth_json = to_json(&new.auth)?;

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO requests
                 (id, collection_id, name, method, url, headers, query_params,
                  body_type, body_content, auth_type, auth_config,
                  sort_order, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                 0, ?12, ?12)",
                params![
                    id.to_string(),
                    new.collection_id.to_string(),
                    new.name,
                    method_str,
                    new.url,
                    headers_json,
                    query_json,
                    body_type_str,
                    body_content(&new.body),
                    auth_type_str,
                    auth_json,
                    now_ts,
                ],
            )?;
            Ok(RequestItem {
                id,
                collection_id: new.collection_id,
                name: new.name,
                method: new.method,
                url: new.url,
                headers: new.headers,
                query: new.query,
                body: new.body,
                auth: new.auth,
                sort_order: 0,
                created_at: now,
                updated_at: now,
            })
        })
    }

    /// 改名
    pub fn rename(&self, id: uuid::Uuid, new_name: String) -> Result<()> {
        if new_name.trim().is_empty() {
            return Err(Error::InvalidInput("request name 不能为空".into()));
        }
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE requests SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![new_name, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 更新 URL(最常用的修改,单独一个方法)
    pub fn update_url(&self, id: uuid::Uuid, new_url: String) -> Result<()> {
        if new_url.trim().is_empty() {
            return Err(Error::InvalidInput("request url 不能为空".into()));
        }
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE requests SET url = ?1, updated_at = ?2 WHERE id = ?3",
                params![new_url, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 更新 method(GET → POST 等)
    pub fn update_method(&self, id: uuid::Uuid, new_method: Method) -> Result<()> {
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        let method_str = method_to_str(new_method);
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE requests SET method = ?1, updated_at = ?2 WHERE id = ?3",
                params![method_str, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 更新 headers(JSON 存)
    pub fn update_headers(&self, id: uuid::Uuid, headers: Vec<KeyValue>) -> Result<()> {
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        let json = to_json(&headers)?;
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE requests SET headers = ?1, updated_at = ?2 WHERE id = ?3",
                params![json, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 更新 query params(JSON 存)
    pub fn update_query(&self, id: uuid::Uuid, query: Vec<KeyValue>) -> Result<()> {
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        let json = to_json(&query)?;
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE requests SET query_params = ?1, updated_at = ?2 WHERE id = ?3",
                params![json, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 更新 body
    pub fn update_body(&self, id: uuid::Uuid, body: Body) -> Result<()> {
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        let type_str = body_type_str(&body);
        let content = body_content(&body);
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE requests SET body_type = ?1, body_content = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![type_str, content, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 更新 auth(JSON 存,单字段包含 type + data)
    pub fn update_auth(&self, id: uuid::Uuid, auth: Auth) -> Result<()> {
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        let type_str = auth_type_str(&auth);
        let json = to_json(&auth)?;
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE requests SET auth_type = ?1, auth_config = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![type_str, json, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 全量更新(一条 SQL 更新所有字段,比调用 6 个单独 update 更高效)
    pub fn update_full(
        &self,
        id: uuid::Uuid,
        name: String,
        method: Method,
        url: String,
        headers: Vec<KeyValue>,
        query: Vec<KeyValue>,
        body: Body,
        auth: Auth,
    ) -> Result<()> {
        if name.trim().is_empty() {
            return Err(Error::InvalidInput("request name 不能为空".into()));
        }
        if url.trim().is_empty() {
            return Err(Error::InvalidInput("request url 不能为空".into()));
        }

        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        let method_str = method_to_str(method);
        let headers_json = to_json(&headers)?;
        let query_json = to_json(&query)?;
        let body_type_str = body_type_str(&body);
        let body_content_str = body_content(&body);
        let auth_type_str = auth_type_str(&auth);
        let auth_json = to_json(&auth)?;

        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE requests SET
                    name = ?1, method = ?2, url = ?3, headers = ?4, query_params = ?5,
                    body_type = ?6, body_content = ?7, auth_type = ?8, auth_config = ?9,
                    updated_at = ?10
                 WHERE id = ?11",
                params![
                    name,
                    method_str,
                    url,
                    headers_json,
                    query_json,
                    body_type_str,
                    body_content_str,
                    auth_type_str,
                    auth_json,
                    now_ts,
                    id_str,
                ],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 删除请求。
    ///
    /// **级联**:`history` 表的 `request_id` 是 `ON DELETE SET NULL`,所以历史保留
    /// (但 `request_id` 变 NULL,UI 显示"已删除的请求")。
    pub fn delete(&self, id: uuid::Uuid) -> Result<()> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "DELETE FROM requests WHERE id = ?1",
                params![id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("request {id}")));
            }
            Ok(())
        })
    }

    /// 搜索请求(在 `name` 和 `url` 里 LIKE,不区分大小写)。
    ///
    /// 性能:`%keyword%` 是全表扫描,数据量 < 10k 时无所谓。
    /// 未来量大可以加 FTS5 全文索引。
    pub fn search(&self, keyword: &str) -> Result<Vec<RequestItem>> {
        let pattern = format!("%{keyword}%");
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, collection_id, name, method, url, headers, query_params,
                        body_type, body_content, auth_type, auth_config,
                        sort_order, created_at, updated_at
                 FROM requests
                 WHERE LOWER(name) LIKE LOWER(?1) OR LOWER(url) LIKE LOWER(?1)
                 ORDER BY updated_at DESC
                 LIMIT 100",
            )?;
            let rows = stmt.query_map(params![pattern], row_to_request)?;
            rows.collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
                .map_err(Error::from)
        })
    }
}

// ─── 内部辅助:枚举 ↔ 字符串 / Body 内容提取 ─────────────────────────────

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

fn body_type_str(b: &Body) -> &'static str {
    match b {
        Body::None => "none",
        Body::Json { .. } => "json",
        Body::Form { .. } => "form",
        Body::Raw { .. } => "raw",
    }
}

/// 从 Body 里提取 "内容" 部分(配合 `body_type` 一起用)
fn body_content(b: &Body) -> Option<String> {
    match b {
        Body::None => None,
        Body::Json { content } | Body::Raw { content, .. } => Some(content.clone()),
        Body::Form { fields } => Some(serde_json::to_string(fields).unwrap_or_default()),
    }
}

fn body_from_parts(type_str: &str, content: Option<String>) -> rusqlite::Result<Body> {
    match type_str {
        "none" => Ok(Body::None),
        "json" => {
            let content = content.unwrap_or_default();
            Ok(Body::Json { content })
        }
        "form" => {
            let fields: Vec<KeyValue> = content
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            Ok(Body::Form { fields })
        }
        "raw" => {
            let content = content.unwrap_or_default();
            Ok(Body::Raw { content, content_type: "text/plain".into() })
        }
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown body_type '{other}'"),
            )),
        )),
    }
}

fn auth_type_str(a: &Auth) -> &'static str {
    match a {
        Auth::None => "none",
        Auth::Bearer { .. } => "bearer",
        Auth::Basic { .. } => "basic",
        Auth::ApiKey { .. } => "apikey",
    }
}

fn auth_from_str(type_str: &str, config_json: Option<&str>) -> rusqlite::Result<Auth> {
    let cfg = config_json.unwrap_or("{}");
    match type_str {
        "none" => Ok(Auth::None),
        "bearer" => serde_json::from_str::<Auth>(cfg).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        }),
        "basic" => serde_json::from_str::<Auth>(cfg).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        }),
        "apikey" => serde_json::from_str::<Auth>(cfg).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        }),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown auth_type '{other}'"),
            )),
        )),
    }
}

/// SQLite 行 → `RequestItem`
fn row_to_request(row: &Row<'_>) -> rusqlite::Result<RequestItem> {
    let id_str: String = row.get(0)?;
    let coll_id_str: String = row.get(1)?;
    let method_str: String = row.get(3)?;
    let headers_json: String = row.get(5)?;
    let query_json: String = row.get(6)?;
    let body_type: String = row.get(7)?;
    let body_content_str: Option<String> = row.get(8)?;
    let auth_type: String = row.get(9)?;
    let auth_config: Option<String> = row.get(10)?;
    let created_at_ts: i64 = row.get(12)?;
    let updated_at_ts: i64 = row.get(13)?;

    Ok(RequestItem {
        id: parse_uuid(&id_str).map_err(uuid_err)?,
        collection_id: parse_uuid(&coll_id_str).map_err(uuid_err)?,
        name: row.get(2)?,
        method: method_from_str(&method_str)?,
        url: row.get(4)?,
        headers: from_json(&headers_json).map_err(serde_err)?,
        query: from_json(&query_json).map_err(serde_err)?,
        body: body_from_parts(&body_type, body_content_str)?,
        auth: auth_from_str(&auth_type, auth_config.as_deref())?,
        sort_order: row.get(11)?,
        created_at: from_unix(created_at_ts),
        updated_at: from_unix(updated_at_ts),
    })
}

fn uuid_err(e: Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
    )
}

fn serde_err(e: Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
    )
}

// 静默 unused import 警告
#[allow(dead_code)]
fn _unused_parse(_s: Option<String>) {
    parse_uuid_opt(_s).ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_db() -> crate::storage::Database {
        crate::storage::Database::open_in_memory().unwrap()
    }

    fn setup() -> (crate::storage::Database, uuid::Uuid) {
        let db = fresh_db();
        let coll = db
            .collections()
            .create(crate::collection::NewCollection {
                name: "Test".into(),
                description: None,
                parent_id: None,
            })
            .unwrap();
        (db, coll.id)
    }

    fn new_req(coll_id: uuid::Uuid) -> NewRequest {
        NewRequest {
            collection_id: coll_id,
            name: "Get user".into(),
            method: Method::Get,
            url: "https://api.example.com/users/1".into(),
            headers: vec![KeyValue {
                key: "Accept".into(),
                value: "application/json".into(),
                enabled: true,
            }],
            query: vec![],
            body: Body::None,
            auth: Auth::None,
        }
    }

    #[test]
    fn test_create_and_find() {
        let (db, coll_id) = setup();
        let req = db.requests().create(new_req(coll_id)).unwrap();

        let found = db.requests().find_by_id(req.id).unwrap();
        assert_eq!(found.name, "Get user");
        assert!(matches!(found.method, Method::Get));
        assert_eq!(found.url, "https://api.example.com/users/1");
        assert_eq!(found.headers.len(), 1);
        assert!(matches!(found.body, Body::None));
    }

    #[test]
    fn test_create_empty_name_fails() {
        let (db, coll_id) = setup();
        let mut n = new_req(coll_id);
        n.name = "  ".into();
        let res = db.requests().create(n);
        assert!(matches!(res, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn test_list_by_collection() {
        let (db, coll_id) = setup();
        db.requests().create(new_req(coll_id)).unwrap();
        let mut n2 = new_req(coll_id);
        n2.name = "Create user".into();
        n2.method = Method::Post;
        db.requests().create(n2).unwrap();

        let list = db.requests().list_by_collection(coll_id).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Create user"); // sort_order=0,按 name 排
        assert_eq!(list[1].name, "Get user");
    }

    #[test]
    fn test_update_url_and_method() {
        let (db, coll_id) = setup();
        let req = db.requests().create(new_req(coll_id)).unwrap();

        db.requests().update_url(req.id, "https://api.example.com/v2/users".into()).unwrap();
        db.requests().update_method(req.id, Method::Post).unwrap();

        let found = db.requests().find_by_id(req.id).unwrap();
        assert_eq!(found.url, "https://api.example.com/v2/users");
        assert!(matches!(found.method, Method::Post));
    }

    #[test]
    fn test_update_body_json() {
        let (db, coll_id) = setup();
        let req = db.requests().create(new_req(coll_id)).unwrap();
        db.requests()
            .update_body(req.id, Body::Json { content: r#"{"name":"x"}"#.into() })
            .unwrap();

        let found = db.requests().find_by_id(req.id).unwrap();
        match found.body {
            Body::Json { content } => assert_eq!(content, r#"{"name":"x"}"#),
            _ => panic!("expected Json body"),
        }
    }

    #[test]
    fn test_update_auth_bearer() {
        let (db, coll_id) = setup();
        let req = db.requests().create(new_req(coll_id)).unwrap();
        db.requests()
            .update_auth(req.id, Auth::Bearer { token: "secret123".into() })
            .unwrap();

        let found = db.requests().find_by_id(req.id).unwrap();
        match found.auth {
            Auth::Bearer { token } => assert_eq!(token, "secret123"),
            _ => panic!("expected Bearer auth"),
        }
    }

    #[test]
    fn test_delete_cascades_history() {
        let (db, coll_id) = setup();
        let req = db.requests().create(new_req(coll_id)).unwrap();
        db.requests().delete(req.id).unwrap();
        assert!(matches!(
            db.requests().find_by_id(req.id),
            Err(Error::NotFound(_))
        ));
    }

    #[test]
    fn test_search() {
        let (db, coll_id) = setup();
        db.requests().create(new_req(coll_id)).unwrap();
        let mut n2 = new_req(coll_id);
        n2.name = "List orders".into();
        n2.url = "https://api.example.com/orders".into();
        db.requests().create(n2).unwrap();

        let results = db.requests().search("user").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Get user");

        let results = db.requests().search("orders").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "List orders");
    }

    #[test]
    fn test_update_full() {
        let (db, coll_id) = setup();
        let req = db.requests().create(new_req(coll_id)).unwrap();

        db.requests()
            .update_full(
                req.id,
                "Updated".into(),
                Method::Post,
                "https://api.example.com/v2/updated".into(),
                vec![KeyValue {
                    key: "X-Test".into(),
                    value: "yes".into(),
                    enabled: false,
                }],
                vec![KeyValue {
                    key: "page".into(),
                    value: "2".into(),
                    enabled: true,
                }],
                Body::Json {
                    content: r#"{"k":"v"}"#.into(),
                },
                Auth::Bearer {
                    token: "tok123".into(),
                },
            )
            .unwrap();

        let found = db.requests().find_by_id(req.id).unwrap();
        assert_eq!(found.name, "Updated");
        assert!(matches!(found.method, Method::Post));
        assert_eq!(found.url, "https://api.example.com/v2/updated");
        assert_eq!(found.headers.len(), 1);
        assert!(!found.headers[0].enabled);
        assert_eq!(found.query.len(), 1);
        assert!(matches!(found.body, Body::Json { .. }));
        assert!(matches!(found.auth, Auth::Bearer { .. }));
    }

    #[test]
    fn test_headers_roundtrip_complex() {
        let (db, coll_id) = setup();
        let mut n = new_req(coll_id);
        n.headers = vec![
            KeyValue { key: "X-API-Key".into(), value: "abc".into(), enabled: true },
            KeyValue { key: "Authorization".into(), value: "Bearer xyz".into(), enabled: false },
            KeyValue { key: "".into(), value: "should be skipped".into(), enabled: true },
        ];
        let req = db.requests().create(n).unwrap();
        let found = db.requests().find_by_id(req.id).unwrap();
        assert_eq!(found.headers.len(), 3);
        assert_eq!(found.headers[0].key, "X-API-Key");
        assert!(!found.headers[1].enabled);
    }
}