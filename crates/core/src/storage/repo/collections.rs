//! 集合(文件夹) Repository
//!
//! ## 模型
//!
//! 集合是请求的分组容器,支持嵌套(用 `parent_id` 实现)。
//! 顶层集合 `parent_id = NULL`,子集合 `parent_id = 父集合.id`。
//!
//! ## 级联删除
//!
//! `requests` 表定义了 `FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE`,
//! 所以删除集合会自动删除里面的所有请求。**变量、历史记录不受影响**。
//!
//! ## 用法
//!
//! ```no_run
//! # use api_holder_core::storage::Database;
//! # use api_holder_core::collection::NewCollection;
//! # let db = Database::open("app.db").unwrap();
//! let coll = db.collections().create(NewCollection {
//!     name: "My API".into(),
//!     description: Some("用户管理接口".into()),
//!     parent_id: None, // 顶层
//! }).unwrap();
//!
//! // 找
//! let found = db.collections().find_by_id(coll.id).unwrap();
//!
//! // 改名字
//! db.collections().rename(coll.id, "User API".into()).unwrap();
//!
//! // 删(级联删 requests)
//! db.collections().delete(coll.id).unwrap();
//! ```

use rusqlite::{params, Row};

use crate::collection::{Collection, NewCollection};
use crate::error::Error;
use crate::Result;

use super::{from_unix, parse_uuid, parse_uuid_opt, to_unix};

/// 集合 Repository。借用 `&Database`,生命周期由调用者管。
pub struct CollectionRepo<'a> {
    db: &'a crate::storage::Database,
}

impl<'a> CollectionRepo<'a> {
    pub(crate) fn new(db: &'a crate::storage::Database) -> Self {
        Self { db }
    }

    /// 列出所有集合,按 `sort_order` → `name` 排序。
    ///
    /// 返回扁平列表(不构建树),UI 层自己组装。
    pub fn list_all(&self) -> Result<Vec<Collection>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, parent_id, sort_order, created_at, updated_at
                 FROM collections
                 ORDER BY sort_order ASC, name ASC",
            )?;
            let rows = stmt.query_map([], row_to_collection)?;
            rows.collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
                .map_err(Error::from)
        })
    }

    /// 根据 ID 查找集合。找不到返回 `Error::NotFound`。
    pub fn find_by_id(&self, id: uuid::Uuid) -> Result<Collection> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, name, description, parent_id, sort_order, created_at, updated_at
                 FROM collections WHERE id = ?1",
                params![id_str],
                row_to_collection,
            )
            .map_err(map_query_error("collection", id))
        })
    }

    /// 创建新集合。
    ///
    /// `sort_order` 默认 0(放最前),后续可以批量调整。
    /// `created_at` / `updated_at` 自动设置为当前时间。
    pub fn create(&self, new: NewCollection) -> Result<Collection> {
        if new.name.trim().is_empty() {
            return Err(Error::InvalidInput("collection name 不能为空".into()));
        }
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();
        let id_str = id.to_string();
        let now_ts = to_unix(now);
        let parent_str = new.parent_id.map(|u| u.to_string());

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO collections
                 (id, name, description, parent_id, sort_order, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5)",
                params![id_str, new.name, new.description, parent_str, now_ts],
            )?;
            Ok(Collection {
                id,
                name: new.name,
                description: new.description,
                parent_id: new.parent_id,
                sort_order: 0,
                created_at: now,
                updated_at: now,
            })
        })
    }

    /// 重命名集合。**只改 name,不影响 updated_at** —
    /// 改完应该再调一次 touch,或者直接在 UI 层合并。
    ///
    /// 实际上我们让 update 自动刷新 updated_at,更符合"改了东西就更新"惯例。
    pub fn rename(&self, id: uuid::Uuid, new_name: String) -> Result<()> {
        if new_name.trim().is_empty() {
            return Err(Error::InvalidInput("collection name 不能为空".into()));
        }
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE collections SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![new_name, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("collection {id}")));
            }
            Ok(())
        })
    }

    /// 更新集合信息(目前只支持 description 和 sort_order)。
    ///
    /// name 用 [`rename`](Self::rename),parent_id 修改比较复杂
    /// (涉及子树循环检测),先不做。
    pub fn update_meta(
        &self,
        id: uuid::Uuid,
        description: Option<String>,
        sort_order: i32,
    ) -> Result<()> {
        let id_str = id.to_string();
        let now_ts = to_unix(chrono::Utc::now());
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "UPDATE collections SET description = ?1, sort_order = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![description, sort_order, now_ts, id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("collection {id}")));
            }
            Ok(())
        })
    }

    /// 删除集合。
    ///
    /// **级联**:数据库 schema 里 `requests` 对 `collections` 有 `ON DELETE CASCADE`,
    /// 所以里面的请求会自动删除。
    ///
    /// 返回值是被删除的行数(0 表示 ID 不存在)。
    pub fn delete(&self, id: uuid::Uuid) -> Result<()> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            let changed = conn.execute(
                "DELETE FROM collections WHERE id = ?1",
                params![id_str],
            )?;
            if changed == 0 {
                return Err(Error::NotFound(format!("collection {id}")));
            }
            Ok(())
        })
    }

    /// 统计集合里的请求数量(便于 UI 显示 "(N requests)")。
    pub fn count_requests(&self, id: uuid::Uuid) -> Result<i64> {
        let id_str = id.to_string();
        self.db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM requests WHERE collection_id = ?1",
                params![id_str],
                |row| row.get(0),
            )?;
            Ok(count)
        })
    }
}

