//! 环境 Repository
//!
//! ## 模型
//!
//! 环境是 `{{variable}}` 插值的命名容器(比如 "Dev" / "Prod")。
//! **全局只有一个 `is_active = true`** — 切换时需要原子操作。
//!
//! ## 原子切换
//!
//! [`EnvironmentRepo::set_active`] 在一个事务里:
//! 1. 把所有 `is_active` 清零
//! 2. 把目标 `is_active` 置 1
//!
//! 避免出现"两个 active"或"零个 active"的中间态。

use rusqlite::{params, Row};

use crate::environment::Environment;
use crate::error::Error;
use crate::Result;

use super::{from_sqlite_bool, from_unix, parse_uuid, to_sqlite_bool, to_unix};

/// 环境 Repository
pub struct EnvironmentRepo<'a> {
    db: &'a crate::storage::Database,
}

impl<'a> EnvironmentRepo<'a> {
    pub(crate) fn new(db: &'a crate::storage::Database) -> Self {
        Self { db }
    }

    /// 列出所有环境,按 name 排序
    pub fn list_all(&self) -> Result<Vec<Environment>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, is_active, created_at, updated_at
                 FROM environments ORDER BY name ASC",
            )?;
            let rows = stmt.query_map([], row_to_environment)?;
            rows.collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
                .map_err(Error::from)
        })
    }

    /// 找单个环境
    pub fn find_by_id(&self, id: uuid::Uuid) -> Result<Environment> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, name, is_active, created_at, updated_at
                 FROM environments WHERE id = ?1",
                params![id_str],
                row_to_environment,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("environment {id}")),
                other => Error::Database(other),
            })
        })
    }

    /// 找当前激活的环境。**没有 active 时返回 NotFound**,不是 Ok(None)。
    pub fn find_active(&self) -> Result<Environment> {
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, name, is_active, created_at, updated_at
                 FROM environments WHERE is_active = 1 LIMIT 1",
                [],
                row_to_environment,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    Error::NotFound("no active environment".into())
                }
                other => Error::Database(other),
            })
        })
    }

    /// 创建新环境
    ///
    /// 默认 `is_active = false`。如果要创建后立即激活,用 [`set_active`](Self::set_active)。
    pub fn create(&self, name: String) -> Result<Environment> {
        if name.trim().is_empty() {
            return Err(Error::InvalidInput("environment name 不能为空".into()));
        }
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();
        let now_ts = to_unix(now);

        self.db.with_conn(|conn| {
            // UNIQUE(name) 约束,name 重复会报错
            conn.execute(
                "INSERT INTO environments (id, name, is_active, created_at, updated_at)
                 VALUES (?1, ?2, 0, ?3, ?3)",
                params![id.to_string(), name, now_ts],
            )?;
            Ok(Environment {
                id,
                name,
                is_active: false,
                created_at: now,
                updated_at: now,
            })
        })
    }

    /// 改名
    pub fn rename(&self, id: uuid::Uuid, new_name: String) -> Result<()> {
        if new_name.trim().is_empty() {
            return Err(Error::InvalidInput("environment name 不能为空".into()));
        }
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE environments SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![new_name, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("environment {id}")));
            }
            Ok(())
        })
    }

    /// **原子**切换激活环境
    ///
    /// 在事务里:
    /// 1. `UPDATE environments SET is_active = 0`(全部清零)
    /// 2. `UPDATE environments SET is_active = 1 WHERE id = ?`(设置目标)
    ///
    /// 如果目标 ID 不存在,事务回滚,所有 active 状态保持原样。
    pub fn set_active(&self, id: uuid::Uuid) -> Result<()> {
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        self.db.with_conn(|conn| {
            let tx = conn.transaction()?;
            // 1. 先全部清零
            tx.execute(
                "UPDATE environments SET is_active = 0, updated_at = ?1",
                params![now_ts],
            )?;
            // 2. 设置目标
            let changed = tx.execute(
                "UPDATE environments SET is_active = 1, updated_at = ?1 WHERE id = ?2",
                params![now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("environment {id}")));
            }
            tx.commit()?;
            Ok(())
        })
    }

    /// 删除环境。
    ///
    /// **级联**:`variables` 表有 `ON DELETE CASCADE`,环境里的变量会一起删。
    /// 历史记录不受影响(不引用 environment_id)。
    pub fn delete(&self, id: uuid::Uuid) -> Result<()> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "DELETE FROM environments WHERE id = ?1",
                params![id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("environment {id}")));
            }
            Ok(())
        })
    }
}

fn row_to_environment(row: &Row<'_>) -> rusqlite::Result<Environment> {
    let id_str: String = row.get(0)?;
    let is_active: i64 = row.get(2)?;
    let created_at_ts: i64 = row.get(3)?;
    let updated_at_ts: i64 = row.get(4)?;

    Ok(Environment {
        id: parse_uuid(&id_str).map_err(uuid_err)?,
        name: row.get(1)?,
        is_active: from_sqlite_bool(is_active),
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

// 静默 unused import 警告
#[allow(dead_code)]
fn _unused() {
    let _ = to_sqlite_bool(true);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_db() -> crate::storage::Database {
        crate::storage::Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_create_and_find() {
        let db = fresh_db();
        let env = db.environments().create("Dev".into()).unwrap();
        assert!(!env.is_active);

        let found = db.environments().find_by_id(env.id).unwrap();
        assert_eq!(found.name, "Dev");
    }

    #[test]
    fn test_unique_name() {
        let db = fresh_db();
        db.environments().create("Dev".into()).unwrap();
        let res = db.environments().create("Dev".into());
        assert!(matches!(res, Err(Error::Database(_))));
    }

    #[test]
    fn test_set_active_is_atomic() {
        let db = fresh_db();
        let e1 = db.environments().create("Dev".into()).unwrap();
        let e2 = db.environments().create("Prod".into()).unwrap();
        let e3 = db.environments().create("Staging".into()).unwrap();

        // 先激活 e1
        db.environments().set_active(e1.id).unwrap();
        assert!(db.environments().find_by_id(e1.id).unwrap().is_active);

        // 切到 e2,e1 应该自动失活
        db.environments().set_active(e2.id).unwrap();
        assert!(!db.environments().find_by_id(e1.id).unwrap().is_active);
        assert!(db.environments().find_by_id(e2.id).unwrap().is_active);
        assert!(!db.environments().find_by_id(e3.id).unwrap().is_active);

        // find_active 应该返回 e2
        assert_eq!(db.environments().find_active().unwrap().id, e2.id);
    }

    #[test]
    fn test_set_active_not_found_keeps_state() {
        let db = fresh_db();
        let e1 = db.environments().create("Dev".into()).unwrap();
        db.environments().set_active(e1.id).unwrap();

        // 切到一个不存在的 id
        let res = db.environments().set_active(uuid::Uuid::new_v4());
        assert!(matches!(res, Err(Error::NotFound(_))));

        // e1 仍然 active(事务回滚)
        assert!(db.environments().find_by_id(e1.id).unwrap().is_active);
    }

    #[test]
    fn test_delete_cascades_variables() {
        let db = fresh_db();
        let env = db.environments().create("Dev".into()).unwrap();
        db.variables().create(env.id, "host".into(), "api.dev".into()).unwrap();
        db.variables().create(env.id, "token".into(), "xxx".into()).unwrap();

        db.environments().delete(env.id).unwrap();
        assert_eq!(db.variables().list_by_env(env.id).unwrap().len(), 0);
    }
}