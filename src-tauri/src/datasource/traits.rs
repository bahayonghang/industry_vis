//! 数据源抽象 trait 定义

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::models::HistoryRecord;

/// 数据源元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub name: String,
    pub database: String,
}

impl SourceMetadata {
    pub fn new(name: String, database: String) -> Self {
        Self { name, database }
    }
}

/// 表信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub schema: String,
    pub name: String,
}

impl TableInfo {
    pub fn new(schema: String, name: String) -> Self {
        Self { schema, name }
    }

    /// 获取完整表名
    pub fn full_name(&self) -> String {
        format!("[{}].[{}]", self.schema, self.name)
    }
}

/// 数据源抽象 trait
#[async_trait]
pub trait DataSource: Send + Sync {
    /// 测试连接
    async fn test_connection(&self) -> AppResult<()>;

    /// 获取数据源元数据
    fn metadata(&self) -> &SourceMetadata;

    /// 列出可用表
    async fn list_tables(&self) -> AppResult<Vec<TableInfo>>;

    /// 获取可用标签列表
    async fn get_available_tags(&self, table: &str) -> AppResult<Vec<String>>;

    /// 模糊搜索标签（从 TagDatabase 表）
    async fn search_tags(&self, keyword: &str, limit: usize) -> AppResult<Vec<String>>;

    /// 查询历史数据
    async fn query_history(
        &self,
        table: &str,
        start_time: &str,
        end_time: &str,
        tags: Option<&[String]>,
    ) -> AppResult<Vec<HistoryRecord>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_metadata() {
        let meta = SourceMetadata::new("localhost:1433".to_string(), "TestDB".to_string());
        assert_eq!(meta.name, "localhost:1433");
        assert_eq!(meta.database, "TestDB");
    }

    #[test]
    fn test_table_info() {
        let info = TableInfo::new("dbo".to_string(), "History".to_string());
        assert_eq!(info.full_name(), "[dbo].[History]");
    }
}
