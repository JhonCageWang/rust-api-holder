//! SQLite 持久化层(Repository 模式)
//!
//! ## 架构
//!
//! - [`Database`] — 入口,持有 rusqlite 连接(已包 `Mutex` 保证线程安全)
//! - [`repo`]    — 各实体的 Repository,通过 [`Database`] 上的便捷方法访问
//!
//! ## 设计原则
//!
//! 1. **业务层通过 Repository 访问数据,不直接写 SQL**(隔离)
//! 2. **每个 Repository 是独立 struct,持有 `&Database` 借用**(零成本)
//! 3. **所有方法返回 `Result<T>`**,底层 `rusqlite::Error` 自动转 [`Error::Database`]
//! 4. **业务错误显式构造**(如 [`Error::NotFound`]),不滥用 `Other`
//!
//! ## 用法示例
//!
//! ```no_run
//! use api_holder_core::storage::Database;
//! use api_holder_core::collection::NewCollection;
//!
//! let db = Database::open("app.db")?;
//!
//! // 链式访问 Repository
//! let coll = db.collections().create(NewCollection {
//!     name: "My API".into(),
//!     description: None,
//!     parent_id: None,
//! })?;
//!
//! let reqs = db.requests().list_by_collection(coll.id)?;
//! # Ok::<(), api_holder_core::Error>(())
//! ```
//!
//! ## 线程安全
//!
//! - [`Database`] 内部是 `Mutex<Connection>`(rusqlite 的 Connection **不是** `Send` 安全的)
//! - `Database::with_conn` 是借用 connection 的唯一途径,确保锁释放
//! - Repository 通过 `&Database` 工作,无需自己管锁
//!
//! ## 测试
//!
//! 每个 Repository 文件末尾都有 `#[cfg(test)] mod tests`,
//! 用 [`Database::open_in_memory`] 跑 SQLite 内存数据库,无需真实文件。

#![allow(dead_code)]

pub mod migrations;
pub mod repo;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

use crate::Result;

/// 数据库封装。
///
/// 内部用 `Mutex<Connection>` 保护 SQLite 连接(rusqlite 本身不是线程安全的)。
/// 应用启动时构造一次,放进全局状态(参见 `crates/app/src/main.rs` 的 `AppState`),
/// 整个生命周期共享。
pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    /// 打开(或创建)数据库文件。
    ///
    /// 会自动:
    /// 1. 创建父目录(`std::fs::create_dir_all`)
    /// 2. 应用所有未跑的 schema migration
    ///
    /// ## 错误
    ///
    /// - 父目录无法创建 → `Error::Io`
    /// - 文件无法打开(权限/磁盘满) → `Error::Database`
    /// - Migration 失败 → `Error::Database`
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        migrations::migrate(&conn)?;
        Ok(Self { conn: Mutex::new(conn), path: path.to_path_buf() })
    }

    /// 在内存中打开数据库(用于测试)。
    ///
    /// `:memory:` 数据库不持久化,进程退出就丢,但生命周期内功能完整。
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        migrations::migrate(&conn)?;
        Ok(Self { conn: Mutex::new(conn), path: "<in-memory>".into() })
    }

    /// 在闭包里借用 connection。
    ///
    /// 这是 Repository 访问 connection 的**唯一**途径:
    /// - 自动加锁 / 解锁(RAII 风格的 `MutexGuard`)
    /// - 闭包返回 `Result<R>`,错误自动透传
    /// - 锁持有范围 = 闭包执行期间,绝不会跨 await(因为是同步)
    ///
    /// `pub(crate)` 防止业务层绕开 Repository 直接写 SQL。
    pub(crate) fn with_conn<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Connection) -> Result<R>,
    {
        let mut guard = self.conn.lock().expect("database mutex poisoned");
        f(&mut guard)
    }

    /// 数据库文件路径。内存数据库返回 `"<in-memory>"`。
    pub fn path(&self) -> &Path {
        &self.path
    }

    // ─── Repository 便捷访问 ───────────────────────────────────────
    //
    // 用法:`db.collections().create(...)`
    //
    // 返回的是借用 `&self` 的 Repository,生命周期由 Database 控制。
    // 不能跨线程发送(因为 `&self` 不是 Send),但够用 —
    // 整个应用就一个 Database,Repository 操作都是同步的。

    /// 获取集合(文件夹) Repository
    pub fn collections(&self) -> repo::CollectionRepo<'_> {
        repo::CollectionRepo::new(self)
    }

    /// 获取请求 Repository
    pub fn requests(&self) -> repo::RequestRepo<'_> {
        repo::RequestRepo::new(self)
    }

    /// 获取环境 Repository
    pub fn environments(&self) -> repo::EnvironmentRepo<'_> {
        repo::EnvironmentRepo::new(self)
    }

    /// 获取环境变量 Repository
    pub fn variables(&self) -> repo::VariableRepo<'_> {
        repo::VariableRepo::new(self)
    }

    /// 获取历史记录 Repository
    pub fn history(&self) -> repo::HistoryRepo<'_> {
        repo::HistoryRepo::new(self)
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