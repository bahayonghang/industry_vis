//! SQL Server 数据源实现
//!
//! 使用 bb8 连接池管理数据库连接。
//! 支持通过 SchemaProfile 配置不同厂商的数据库结构。

use async_trait::async_trait;
use std::sync::Arc;
use tiberius::Query;
use tracing::{debug, error, info};

use super::pool::ConnectionPool;
use super::profiles::ProfileRegistry;
use super::schema_profile::SchemaProfile;
use super::traits::{DataSource, SourceMetadata, TableInfo};
use crate::config::DatabaseConfig;
use crate::error::{AppError, AppResult};
use crate::models::HistoryRecord;

/// SQL Server 数据源实现
///
/// 支持通过 `SchemaProfile` 配置不同厂商的数据库结构。
/// 默认使用 `DefaultProfile`。
#[derive(Clone)]
pub struct SqlServerSource {
    pool: Arc<ConnectionPool>,
    metadata: SourceMetadata,
    profile: Arc<dyn SchemaProfile>,
}

impl SqlServerSource {
    /// 创建新的数据源（使用连接池和默认 Profile）
    pub async fn new(config: DatabaseConfig) -> AppResult<Self> {
        Self::new_with_profile(config, ProfileRegistry::default_profile()).await
    }

    /// 创建新的数据源（使用连接池和指定 Profile）
    pub async fn new_with_profile(
        config: DatabaseConfig,
        profile: Arc<dyn SchemaProfile>,
    ) -> AppResult<Self> {
        let metadata = SourceMetadata::new(
            format!("{}:{}", config.server, config.port),
            config.database.clone(),
        );

        let pool = ConnectionPool::with_defaults(config).await?;

        info!(target: "industry_vis::datasource",
            profile = %profile.name(),
            "创建 SqlServerSource，使用 Profile: {}", profile.name()
        );

        Ok(Self {
            pool: Arc::new(pool),
            metadata,
            profile,
        })
    }

    /// 从现有连接池创建（使用默认 Profile）
    pub fn from_pool(pool: Arc<ConnectionPool>) -> Self {
        Self::from_pool_with_profile(pool, ProfileRegistry::default_profile())
    }

    /// 从现有连接池创建（使用指定 Profile）
    pub fn from_pool_with_profile(
        pool: Arc<ConnectionPool>,
        profile: Arc<dyn SchemaProfile>,
    ) -> Self {
        let config = pool.config();
        let metadata = SourceMetadata::new(
            format!("{}:{}", config.server, config.port),
            config.database.clone(),
        );

        Self {
            pool,
            metadata,
            profile,
        }
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &Arc<ConnectionPool> {
        &self.pool
    }

    /// 获取 Profile 引用
    pub fn profile(&self) -> &Arc<dyn SchemaProfile> {
        &self.profile
    }

    /// 获取数据库名称
    fn database(&self) -> &str {
        &self.metadata.database
    }
}

#[async_trait]
impl DataSource for SqlServerSource {
    async fn test_connection(&self) -> AppResult<()> {
        let mut conn = self.pool.get().await?;

        // Execute a simple query to verify connection
        let query = Query::new("SELECT 1");
        query
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Connection(format!("测试查询失败: {}", e)))?;

        Ok(())
    }

    fn metadata(&self) -> &SourceMetadata {
        &self.metadata
    }