/// 把 SQLite 行映射成 `Collection` struct。
///
/// 列顺序对应 [`CollectionRepo::list_all`] 的 `SELECT`。
fn row_to_collection(row: &Row<'_>) -> rusqlite::Result<Collection> {
    let id_str: String = row.get(0)?;
    let parent_str: Option<String> = row.get(3)?;
    let created_at_ts: i64 = row.get(5)?;
    let updated_at_ts: i64 = row.get(6)?;

    Ok(Collection {
        id: parse_uuid(&id_str).map_err(sqlite_uuid_error)?,
        name: row.get(1)?,
        description: row.get(2)?,
        parent_id: parse_uuid_opt(parent_str).map_err(sqlite_uuid_opt_error)?,
        sort_order: row.get(4)?,
        created_at: from_unix(created_at_ts),
        updated_at: from_unix(updated_at_ts),
    })
}

/// 把 `query_row` 的 `QueryReturnedNoRows` 转成 `Error::NotFound`,其他原样透传。
fn map_query_error(entity: &'static str, id: uuid::Uuid) -> impl FnOnce(rusqlite::Error) -> Error {
    move |e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("{entity} {id}")),
        other => Error::Database(other),
    }
}

/// UUID 解析失败 → 包装成 rusqlite 错误(避免污染 Result 类型)
fn sqlite_uuid_error(e: crate::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
    )
}

fn sqlite_uuid_opt_error(e: crate::Error) -> rusqlite::Error {
    sqlite_uuid_error(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 每次测试拿全新内存 DB
    fn fresh_db() -> crate::storage::Database {
        crate::storage::Database::open_in_memory().unwrap()
    }

    fn nc(name: &str) -> NewCollection {
        NewCollection { name: name.into(), description: None, parent_id: None }
    }

    #[test]
    fn test_create_and_find() {
        let db = fresh_db();
        let coll = db
            .collections()
            .create(NewCollection {
                name: "My API".into(),
                description: Some("用户管理".into()),
                parent_id: None,
            })
            .unwrap();

        let found = db.collections().find_by_id(coll.id).unwrap();
        assert_eq!(found.name, "My API");
        assert_eq!(found.description.as_deref(), Some("用户管理"));
        assert!(found.parent_id.is_none());
        assert_eq!(found.sort_order, 0);
    }

    #[test]
    fn test_create_empty_name_fails() {
        let db = fresh_db();
        let res = db.collections().create(nc(""));
        assert!(matches!(res, Err(Error::InvalidInput(_))));
    }

    #[test]
    fn test_list_all_empty() {
        let db = fresh_db();
        assert!(db.collections().list_all().unwrap().is_empty());
    }

    #[test]
    fn test_list_all_order() {
        let db = fresh_db();
        // sort_order 都是 0,所以按 name 排序
        db.collections().create(nc("Bravo")).unwrap();
        db.collections().create(nc("Alpha")).unwrap();
        db.collections().create(nc("Charlie")).unwrap();

        let all = db.collections().list_all().unwrap();
        let names: Vec<_> = all.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["Alpha", "Bravo", "Charlie"]);
    }

    #[test]
    fn test_find_not_found() {
        let db = fresh_db();
        let res = db.collections().find_by_id(uuid::Uuid::new_v4());
        assert!(matches!(res, Err(Error::NotFound(_))));
    }

    #[test]
    fn test_rename() {
        let db = fresh_db();
        let coll = db.collections().create(nc("old")).unwrap();
        db.collections().rename(coll.id, "new".into()).unwrap();
        assert_eq!(db.collections().find_by_id(coll.id).unwrap().name, "new");
    }

    #[test]
    fn test_rename_empty_fails() {
        let db = fresh_db();
        let coll = db.collections().create(nc("x")).unwrap();
        let res = db.collections().rename(coll.id, "  ".into());
        assert!(matches!(res, Err(Error::InvalidInput(_))));
        // 名字没改
        assert_eq!(db.collections().find_by_id(coll.id).unwrap().name, "x");
    }

    #[test]
    fn test_update_meta() {
        let db = fresh_db();
        let coll = db
            .collections()
            .create(NewCollection {
                name: "x".into(),
                description: Some("a".into()),
                parent_id: None,
            })
            .unwrap();
        db.collections().update_meta(coll.id, Some("b".into()), 5).unwrap();
        let found = db.collections().find_by_id(coll.id).unwrap();
        assert_eq!(found.description.as_deref(), Some("b"));
        assert_eq!(found.sort_order, 5);
    }

    #[test]
    fn test_delete() {
        let db = fresh_db();
        let coll = db.collections().create(nc("x")).unwrap();
        db.collections().delete(coll.id).unwrap();
        assert!(matches!(
            db.collections().find_by_id(coll.id),
            Err(Error::NotFound(_))
        ));
    }

    #[test]
    fn test_delete_not_found() {
        let db = fresh_db();
        let res = db.collections().delete(uuid::Uuid::new_v4());
        assert!(matches!(res, Err(Error::NotFound(_))));
    }

    #[test]
    fn test_count_requests() {
        let db = fresh_db();
        let coll = db.collections().create(nc("x")).unwrap();
        assert_eq!(db.collections().count_requests(coll.id).unwrap(), 0);
        // 添加请求的测试在 requests repo 里
    }
}