//! 第三方格式导入
//!
//! 目前只支持 Postman Collection v2.1 JSON。
//!
//! TODO(Week 7): Postman 解析器实装
//!
//! 解析流程:
//! 1. 读 JSON 到 `PostmanCollectionV2` 结构
//! 2. 递归遍历 `item` 树(folder / request)
//! 3. 转为我们内部的 `Collection` + `RequestItem`,写入数据库

#![allow(dead_code)]

/// 占位 — Week 7 实装
pub fn import_postman_collection(_json: &str) -> crate::Result<()> {
    todo!("Postman import will be implemented in Week 7")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_import_placeholder() {
        // 暂时只是个占位测试,确保模块能被编译
        assert!(true);
    }
}
