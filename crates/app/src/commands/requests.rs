//! 请求(Request)相关的 Tauri Commands
//!
//! 大部分方法一对一映射 [`RequestRepo`]。
//! 字段级更新(method/url/headers/...)单独暴露,前端可以局部编辑。

use tauri::State;
use uuid::Uuid;

use api_holder_core::collection::{NewRequest, RequestItem};
use api_holder_core::http::{Auth, Body, KeyValue, Method};

use crate::AppState;

fn parse_id(s: &str) -> Result<Uuid, String> {
    Uuid::parse_str(s).map_err(|e| format!("invalid uuid '{s}': {e}"))
}

/// 列出某集合下的所有请求
#[tauri::command]
pub async fn list_requests(
    state: State<'_, AppState>,
    collection_id: String,
) -> Result<Vec<RequestItem>, String> {
    let id = parse_id(&collection_id)?;
    state
        .db
        .requests()
        .list_by_collection(id)
        .map_err(|e| e.to_string())
}

/// 找单个请求
#[tauri::command]
pub async fn get_request(
    state: State<'_, AppState>,
    id: String,
) -> Result<RequestItem, String> {
    let id = parse_id(&id)?;
    state.db.requests().find_by_id(id).map_err(|e| e.to_string())
}

/// 创建请求
#[tauri::command]
pub async fn create_request(
    state: State<'_, AppState>,
    new: NewRequest,
) -> Result<RequestItem, String> {
    state.db.requests().create(new).map_err(|e| e.to_string())
}

/// 改名
#[tauri::command]
pub async fn rename_request(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    state
        .db
        .requests()
        .rename(parse_id(&id)?, new_name)
        .map_err(|e| e.to_string())
}

/// 改 URL
#[tauri::command]
pub async fn update_request_url(
    state: State<'_, AppState>,
    id: String,
    new_url: String,
) -> Result<(), String> {
    state
        .db
        .requests()
        .update_url(parse_id(&id)?, new_url)
        .map_err(|e| e.to_string())
}

/// 改 method
#[tauri::command]
pub async fn update_request_method(
    state: State<'_, AppState>,
    id: String,
    new_method: Method,
) -> Result<(), String> {
    state
        .db
        .requests()
        .update_method(parse_id(&id)?, new_method)
        .map_err(|e| e.to_string())
}

/// 改 headers
#[tauri::command]
pub async fn update_request_headers(
    state: State<'_, AppState>,
    id: String,
    headers: Vec<KeyValue>,
) -> Result<(), String> {
    state
        .db
        .requests()
        .update_headers(parse_id(&id)?, headers)
        .map_err(|e| e.to_string())
}

/// 改 query params
#[tauri::command]
pub async fn update_request_query(
    state: State<'_, AppState>,
    id: String,
    query: Vec<KeyValue>,
) -> Result<(), String> {
    state
        .db
        .requests()
        .update_query(parse_id(&id)?, query)
        .map_err(|e| e.to_string())
}

/// 改 body
#[tauri::command]
pub async fn update_request_body(
    state: State<'_, AppState>,
    id: String,
    body: Body,
) -> Result<(), String> {
    state
        .db
        .requests()
        .update_body(parse_id(&id)?, body)
        .map_err(|e| e.to_string())
}

/// 改 auth
#[tauri::command]
pub async fn update_request_auth(
    state: State<'_, AppState>,
    id: String,
    auth: Auth,
) -> Result<(), String> {
    state
        .db
        .requests()
        .update_auth(parse_id(&id)?, auth)
        .map_err(|e| e.to_string())
}

/// 删除请求
#[tauri::command]
pub async fn delete_request(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state
        .db
        .requests()
        .delete(parse_id(&id)?)
        .map_err(|e| e.to_string())
}

/// 搜索(name + url,大小写不敏感)
#[tauri::command]
pub async fn search_requests(
    state: State<'_, AppState>,
    keyword: String,
) -> Result<Vec<RequestItem>, String> {
    state
        .db
        .requests()
        .search(&keyword)
        .map_err(|e| e.to_string())
}