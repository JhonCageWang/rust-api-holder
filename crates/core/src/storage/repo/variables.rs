//! 环境变量 Repository
//!
//! 每个变量归属于某个 `Environment`,按 `key` 在前端展示。
//! 设计目标:
//! - 单个增删改(`create` / `update` / `delete`)
//! - **批量替换**(`bulk_replace`):前端一次编辑完整个表单,一次性保存,
//!   避免频繁的 add/remove 触发性能问题和 FK 抖动
//!
//! ## 唯一性
//!
//! `(environment_id, key)` 唯一索引 — 一个环境下 `key` 不能重复。
//! 创建重复 key 返回 `Error::Database` (UNIQUE 约束),调用方可以删除旧的再创建。

use rusqlite::{params, Row};

use crate::environment::Variable;
use crate::error::Error;
use crate::Result;

use super::{from_sqlite_bool, from_unix, parse_uuid, to_sqlite_bool, to_unix};

/// 环境变量 Repository
pub struct VariableRepo<'a> {
    db: &'a crate::storage::Database,
}

impl<'a> VariableRepo<'a> {
    pub(crate) fn new(db: &'a crate::storage::Database) -> Self {
        Self { db }
    }

    /// 列出某个环境下的所有变量,按 key 排序
    pub fn list_by_env(&self, env_id: uuid::Uuid) -> Result<Vec<Variable>> {
        let id_str = env_id.to_string();
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, environment_id, key, value, enabled
                 FROM variables WHERE environment_id = ?1 ORDER BY key ASC",
            )?;
            let rows = stmt.query_map(params![id_str], row_to_variable)?;
            rows.collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
                .map_err(Error::from)
        })
    }

    /// 创建单个变量。
    ///
    /// 同一 `environment_id` 下 `key` 重复会失败(UNIQUE 约束)。
    pub fn create(
        &self,
        env_id: uuid::Uuid,
        key: String,
        value: String,
    ) -> Result<Variable> {
        if key.trim().is_empty() {
            return Err(Error::InvalidInput("variable key 不能为空".into()));
        }
        let id = uuid::Uuid::new_v4();

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO variables (id, environment_id, key, value, enabled)
                 VALUES (?1, ?2, ?3, ?4, 1)",
                params![id.to_string(), env_id.to_string(), key, value],
            )?;
            Ok(Variable {
                id,
                environment_id: env_id,
                key,
                value,
                enabled: true,
            })
        })
    }

    /// 更新变量的 value 和 enabled 状态
    pub fn update(&self, id: uuid::Uuid, new_value: String, enabled: bool) -> Result<()> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE variables SET value = ?1, enabled = ?2 WHERE id = ?3",
                params![new_value, to_sqlite_bool(enabled), id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("variable {id}")));
            }
            Ok(())
        })
    }

    /// 删除变量
    pub fn delete(&self, id: uuid::Uuid) -> Result<()> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "DELETE FROM variables WHERE id = ?1",
                params![id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("variable {id}")));
            }
            Ok(())
        })
    }

    /// **批量替换**整个环境的变量(在一个事务里)。
    ///
    /// 工作流程:
    /// 1. DELETE 该环境下所有现有变量
    /// 2. INSERT 新的变量列表
    ///
    /// 在事务里,**前端一次编辑 = 一次 bulk_replace**,简单可靠。
    ///
    /// 注意:这个操作会清掉所有现有变量,前端要传"完整列表"。
    pub fn bulk_replace(&self, env_id: uuid::Uuid, vars: Vec<Variable>) -> Result<()> {
        let env_id_str = env_id.to_string();
        self.db.with_conn(|conn| {
            let tx = conn.transaction()?;

            // 1. 全删
            tx.execute(
                "DELETE FROM variables WHERE environment_id = ?1",
                params![env_id_str],
            )?;

            // 2. 逐个 insert
            for v in &vars {
                if v.key.trim().is_empty() {
                    return Err(Error::InvalidInput("variable key 不能为空".into()));
                }
                tx.execute(
                    "INSERT INTO variables (id, environment_id, key, value, enabled)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        v.id.to_string(),
                        env_id_str,
                        v.key,
                        v.value,
                        to_sqlite_bool(v.enabled),
                    ],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
    }

    /// 统计某个环境下的变量数(便于 UI 显示)
    pub fn count_by_env(&self, env_id: uuid::Uuid) -> Result<i64> {
        let id_str = env_id.to_string();
        self.db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM variables WHERE environment_id = ?1",
                params![id_str],
                |row| row.get(0),
            )?;
            Ok(count)
        })
    }
}

