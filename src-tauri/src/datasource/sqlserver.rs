//! SQL Server 数据源实现
//!
//! 使用 bb8 连接池管理数据库连接。

use async_trait::async_trait;
use std::sync::Arc;
use tiberius::Query;
use tracing::{debug, error, info};

use super::pool::ConnectionPool;
use super::traits::{DataSource, SourceMetadata, TableInfo};
use crate::config::DatabaseConfig;
use crate::error::{AppError, AppResult};
use crate::models::HistoryRecord;

/// SQL Server 数据源实现
pub struct SqlServerSource {
    pool: Arc<ConnectionPool>,
    metadata: SourceMetadata,
}

impl SqlServerSource {
    /// 创建新的数据源（使用连接池）
    pub async fn new(config: DatabaseConfig) -> AppResult<Self> {
        let metadata = SourceMetadata::new(
            format!("{}:{}", config.server, config.port),
            config.database.clone(),
        );

        let pool = ConnectionPool::with_defaults(config).await?;

        Ok(Self {
            pool: Arc::new(pool),
            metadata,
        })
    }

    /// 从现有连接池创建
    pub fn from_pool(pool: Arc<ConnectionPool>) -> Self {
        let config = pool.config();
        let metadata = SourceMetadata::new(
            format!("{}:{}", config.server, config.port),
            config.database.clone(),
        );

        Self { pool, metadata }
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &Arc<ConnectionPool> {
        &self.pool
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
            "SELECT TABLE_SCHEMA, TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE = 'BASE TABLE'"
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

        // 从 TagDataBase 表模糊搜索 TagName
        let search_pattern = format!("%{}%", keyword);

        let sql = format!(
            r#"SELECT DISTINCT TOP {} TagName 
               FROM [TagDataBase] 
               WHERE TagName LIKE @P1
               ORDER BY TagName"#,
            limit
        );

        debug!(target: "industry_vis::datasource",
            database = %database,
            keyword = %keyword,
            pattern = %search_pattern,
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
        let tag_filter = match tags {
            Some(t) if !t.is_empty() => {
                let tag_list = t
                    .iter()
                    .map(|s| format!("'{}'", s.replace('\'', "''")))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("AND TagName IN ({})", tag_list)
            }
            _ => String::new(),
        };

        // 优化SQL：
        // 1. 使用 WITH (NOLOCK) 减少锁等待
        // 2. 只按 DateTime 排序，充分利用索引
        let sql = format!(
            r#"SELECT DateTime, TagName, TagVal, TagQuality 
            FROM [{}] WITH (NOLOCK)
            WHERE DateTime BETWEEN '{}' AND '{}'
            {}
            ORDER BY DateTime"#,
            table.replace(']', "]]"),
            start_time.replace('\'', "''"),
            end_time.replace('\'', "''"),
            tag_filter
        );

        debug!(target: "industry_vis::datasource",
            database = %database,
            table = %table,
            start_time = %start_time,
            end_time = %end_time,
            tag_count = tag_count,
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

        let records: Vec<HistoryRecord> = rows
            .iter()
            .map(|row| {
                let dt: Option<chrono::NaiveDateTime> = row.get(0);
                let date_time = dt
                    .map(|d| d.format("%Y-%m-%dT%H:%M:%S%.3f").to_string())
                    .unwrap_or_default();

                HistoryRecord::new(
                    date_time,
                    row.get::<&str, _>(1).unwrap_or("").trim().to_string(),
                    row.get::<f32, _>(2).unwrap_or(0.0) as f64,
                    row.get::<&str, _>(3).unwrap_or("").trim().to_string(),
                )
            })
            .collect();

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
