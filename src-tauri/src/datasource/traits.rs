use async_trait::async_trait;

use crate::error::AppResult;
use crate::models::HistoryRecord;

/// 数据源元数据
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SourceMetadata {
    pub name: String,
    pub database: String,
}

/// 表信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TableInfo {
    pub schema: String,
    pub name: String,
}

/// 数据源抽象 trait
#[async_trait]
#[allow(dead_code)]
pub trait DataSource: Send + Sync {
    /// 测试连接
    async fn test_connection(&self) -> AppResult<()>;

    /// 获取数据源元数据
    fn metadata(&self) -> &SourceMetadata;

    /// 列出可用表
    async fn list_tables(&self) -> AppResult<Vec<TableInfo>>;

    /// 获取可用标签列表
    async fn get_available_tags(&self, table: &str) -> AppResult<Vec<String>>;

    /// 查询历史数据
    async fn query_history(
        &self,
        table: &str,
        start_time: &str,
        end_time: &str,
        tags: Option<&[String]>,
    ) -> AppResult<Vec<HistoryRecord>>;
}