fn row_to_variable(row: &Row<'_>) -> rusqlite::Result<Variable> {
    let id_str: String = row.get(0)?;
    let env_id_str: String = row.get(1)?;
    let enabled: i64 = row.get(4)?;

    Ok(Variable {
        id: parse_uuid(&id_str).map_err(uuid_err)?,
        environment_id: parse_uuid(&env_id_str).map_err(uuid_err)?,
        key: row.get(2)?,
        value: row.get(3)?,
        enabled: from_sqlite_bool(enabled),
    })
}

fn uuid_err(e: Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
    )
}

// 静默 unused
#[allow(dead_code)]
fn _unused() {
    let _ = to_unix(chrono::Utc::now());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (crate::storage::Database, uuid::Uuid) {
        let db = crate::storage::Database::open_in_memory().unwrap();
        let env = db.environments().create("Dev".into()).unwrap();
        (db, env.id)
    }

    #[test]
    fn test_create_and_list() {
        let (db, env_id) = setup();
        let v = db.variables().create(env_id, "host".into(), "api.dev".into()).unwrap();
        assert_eq!(v.key, "host");
        assert!(v.enabled);

        let list = db.variables().list_by_env(env_id).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].value, "api.dev");
    }

    #[test]
    fn test_unique_key_per_env() {
        let (db, env_id) = setup();
        db.variables().create(env_id, "host".into(), "a".into()).unwrap();
        let res = db.variables().create(env_id, "host".into(), "b".into());
        assert!(matches!(res, Err(Error::Database(_))));
    }

    #[test]
    fn test_same_key_different_envs() {
        let db = crate::storage::Database::open_in_memory().unwrap();
        let e1 = db.environments().create("Dev".into()).unwrap();
        let e2 = db.environments().create("Prod".into()).unwrap();
        // 不同环境下可以同名
        db.variables().create(e1.id, "host".into(), "a".into()).unwrap();
        db.variables().create(e2.id, "host".into(), "b".into()).unwrap();
    }

    #[test]
    fn test_empty_key_fails() {
        let (db, env_id) = setup();
        let res = db.variables().create(env_id, "  ".into(), "x".into());
        assert!(matches!(res, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn test_update() {
        let (db, env_id) = setup();
        let v = db.variables().create(env_id, "k".into(), "old".into()).unwrap();
        db.variables().update(v.id, "new".into(), false).unwrap();
        let found = db.variables().list_by_env(env_id).unwrap();
        assert_eq!(found[0].value, "new");
        assert!(!found[0].enabled);
    }

    #[test]
    fn test_delete() {
        let (db, env_id) = setup();
        let v = db.variables().create(env_id, "k".into(), "v".into()).unwrap();
        db.variables().delete(v.id).unwrap();
        assert_eq!(db.variables().list_by_env(env_id).unwrap().len(), 0);
    }

    #[test]
    fn test_bulk_replace() {
        let (db, env_id) = setup();
        // 先建一些
        db.variables().create(env_id, "old1".into(), "v1".into()).unwrap();
        db.variables().create(env_id, "old2".into(), "v2".into()).unwrap();

        // 整体替换
        let new_vars = vec![
            Variable {
                id: uuid::Uuid::new_v4(),
                environment_id: env_id,
                key: "new_a".into(),
                value: "x".into(),
                enabled: true,
            },
            Variable {
                id: uuid::Uuid::new_v4(),
                environment_id: env_id,
                key: "new_b".into(),
                value: "y".into(),
                enabled: false,
            },
        ];
        db.variables().bulk_replace(env_id, new_vars).unwrap();

        let list = db.variables().list_by_env(env_id).unwrap();
        assert_eq!(list.len(), 2);
        let keys: Vec<_> = list.iter().map(|v| v.key.as_str()).collect();
        assert_eq!(keys, vec!["new_a", "new_b"]);
        assert!(list[0].enabled);
        assert!(!list[1].enabled);
    }
}