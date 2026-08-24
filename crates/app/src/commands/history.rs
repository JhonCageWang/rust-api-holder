//! 历史记录(History)相关的 Tauri Commands
//!
//! 注意:目前没有 `record_history` 命令 — 历史记录是在
//! `execute_request` 内部自动写入的(Week 4+ 接进来)。

use tauri::State;
use uuid::Uuid;

use api_holder_core::history::HistoryEntry;

use crate::AppState;

fn parse_id(s: &str) -> Result<Uuid, String> {
    Uuid::parse_str(s).map_err(|e| format!("invalid uuid '{s}': {e}"))
}

/// 分页列出历史
#[tauri::command]
pub async fn list_history(
    state: State<'_, AppState>,
    limit: i64,
    offset: i64,
) -> Result<Vec<HistoryEntry>, String> {
    state
        .db
        .history()
        .list(limit, offset)
        .map_err(|e| e.to_string())
}

/// 删除单条
#[tauri::command]
pub async fn delete_history(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state
        .db
        .history()
        .delete(parse_id(&id)?)
        .map_err(|e| e.to_string())
}

/// 清理 N 天前的历史(返回删除条数)
#[tauri::command]
pub async fn delete_old_history(
    state: State<'_, AppState>,
    days: i64,
) -> Result<usize, String> {
    state
        .db
        .history()
        .delete_older_than(days)
        .map_err(|e| e.to_string())
}

/// 历史总数
#[tauri::command]
pub async fn count_history(state: State<'_, AppState>) -> Result<i64, String> {
    state.db.history().count().map_err(|e| e.to_string())
}