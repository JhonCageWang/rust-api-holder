//! 数据库 schema 迁移
//!
//! 每次 schema 变化时:
//! 1. 在 [`MIGRATIONS`] 里追加新版本 SQL
//! 2. 用 `version` 字段标记顺序
//! 3. 已应用的版本记录在 `_migrations` 表
//!
//! TODO(Week 3): 实装真正的迁移逻辑(目前只是把所有 v1 SQL 一次性跑)

#![allow(dead_code)]

use rusqlite::Connection;

use crate::Result;

/// 单个迁移版本
pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub sql: &'static str,
}

/// 内置迁移列表(按 version 升序)
pub static MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "initial schema",
    sql: INITIAL_SCHEMA,
}];

const INITIAL_SCHEMA: &str = r#"
-- 迁移记录表
CREATE TABLE IF NOT EXISTS _migrations (
    version    INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);

-- 集合
CREATE TABLE IF NOT EXISTS collections (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    parent_id   TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

-- 请求
CREATE TABLE IF NOT EXISTS requests (
    id            TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL,
    name          TEXT NOT NULL,
    method        TEXT NOT NULL,
    url           TEXT NOT NULL,
    headers       TEXT NOT NULL DEFAULT '[]',
    query_params  TEXT NOT NULL DEFAULT '[]',
    body_type     TEXT NOT NULL DEFAULT 'none',
    body_content  TEXT,
    auth_type     TEXT NOT NULL DEFAULT 'none',
    auth_config   TEXT,
    sort_order    INTEGER NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);

-- 环境
CREATE TABLE IF NOT EXISTS environments (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    is_active  INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 环境变量
CREATE TABLE IF NOT EXISTS variables (
    id             TEXT PRIMARY KEY,
    environment_id TEXT NOT NULL,
    key            TEXT NOT NULL,
    value          TEXT NOT NULL,
    enabled        INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (environment_id) REFERENCES environments(id) ON DELETE CASCADE,
    UNIQUE (environment_id, key)
);

-- 历史记录
CREATE TABLE IF NOT EXISTS history (
    id               TEXT PRIMARY KEY,
    request_id       TEXT,
    method           TEXT NOT NULL,
    url              TEXT NOT NULL,
    request_snapshot TEXT NOT NULL,
    status_code      INTEGER,
    response_headers TEXT,
    response_body    TEXT,
    duration_ms      INTEGER,
    error            TEXT,
    sent_at          INTEGER NOT NULL,
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_history_sent_at ON history(sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_requests_collection ON requests(collection_id);
CREATE INDEX IF NOT EXISTS idx_variables_env ON variables(environment_id);
"#;

/// 应用所有未应用的迁移
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version    INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )?;

    // 简化版 — 暂时只跑最新 schema(未来改成逐版本应用)
    for migration in MIGRATIONS {
        let already_applied: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations WHERE version = ?1",
                rusqlite::params![migration.version],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)?;

        if !already_applied {
            conn.execute_batch(migration.sql)?;
            conn.execute(
                "INSERT INTO _migrations (version, applied_at) VALUES (?1, ?2)",
                rusqlite::params![migration.version, chrono::Utc::now().timestamp()],
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        // 验证表是否真的创建了
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='collections'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migrate_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // 跑两次不应报错
    }
}
