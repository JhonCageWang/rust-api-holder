//! HTTP 请求执行模块
//!
//! 封装 `reqwest`,提供:
//! - 多种 HTTP 方法支持
//! - Headers / Query / Body / Auth 配置
//! - 响应统一封装(状态码 / 耗时 / headers / body)
//! - `{{var}}` 变量插值
//!
//! 设计目标:可独立单元测试,无需启动 Tauri。

#![allow(dead_code)] // Week 2 才实现,先放占位

use serde::{Deserialize, Serialize};

/// HTTP 方法枚举(覆盖常用方法)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl Method {
    /// 转为 reqwest 用的 Method
    pub fn to_reqwest(self) -> reqwest::Method {
        match self {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Delete => reqwest::Method::DELETE,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        }
    }
}

/// 单个键值对(用于 Headers / Query Params)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// HTTP 请求体类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    /// 无 body(GET / HEAD 等)
    None,
    /// JSON body
    Json { content: String },
    /// 表单 body(application/x-www-form-urlencoded)
    Form { fields: Vec<KeyValue> },
    /// 原始文本(text/plain 等)
    Raw { content: String, content_type: String },
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Auth {
    None,
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { key: String, value: String, in_header: bool },
}

/// 完整的 HTTP 请求定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: Method,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<KeyValue>,
    #[serde(default)]
    pub query: Vec<KeyValue>,
    #[serde(default)]
    pub body: Body,
    #[serde(default)]
    pub auth: Auth,
}

/// HTTP 响应(用于 UI 展示)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<KeyValue>,
    pub body: String,
    pub duration_ms: u64,
    pub size_bytes: usize,
}

/// TODO(Week 2): 实现实际的 HTTP 执行逻辑
///
/// ```no_run
/// use api_holder_core::http::{Request, Method, Response};
/// async fn example() -> Result<Response, Box<dyn std::error::Error>> {
///     let req = Request { /* ... */ method: Method::Get, url: "https://httpbin.org/get".into(), headers: vec![], query: vec![], body: Default::default(), auth: Default::default() };
///     // client.execute(req).await
///     todo!()
/// }
/// ```
pub async fn execute(_req: Request) -> crate::Result<Response> {
    // Week 2 实装
    todo!("HTTP execution will be implemented in Week 2")
}

impl Default for Body {
    fn default() -> Self {
        Body::None
    }
}

impl Default for Auth {
    fn default() -> Self {
        Auth::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_serde() {
        let m = Method::Get;
        let s = serde_json::to_string(&m).unwrap();
        assert_eq!(s, "\"GET\"");

        let m: Method = serde_json::from_str("\"POST\"").unwrap();
        assert_eq!(m, Method::Post);
    }

    #[test]
    fn test_body_default() {
        assert!(matches!(Body::default(), Body::None));
    }
}