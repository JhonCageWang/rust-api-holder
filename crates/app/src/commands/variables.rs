//! 环境变量(Variable)相关的 Tauri Commands

use tauri::State;
use uuid::Uuid;

use api_holder_core::environment::Variable;

use crate::AppState;

fn parse_id(s: &str) -> Result<Uuid, String> {
    Uuid::parse_str(s).map_err(|e| format!("invalid uuid '{s}': {e}"))
}

/// 列出某环境下的所有变量
#[tauri::command]
pub async fn list_variables(
    state: State<'_, AppState>,
    environment_id: String,
) -> Result<Vec<Variable>, String> {
    state
        .db
        .variables()
        .list_by_env(parse_id(&environment_id)?)
        .map_err(|e| e.to_string())
}

/// 创建单个变量
#[tauri::command]
pub async fn create_variable(
    state: State<'_, AppState>,
    environment_id: String,
    key: String,
    value: String,
) -> Result<Variable, String> {
    state
        .db
        .variables()
        .create(parse_id(&environment_id)?, key, value)
        .map_err(|e| e.to_string())
}

/// 更新 value 和 enabled 状态
#[tauri::command]
pub async fn update_variable(
    state: State<'_, AppState>,
    id: String,
    new_value: String,
    enabled: bool,
) -> Result<(), String> {
    state
        .db
        .variables()
        .update(parse_id(&id)?, new_value, enabled)
        .map_err(|e| e.to_string())
}

/// 删除变量
#[tauri::command]
pub async fn delete_variable(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state
        .db
        .variables()
        .delete(parse_id(&id)?)
        .map_err(|e| e.to_string())
}

/// 批量替换整个环境的变量列表(事务:全删全插)
#[tauri::command]
pub async fn bulk_replace_variables(
    state: State<'_, AppState>,
    environment_id: String,
    variables: Vec<Variable>,
) -> Result<(), String> {
    state
        .db
        .variables()
        .bulk_replace(parse_id(&environment_id)?, variables)
        .map_err(|e| e.to_string())
}