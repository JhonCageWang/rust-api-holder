//! 环境变量管理
//!
//! 一个 Environment 包含若干 `key=value` 变量。
//! 同一时刻只能有一个 Environment 是 active 的。
//!
//! 在发送请求前,把 `{{varName}}` 替换为变量值。
//!
//! TODO(Week 2): 变量插值引擎
//! TODO(Week 6): Environment 的 CRUD + 激活切换

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

/// 把字符串中的 `{{var}}` 替换为给定变量表的对应值。
///
/// 规则:
/// - 找不到的变量保留原样(如 `{{unknown}}`),便于排错
/// - 不会嵌套解析(防止 `{{a{{b}}}}` 的边界问题)
/// - 不做 trim / 大小写转换 — 完全按 key 精确匹配
///
/// # Example
///
/// ```
/// use api_holder_core::environment::interpolate;
/// let mut vars = std::collections::HashMap::new();
/// vars.insert("host".to_string(), "api.example.com".to_string());
/// assert_eq!(
///     interpolate("https://{{host}}/users", &vars),
///     "https://api.example.com/users"
/// );
/// ```
pub fn interpolate(input: &str, vars: &std::collections::HashMap<String, String>) -> String {
    // Week 2 实装,先用最朴素实现
    let mut result = input.to_string();
    for (key, value) in vars {
        let token = format!("{{{{{}}}}}", key);
        result = result.replace(&token, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_interpolate_basic() {
        let mut vars = HashMap::new();
        vars.insert("host".to_string(), "api.example.com".to_string());
        vars.insert("id".to_string(), "42".to_string());

        assert_eq!(
            interpolate("https://{{host}}/users/{{id}}", &vars),
            "https://api.example.com/users/42"
        );
    }

    #[test]
    fn test_interpolate_missing_keeps_original() {
        let vars = HashMap::new();
        assert_eq!(
            interpolate("hello {{name}}", &vars),
            "hello {{name}}"
        );
    }

    #[test]
    fn test_interpolate_no_vars() {
        let vars = HashMap::new();
        assert_eq!(interpolate("plain text", &vars), "plain text");
    }
}