    async fn list_tables(&self) -> AppResult<Vec<TableInfo>> {
        let mut conn = self.pool.get().await?;

        let query = Query::new(
            "SELECT TABLE_SCHEMA, TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE = 'BASE TABLE'",
        );

        let stream = query
            .query(&mut *conn)
            .await
            .map_err(|e| AppError::Query(format!("获取表列表失败: {}", e)))?;

        let rows = stream
            .into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("获取表结果失败: {}", e)))?;

        let tables = rows
            .iter()
            .map(|row| {
                TableInfo::new(
                    row.get::<&str, _>(0).unwrap_or("dbo").to_string(),
                    row.get::<&str, _>(1).unwrap_or("").to_string(),
                )
            })
            .collect();

        Ok(tables)
    }

    async fn get_available_tags(&self, table: &str) -> AppResult<Vec<String>> {
        let mut conn = self.pool.get().await?;

        let sql = format!(
            "SELECT DISTINCT TagName FROM [{}] ORDER BY TagName",
            table.replace(']', "]]")
        );

        let query = Query::new(&sql);
        let stream = query
            .query(&mut *conn)
            .await
            .map_err(|e| AppError::Query(format!("获取标签失败: {}", e)))?;

        let rows = stream
            .into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("获取标签结果失败: {}", e)))?;

        let tags = rows
            .iter()
            .filter_map(|row| row.get::<&str, _>(0).map(|s| s.trim().to_string()))
            .collect();

        Ok(tags)
    }

    async fn search_tags(&self, keyword: &str, limit: usize) -> AppResult<Vec<String>> {
        let mut conn = self.pool.get().await?;
        let database = self.database().to_string();

        // 使用 Profile 生成 SQL
        let search_pattern = format!("%{}%", keyword);
        let sql = self.profile.tag_search_sql(limit);

        debug!(target: "industry_vis::datasource",
            database = %database,
            keyword = %keyword,
            pattern = %search_pattern,
            profile = %self.profile.name(),
            "执行标签搜索 SQL: {}", sql
        );

        let mut query = Query::new(&sql);
        query.bind(&search_pattern);

        let stream = query.query(&mut *conn).await.map_err(|e| {
            error!(target: "industry_vis::datasource",
                database = %database,
                keyword = %keyword,
                error = %e,
                "标签搜索失败"
            );
            AppError::Query(format!(
                "搜索标签失败 (数据库: {}, 关键词: {}): {}",
                database, keyword, e
            ))
        })?;

        let rows = stream
            .into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("获取搜索结果失败: {}", e)))?;

        let tags: Vec<String> = rows
            .iter()
            .filter_map(|row| row.get::<&str, _>(0).map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect();

        info!(target: "industry_vis::datasource",
            database = %database,
            keyword = %keyword,
            count = tags.len(),
            "标签搜索完成"
        );

        Ok(tags)
    }

    async fn query_history(
        &self,
        table: &str,
        start_time: &str,
        end_time: &str,
        tags: Option<&[String]>,
    ) -> AppResult<Vec<HistoryRecord>> {
        let mut conn = self.pool.get().await?;
        let database = self.database().to_string();

        let tag_count = tags.map(|t| t.len()).unwrap_or(0);

        // 使用 Profile 生成 SQL
        let tag_filter = self.profile.build_tag_filter(tags);
        let sql = self
            .profile
            .history_query_sql(table, start_time, end_time, &tag_filter);

        debug!(target: "industry_vis::datasource",
            database = %database,
            table = %table,
            start_time = %start_time,
            end_time = %end_time,
            tag_count = tag_count,
            profile = %self.profile.name(),
            "执行历史查询"
        );

        let query = Query::new(&sql);
        let stream = query.query(&mut *conn).await.map_err(|e| {
            error!(target: "industry_vis::datasource",
                database = %database,
                error = %e,
                "历史查询失败"
            );
            AppError::Query(format!("历史查询失败: {}", e))
        })?;

        let rows = stream
            .into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("获取历史结果失败: {}", e)))?;

        // 使用 Profile 映射行数据
        let mut records: Vec<HistoryRecord> = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            records.push(self.profile.map_history_row(row)?);
        }

        info!(target: "industry_vis::datasource",
            database = %database,
            table = %table,
            records = records.len(),
            "历史查询完成"
        );

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        // 这个测试验证 SourceMetadata 的创建逻辑
        let meta = SourceMetadata::new("localhost:1433".to_string(), "TestDB".to_string());
        assert_eq!(meta.name, "localhost:1433");
        assert_eq!(meta.database, "TestDB");
    }

    // 数据库连接测试需要实际的数据库，在集成测试中进行
}
