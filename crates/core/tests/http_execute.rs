//! `http::execute` 的集成测试
//!
//! ## 测试策略
//!
//! 用 [`httpmock`] 在测试进程里起一个临时 mock HTTP 服务器,验证两件事:
//!
//! 1. **出去的请求**真的带上了参数(method / url / headers / query / body / auth)
//! 2. **回来的响应**真的被装进了我们的 `Response` 结构
//!
//! 这样不用打真实网络,又快又稳,CI 也能放心跑。
//!
//! ## 读测试的顺序
//!
//! 1. `test_get_basic`                — 最简 GET,搞懂测试骨架
//! 2. `test_post_json_body`           — POST + JSON body
//! 3. `test_form_body`                — POST + Form body
//! 4. `test_raw_body`                 — POST + Raw body + 自定义 Content-Type
//! 5. `test_query_params`             — query string 拼对
//! 6. `test_custom_headers`           — 自定义 header 发出去
//! 7. `test_bearer_auth` / `test_basic_auth` — Auth 自动加 header
//! 8. `test_status_404_not_error`     — 状态码 vs 网络错误的区别
//! 9. `test_bad_url_returns_error`    — 非法 URL 怎么报错
//! 10. `test_disabled_param_skipped`  — 边界:`enabled=false` 该跳过
//! 11. `test_with_vars_interpolation` — `{{var}}` 端到端替换
//!
//! ## 跑测试
//!
//! ```bash
//! cargo test -p api-holder-core --test http_execute
//! # 看具体某一条:
//! cargo test -p api-holder-core --test http_execute test_bearer_auth -- --nocapture
//! # 加 --nocapture 可以看到 println! / eprintln! 输出
//! ```

use api_holder_core::http::{
    default_client, execute, execute_with_vars, Auth, Body, KeyValue, Method, Request,
};
use api_holder_core::Error;
use base64::Engine;
use httpmock::prelude::*;
use std::collections::HashMap;

// ════════════════════════════════════════════════════════════════════
//  helpers
// ════════════════════════════════════════════════════════════════════
//
// Request 有 6 个字段,每个 test 都构造一遍太啰嗦。
// 封装两个常见模式,后面的 test 就能只关注「差异部分」。

/// 共享的 reqwest::Client(测试里都复用这一个,享受连接池)
/// 注:reqwest::Client 实现了 Clone(内部 Arc),clone 是 cheap 的。
fn shared_client() -> reqwest::Client {
    default_client().expect("default client build failed")
}

/// 构造一个最简单的 GET 请求(headers/query/body/auth 全空)
fn make_get(server: &MockServer, path: &str) -> Request {
    Request {
        method: Method::Get,
        url: server.url(path),
        headers: vec![],
        query: vec![],
        body: Body::None,
        auth: Auth::None,
    }
}

/// 在 `make_get` 基础上加 JSON body
fn make_post_json(server: &MockServer, path: &str, json: &str) -> Request {
    Request {
        method: Method::Post,
        body: Body::Json {
            content: json.to_string(),
        },
        ..make_get(server, path)
    }
}

