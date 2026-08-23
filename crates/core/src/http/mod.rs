//! HTTP 请求执行模块
//!
//! 封装 `reqwest`,提供:
//! - 多种 HTTP 方法支持
//! - Headers / Query / Body / Auth 配置
//! - 响应统一封装(状态码 / 耗时 / headers / body)
//! - `{{var}}` 变量插值
//!
//! 设计目标:可独立单元测试,无需启动 Tauri。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::environment::interpolate;

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

/// 把 [`Request`] 里所有字符串字段中的 `{{var}}` 替换掉,返回新的 Request。
///
/// 为什么抽出来:`execute()` 只关心「怎么发请求」,
/// 「变量解析」是独立的关注点,这样也方便写单元测试。
fn apply_vars(req: Request, vars: &HashMap<String, String>) -> Request {
    let mut req = req;

    // URL 是最常见的占位符位置
    req.url = interpolate(&req.url, vars);

    // Headers / Query:只替换 value,不替换 key(key 是写死的字段名)
    req.headers = req.headers.into_iter().map(|mut kv| {
        kv.value = interpolate(&kv.value, vars);
        kv
    }).collect();
    req.query = req.query.into_iter().map(|mut kv| {
        kv.value = interpolate(&kv.value, vars);
        kv
    }).collect();

    // Auth 四种变体都要逐一处理
    req.auth = match req.auth {
        Auth::None => Auth::None,
        Auth::Bearer { token } => Auth::Bearer {
            token: interpolate(&token, vars),
        },
        Auth::Basic { username, password } => Auth::Basic {
            username: interpolate(&username, vars),
            password: interpolate(&password, vars),
        },
        Auth::ApiKey { key, value, in_header } => Auth::ApiKey {
            key: interpolate(&key, vars),
            value: interpolate(&value, vars),
            in_header,
        },
    };

    // Body 三种带内容的变体都要处理
    req.body = match req.body {
        Body::None => Body::None,
        Body::Json { content } => Body::Json {
            content: interpolate(&content, vars),
        },
        Body::Form { fields } => Body::Form {
            fields: fields.into_iter().map(|mut kv| {
                kv.value = interpolate(&kv.value, vars);
                kv
            }).collect(),
        },
        Body::Raw { content, content_type } => Body::Raw {
            content: interpolate(&content, vars),
            content_type: interpolate(&content_type, vars),
        },
    };

    req
}

/// 核心:把我们的 [`Request`] 真正发出去,拿到 [`Response`]。
///
/// ## 执行流程(11 步)
///
/// 1. **变量插值**(通过 `execute_with_vars`)
/// 2. 启动计时器 `Instant::now()`
/// 3. 构造 `reqwest::Client`(30 秒超时)
/// 4. 搭起 `RequestBuilder`(method + url,这一步会校验 URL)
/// 5. 拼 Query Params(只取启用且 key 非空的)
/// 6. 拼 Headers(非法 header 名/值静默跳过)
/// 7. 处理 Auth(Bearer / Basic / ApiKey)
/// 8. 处理 Body(Json / Form / Raw)
/// 9. `send()` 真正发请求
/// 10. 把 reqwest 的 `HeaderMap` 拷成 `Vec<KeyValue>`
/// 11. `text()` 读 body,算耗时,装进 [`Response`] 返回
pub async fn execute(req: Request) -> crate::Result<Response> {
    // 不带变量的便捷入口
    execute_with_vars(req, &HashMap::new()).await
}

