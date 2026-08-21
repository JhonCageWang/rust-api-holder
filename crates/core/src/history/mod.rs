//! 请求历史记录
//!
//! 每次发送请求(无论成功失败)都会写入一条 history 记录。
//! 原始请求被修改/删除不影响历史(快照语义)。
//!
//! TODO(Week 6): 完整的自动入库 + 查询 + UI 回填

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::http::{KeyValue, Request, Response};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    /// 关联的原始请求 ID,可空(原始请求被删了历史还在)
    pub request_id: Option<Uuid>,
    /// 完整请求快照(用于"再发一次"功能)
    pub request_snapshot: Request,
    /// 响应快照
    pub response: Option<Response>,
    /// 网络错误信息(成功响应则为 None)
    pub error: Option<String>,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry_creation() {
        let entry = HistoryEntry {
            id: Uuid::new_v4(),
            request_id: None,
            request_snapshot: Request {
                method: crate::http::Method::Get,
                url: "https://example.com".into(),
                headers: vec![],
                query: vec![],
                body: Default::default(),
                auth: Default::default(),
            },
            response: None,
            error: None,
            sent_at: chrono::Utc::now(),
        };
        assert!(entry.response.is_none());
        assert!(entry.error.is_none());
    }
}