// ════════════════════════════════════════════════════════════════════
//  1. 最基础的 GET —— Response 字段都被装好
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_get_basic() {
    let server = MockServer::start();
    let client = shared_client();

    // .mock 接受两个闭包:when = 进来的请求长啥样,then = 给什么回
    let mock = server.mock(|when, then| {
        when.method(GET).path("/hello");
        then.status(200)
            .header("content-type", "text/plain")
            .body("world");
    });

    let resp = execute(&client, make_get(&server, "/hello")).await.unwrap();

    // ── 响应主体 ────────────────────────────────
    assert_eq!(resp.status, 200);
    assert_eq!(resp.status_text, "OK");
    assert_eq!(resp.body, "world");
    assert_eq!(resp.size_bytes, "world".len());

    // ── 响应头(注意是 Vec<KeyValue>,不是 HashMap)──
    let ct = resp
        .headers
        .iter()
        .find(|kv| kv.key == "content-type")
        .expect("应该有 content-type 响应头");
    assert_eq!(ct.value, "text/plain");

    // ── 计时合理(mock server 本机,应该 << 1s)────
    assert!(resp.duration_ms < 1000, "耗时异常: {}ms", resp.duration_ms);

    // ── 验证 mock 真的被命中(没命中 → panic) ─────
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  2. POST + JSON body —— Content-Type 自动设,body 原样发出
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_post_json_body() {
    let server = MockServer::start();
    let client = shared_client();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/users")
            .header("content-type", "application/json")
            .body(r#"{"name":"alice"}"#); // 必须**完全**匹配发来的 body
        then.status(201).body(r#"{"id":42}"#);
    });

    let resp = execute(
        &client,
        make_post_json(&server, "/users", r#"{"name":"alice"}"#),
    )
    .await
    .unwrap();

    assert_eq!(resp.status, 201);
    assert_eq!(resp.body, r#"{"id":42}"#);
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  3. POST + Form body —— Content-Type: application/x-www-form-urlencoded
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_form_body() {
    let server = MockServer::start();
    let client = shared_client();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/login")
            .header("content-type", "application/x-www-form-urlencoded")
            // body 会被序列化成 user=alice&pass=wonderland
            .body("user=alice&pass=wonderland");
        then.status(200).body("logged in");
    });

    let req = Request {
        method: Method::Post,
        url: server.url("/login"),
        headers: vec![],
        query: vec![],
        body: Body::Form {
            fields: vec![
                KeyValue {
                    key: "user".into(),
                    value: "alice".into(),
                    enabled: true,
                },
                KeyValue {
                    key: "pass".into(),
                    value: "wonderland".into(),
                    enabled: true,
                },
            ],
        },
        auth: Auth::None,
    };

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, "logged in");
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  4. POST + Raw body —— 自定义 Content-Type
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_raw_body() {
    let server = MockServer::start();
    let client = shared_client();

    let xml_payload = r#"<?xml version="1.0"?><user><name>alice</name></user>"#;

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/soap")
            .header("content-type", "application/xml")
            .body(xml_payload);
        then.status(200).body("ok");
    });

    let req = Request {
        method: Method::Post,
        url: server.url("/soap"),
        headers: vec![],
        query: vec![],
        body: Body::Raw {
            content: xml_payload.to_string(),
            content_type: "application/xml".into(),
        },
        auth: Auth::None,
    };

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  5. Query Params —— ?k=v 拼对了
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_query_params() {
    let server = MockServer::start();
    let client = shared_client();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/search")
            .query_param("q", "rust")
            .query_param("page", "2");
        then.status(200).body("found");
    });

    let mut req = make_get(&server, "/search");
    req.query = vec![
        KeyValue {
            key: "q".into(),
            value: "rust".into(),
            enabled: true,
        },
        KeyValue {
            key: "page".into(),
            value: "2".into(),
            enabled: true,
        },
    ];

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  6. 自定义 Header —— 用户传的 header 真的发出去
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_custom_headers() {
    let server = MockServer::start();
    let client = shared_client();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/x")
            .header("x-trace-id", "abc-123")
            .header("x-tenant", "acme");
        then.status(200);
    });

    let mut req = make_get(&server, "/x");
    req.headers = vec![
        KeyValue {
            key: "X-Trace-Id".into(),
            value: "abc-123".into(),
            enabled: true,
        },
        KeyValue {
            key: "X-Tenant".into(),
            value: "acme".into(),
            enabled: true,
        },
    ];

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  7. Bearer Auth —— Authorization: Bearer xxx 自动加上
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_bearer_auth() {
    let server = MockServer::start();
    let client = shared_client();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/secure")
            .header("authorization", "Bearer my-secret-token");
        then.status(200);
    });

    let mut req = make_get(&server, "/secure");
    req.auth = Auth::Bearer {
        token: "my-secret-token".into(),
    };

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  8. Basic Auth —— Authorization: Basic base64(user:pass) 自动加上
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_basic_auth() {
    let server = MockServer::start();
    let client = shared_client();

    // 主动算一遍 base64,避免依赖某个特定 base64 crate 版本的细节
    let expected_token = base64::engine::general_purpose::STANDARD.encode("alice:wonderland");

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/secure")
            .header("authorization", format!("Basic {expected_token}"));
        then.status(200);
    });

    let mut req = make_get(&server, "/secure");
    req.auth = Auth::Basic {
        username: "alice".into(),
        password: "wonderland".into(),
    };

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  9. 404 不是 Error —— 业务语义上服务器正常响应,不该被当 Error
//     Error 只在「请求压根没发出去 / 超时」时才会返回
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_status_404_not_error() {
    let server = MockServer::start();
    let client = shared_client();

    let mock = server.mock(|when, then| {
        when.method(GET).path("/missing");
        then.status(404).body("not found");
    });

    let resp = execute(&client, make_get(&server, "/missing"))
        .await
        .unwrap();

    assert_eq!(resp.status, 404);
    assert_eq!(resp.status_text, "Not Found");
    assert_eq!(resp.body, "not found");
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
//  10. 非法 URL → Error::Http
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_bad_url_returns_error() {
    let client = shared_client();
    let req = Request {
        method: Method::Get,
        url: "not-a-valid-url".into(), // 没 http://
        headers: vec![],
        query: vec![],
        body: Body::None,
        auth: Auth::None,
    };

    let result = execute(&client, req).await;
    assert!(result.is_err(), "非法 URL 应该返回 Err,实际: {result:?}");

    // 验证错误类型是我们 enum 里的 Http 变体(不是 Database / IO 之类)
    match result.unwrap_err() {
        Error::Http(_) => {} // OK
        other => panic!("期望 Error::Http(_),实际: {other:?}"),
    }
}

// ════════════════════════════════════════════════════════════════════
//  11. enabled=false 的字段**不会**出现在发出的请求里
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_disabled_param_skipped() {
    let server = MockServer::start();
    let client = shared_client();

    // httpmock 0.7 的 query_param() 只检查"包含某个参数",
    // 无法表达"不能同时有 hidden=secret"。
    // 所以用 when.matches() 拿到底层 HttpMockRequest,自己检查 query_params。
    let mock = server.mock(|when, then| {
        when.method(GET).path("/x").matches(|req| {
            // req.query_params: Option<Vec<(String, String)>>
            // 我们断言:有 visible=yes,**没有** hidden
            let qps = req.query_params.as_deref().unwrap_or(&[]);
            let has_visible = qps.iter().any(|(k, v)| k == "visible" && v == "yes");
            let has_hidden = qps.iter().any(|(k, _)| k == "hidden");
            has_visible && !has_hidden
        });
        then.status(200);
    });

    let mut req = make_get(&server, "/x");
    req.query = vec![
        KeyValue {
            key: "visible".into(),
            value: "yes".into(),
            enabled: true,
        },
        KeyValue {
            key: "hidden".into(),
            value: "secret".into(),
            enabled: false, // ← 关掉
        },
    ];

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
// 12. execute_with_vars —— {{var}} 在 URL 里被替换,发出去对得上
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_with_vars_interpolation() {
    let server = MockServer::start();
    let client = shared_client();

    // 期望服务器收到 /users/42(已经被插值替换)
    let mock = server.mock(|when, then| {
        when.method(GET).path("/users/42");
        then.status(200).body("ok");
    });

    // URL 里塞 {{id}} 字符串,让 execute_with_vars 去替换
    //   server.url("") 返回类似 http://127.0.0.1:34567
    //   format! 第二个参数 "{{id}}" 是字面量
    let url_with_var = format!("{}/users/{}", server.url(""), "{{id}}");

    let req = Request {
        method: Method::Get,
        url: url_with_var,
        headers: vec![],
        query: vec![],
        body: Body::None,
        auth: Auth::None,
    };

    let mut vars = HashMap::new();
    vars.insert("id".to_string(), "42".to_string());

    let resp = execute_with_vars(&client, req, &vars).await.unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body, "ok");
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
// 13. ApiKey in_header:真的放到 header 里
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_apikey_in_header() {
    let server = MockServer::start();
    let client = shared_client();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/x")
            .header("x-api-key", "my-secret-key");
        then.status(200);
    });

    let mut req = make_get(&server, "/x");
    req.auth = Auth::ApiKey {
        key: "X-Api-Key".into(),
        value: "my-secret-key".into(),
        in_header: true,
    };

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    mock.assert();
}

// ════════════════════════════════════════════════════════════════════
// 14. ApiKey in_query:真的放到 query string 里
// ════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_apikey_in_query() {
    let server = MockServer::start();
    let client = shared_client();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/x")
            .query_param("api_key", "my-secret-key");
        then.status(200);
    });

    let mut req = make_get(&server, "/x");
    req.auth = Auth::ApiKey {
        key: "api_key".into(),
        value: "my-secret-key".into(),
        in_header: false,
    };

    let resp = execute(&client, req).await.unwrap();
    assert_eq!(resp.status, 200);
    mock.assert();
}
