//! 集合(Collection)相关的 Tauri Commands
//!
//! 把 [`CollectionRepo`] 的方法暴露给前端。所有命令返回 `Result<T, String>`,
//! 错误用 `e.to_string()` 转字符串(前端 `useInvoke` 会包成 Error 对象)。

use tauri::State;
use uuid::Uuid;

use api_holder_core::collection::{Collection, NewCollection};

use crate::AppState;

/// 列出所有集合
#[tauri::command]
pub async fn list_collections(
    state: State<'_, AppState>,
) -> Result<Vec<Collection>, String> {
    state.db.collections().list_all().map_err(|e| e.to_string())
}

/// 创建集合
#[tauri::command]
pub async fn create_collection(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    parent_id: Option<String>,
) -> Result<Collection, String> {
    let parent_id = parent_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| e.to_string())?;
    let new = NewCollection { name, description, parent_id };
    state.db.collections().create(new).map_err(|e| e.to_string())
}

/// 重命名集合
#[tauri::command]
pub async fn rename_collection(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db.collections().rename(id, new_name).map_err(|e| e.to_string())
}

/// 删除集合(级联删里面的 requests)
#[tauri::command]
pub async fn delete_collection(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db.collections().delete(id).map_err(|e| e.to_string())
}

/// 统计集合里的请求数量
#[tauri::command]
pub async fn count_collection_requests(
    state: State<'_, AppState>,
    id: String,
) -> Result<i64, String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .db
        .collections()
        .count_requests(id)
        .map_err(|e| e.to_string())
}