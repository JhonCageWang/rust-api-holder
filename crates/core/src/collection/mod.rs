//! 集合 / 请求 模型
//!
//! 集合(Collection)是请求(Request)的分组,支持嵌套(文件夹)。
//! 数据模型对应 SQLite 表 `collections` 和 `requests`。
//!
//! TODO(Week 3): 完整的 CRUD 操作 + Repository 实现

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::http::{Auth, Body, Method};

/// 集合(可作为文件夹)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 请求(归属于某个 Collection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestItem {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub name: String,
    pub method: Method,
    pub url: String,
    pub headers: Vec<super::http::KeyValue>,
    pub query: Vec<super::http::KeyValue>,
    pub body: Body,
    pub auth: Auth,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 创建新集合时的输入参数
#[derive(Debug, Clone)]
pub struct NewCollection {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_uuid() {
        let c = Collection {
            id: Uuid::new_v4(),
            name: "Test".into(),
            description: None,
            parent_id: None,
            sort_order: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(c.name, "Test");
    }
}