/// 同 [`execute`],但会先用 `vars` 把 `{{var}}` 替换掉。
///
/// 调用场景:UI 上选了某个 Environment,把它激活的变量表传进来。
pub async fn execute_with_vars(
    req: Request,
    vars: &HashMap<String, String>,
) -> crate::Result<Response> {
    // ─── 1. 变量插值 ──────────────────────────────────────────────
    // 即使 vars 为空也调一下,逻辑统一;`interpolate` 内部无 vars 时是 no-op。
    let req = apply_vars(req, vars);

    // ─── 2. 计时开始 ──────────────────────────────────────────────
    let started = Instant::now();

    // ─── 3. 构造 HTTP 客户端(30 秒超时)───────────────────────────
    // builder().build() 返回 Result,`?` 借助 Error 里的
    // `#[from] reqwest::Error` 自动转成我们的 Error。
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // ─── 4. 搭起 RequestBuilder(method + url)─────────────────────
    // 这一步就开始解析 URL 了,非法 URL(没 scheme 之类)会在 `?` 处报错。
    let mut builder = client.request(
        req.method.to_reqwest(),
        req.url.as_str(),
    );

    // ─── 5. Query Params ─────────────────────────────────────────
    // 只挑启用的、key 非空的 — 避免 `?=foo` 这种空键污染 URL。
    //   reqwest 收到 (k, v) 数组会自动拼成 `?k=v&k=v...`
    for kv in req.query.iter()
        .filter(|kv| kv.enabled && !kv.key.is_empty())
    {
        builder = builder.query(&[(kv.key.as_str(), kv.value.as_str())]);
    }

    // ─── 6. Headers ──────────────────────────────────────────────
    // 非法 header 名(含换行/控制字符)或非 UTF-8 value,静默跳过 —
    // 不让单个坏 header 把整个请求搞挂。
    for kv in req.headers.iter()
        .filter(|kv| kv.enabled && !kv.key.is_empty())
    {
        let name = match kv.key.parse::<reqwest::header::HeaderName>() {
            Ok(n) => n,
            Err(_) => continue, // 跳过非法 header 名
        };
        let value = match kv.value.parse::<reqwest::header::HeaderValue>() {
            Ok(v) => v,
            Err(_) => continue, // 跳过非法 header 值
        };
        builder = builder.header(name, value);
    }

    // ─── 7. Auth ─────────────────────────────────────────────────
    builder = match req.auth {
        // 不用认证,builder 不动
        Auth::None => builder,
        // Authorization: Bearer <token>
        Auth::Bearer { token } => builder.bearer_auth(token),
        // Authorization: Basic base64(user:pass)
        Auth::Basic { username, password } => {
            builder.basic_auth(username, Some(password))
        }
        // ApiKey 可以放 header 里,也可以放 query 里(看 in_header)
        Auth::ApiKey { key, value, in_header } => {
            if in_header {
                match key.parse::<reqwest::header::HeaderName>() {
                    Ok(name) => builder.header(name, value),
                    Err(_) => builder, // 跳过非法 header 名
                }
            } else {
                // 拼到 query 上,会和上面 query 字段的内容合并。
                builder.query(&[(key.as_str(), value.as_str())])
            }
        }
    };

    // ─── 8. Body ─────────────────────────────────────────────────
    builder = match req.body {
        // GET / HEAD 等,不发 body
        Body::None => builder,
        // JSON:显式设 Content-Type,原样发送 content 字符串
        Body::Json { content } => builder
            .header("Content-Type", "application/json")
            .body(content),
        // Form:application/x-www-form-urlencoded
        Body::Form { fields } => {
            // 同样只挑启用且 key 非空的
            let form: Vec<(&str, &str)> = fields.iter()
                .filter(|kv| kv.enabled && !kv.key.is_empty())
                .map(|kv| (kv.key.as_str(), kv.value.as_str()))
                .collect();
            builder.form(&form)
        }
        // Raw:自定义 content-type 的纯文本
        Body::Raw { content, content_type } => builder
            .header("Content-Type", content_type)
            .body(content),
    };

    // ─── 9. 真正发出去 ──────────────────────────────────────────
    // `?` 把 reqwest::Error 转成我们的 Error(因为 Error 里有 #[from])
    let response = builder.send().await?;

    // ─── 10. 拷出 headers ────────────────────────────────────────
    // ⚠️ 顺序敏感:`response.text()` 会消费 body,但 `iter()` 只是借用,
    // 所以**先**把 headers 拷出来,再读 body。
    let headers: Vec<KeyValue> = response.headers()
        .iter()
        .map(|(name, value)| KeyValue {
            key: name.as_str().to_string(),
            // header value 可能是非 UTF-8 bytes,这里丢成空串而不是炸掉
            value: value.to_str().unwrap_or("").to_string(),
            enabled: true,
        })
        .collect();

    let status = response.status();

    // ─── 11. 读 body / 算耗时 / 装 Response ─────────────────────
    // `text()` 返回 String,`.len()` 就是字节数(UTF-8 视角)
    let body_text = response.text().await?;
    let size_bytes = body_text.len();
    let duration_ms = started.elapsed().as_millis() as u64;

    Ok(Response {
        status: status.as_u16(),
        // 状态文字描述,如 "OK" / "Not Found"
        status_text: status.canonical_reason().unwrap_or("").to_string(),
        headers,
        body: body_text,
        duration_ms,
        size_bytes,
    })
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

    #[test]
    fn test_apply_vars() {
        // 验证 {{var}} 在 url / headers / body 都会被替换
        // (auth 的插值逻辑也类似,这里就不重复测了)
        let mut vars = HashMap::new();
        vars.insert("host".to_string(), "api.example.com".to_string());
        vars.insert("token".to_string(), "secret123".to_string());

        let req = Request {
            method: Method::Get,
            url: "https://{{host}}/users".to_string(),
            headers: vec![KeyValue {
                key: "Authorization".to_string(),
                value: "Bearer {{token}}".to_string(),
                enabled: true,
            }],
            query: vec![],
            body: Body::Json {
                content: r#"{"base": "https://{{host}}"}"#.to_string(),
            },
            auth: Auth::None,
        };

        let result = apply_vars(req, &vars);
        assert_eq!(result.url, "https://api.example.com/users");
        assert_eq!(result.headers[0].value, "Bearer secret123");
        match result.body {
            Body::Json { content } => {
                assert_eq!(content, r#"{"base": "https://api.example.com"}"#);
            }
            _ => panic!("body type 变了"),
        }
    }
}