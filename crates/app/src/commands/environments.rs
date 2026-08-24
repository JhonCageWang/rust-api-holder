//! 环境(Environment)相关的 Tauri Commands

use tauri::State;
use uuid::Uuid;

use api_holder_core::environment::Environment;

use crate::AppState;

fn parse_id(s: &str) -> Result<Uuid, String> {
    Uuid::parse_str(s).map_err(|e| format!("invalid uuid '{s}': {e}"))
}

/// 列出所有环境
#[tauri::command]
pub async fn list_environments(
    state: State<'_, AppState>,
) -> Result<Vec<Environment>, String> {
    state.db.environments().list_all().map_err(|e| e.to_string())
}

/// 取当前激活的环境(没有时返回 Err)
#[tauri::command]
pub async fn get_active_environment(
    state: State<'_, AppState>,
) -> Result<Option<Environment>, String> {
    match state.db.environments().find_active() {
        Ok(env) => Ok(Some(env)),
        Err(api_holder_core::Error::NotFound(_)) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 创建环境
#[tauri::command]
pub async fn create_environment(
    state: State<'_, AppState>,
    name: String,
) -> Result<Environment, String> {
    state
        .db
        .environments()
        .create(name)
        .map_err(|e| e.to_string())
}

/// 重命名环境
#[tauri::command]
pub async fn rename_environment(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    state
        .db
        .environments()
        .rename(parse_id(&id)?, new_name)
        .map_err(|e| e.to_string())
}

/// 原子切换激活(把所有 is_active 清零,设置目标为 1)
#[tauri::command]
pub async fn set_active_environment(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state
        .db
        .environments()
        .set_active(parse_id(&id)?)
        .map_err(|e| e.to_string())
}

/// 删除环境(级联删 variables)
#[tauri::command]
pub async fn delete_environment(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state
        .db
        .environments()
        .delete(parse_id(&id)?)
        .map_err(|e| e.to_string())
}