//! SQLite 持久化层
//!
//! 架构:Repository 模式
//! - [`Database`] 是入口,持有连接 + 路径
//! - [`repo`] 子模块放各个实体的 Repository
//! - 业务层通过 Repository 操作数据,不直接写 SQL
//!
//! TODO(Week 3): 完整的 migration + CRUD 实现

#![allow(dead_code)]

pub mod migrations;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

use crate::Result;

/// 数据库封装,内部用 Mutex 保证线程安全
pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    /// 打开(或创建)数据库文件
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        // 应用 schema migration
        migrations::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: path.to_path_buf(),
        })
    }

    /// 在内存中打开(用于测试)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        migrations::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: "<in-memory>".into(),
        })
    }

    /// 获取内部连接(供 Repository 用)
    pub(crate) fn with_conn<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Connection) -> Result<R>,
    {
        let mut guard = self.conn.lock().expect("database mutex poisoned");
        f(&mut guard)
    }

    /// 数据库文件路径
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(db.path().to_str().unwrap(), "<in-memory>");
    }

    #[test]
    fn test_open_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("subdir").join("test.db");
        let _db = Database::open(&db_path).unwrap();
        assert!(db_path.exists());
    }
}
