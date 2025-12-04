//! 查询服务
//!
//! 整合数据源、缓存、数据处理，提供统一的查询接口。

use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use crate::cache::{CacheKey, QueryCache};
use crate::datasource::{ConnectionPool, DataSource, SqlServerSource};
use crate::error::AppResult;
use crate::models::{DataProcessingConfig, HistoryRecord, QueryParams, QueryResult, QueryResultV2};
use crate::processing;

/// 查询服务
pub struct QueryService {
    source: SqlServerSource,
    cache: Arc<QueryCache>,
    default_table: String,
}

impl QueryService {
    /// 创建新的查询服务
    pub fn new(pool: Arc<ConnectionPool>, cache: Arc<QueryCache>, default_table: String) -> Self {
        let source = SqlServerSource::from_pool(pool);
        Self {
            source,
            cache,
            default_table,
        }
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &Arc<ConnectionPool> {
        self.source.pool()
    }

    /// 获取默认表名
    pub fn default_table(&self) -> &str {
        &self.default_table
    }

    /// 获取可用标签列表
    pub async fn get_available_tags(&self) -> AppResult<Vec<String>> {
        self.source.get_available_tags(&self.default_table).await
    }

    /// 搜索标签
    pub async fn search_tags(&self, keyword: &str, limit: usize) -> AppResult<Vec<String>> {
        self.source.search_tags(keyword, limit).await
    }

    /// 测试连接
    pub async fn test_connection(&self) -> AppResult<()> {
        self.source.test_connection().await
    }

    /// 查询历史数据 (V1 格式)
    pub async fn query_history(
        &self,
        params: &QueryParams,
        processing_config: Option<&DataProcessingConfig>,
        force_refresh: bool,
    ) -> AppResult<QueryResult> {
        let tags_ref = params.tags.as_deref();

        // 构建缓存键
        let cache_key = CacheKey::new(
            &self.default_table,
            &params.start_time,
            &params.end_time,
            tags_ref,
            processing_config,
        );

        // 检查缓存（非强制刷新时）
        if !force_refresh {
            if let Some(cached_records) = self.cache.get(&cache_key).await {
                info!(target: "industry_vis::query_service",
                    "缓存命中，返回 {} 条记录", cached_records.len()
                );

                let total = cached_records.len();
                let records = apply_pagination(cached_records, params.offset, params.limit);
                return Ok(QueryResult { records, total });
            }
        }

        // 从数据库查询
        let records = self
            .source
            .query_history(
                &self.default_table,
                &params.start_time,
                &params.end_time,
                tags_ref,
            )
            .await?;

        let total = records.len();
        info!(target: "industry_vis::query_service", "查询到 {} 条原始记录", total);

        // 数据处理
        let processed_records = processing::process_query_result(records, processing_config)?;

        // 存入缓存
        self.cache.put(cache_key, processed_records.clone()).await;

        // 应用分页
        let records = apply_pagination(processed_records, params.offset, params.limit);

        Ok(QueryResult { records, total })
    }

    /// 查询历史数据 V2 (预分组格式)
    pub async fn query_history_v2(
        &self,
        params: &QueryParams,
        processing_config: Option<&DataProcessingConfig>,
        force_refresh: bool,
    ) -> AppResult<QueryResultV2> {
        let start_time = Instant::now();
        let tags_ref = params.tags.as_deref();

        // 构建缓存键
        let cache_key = CacheKey::new(
            &self.default_table,
            &params.start_time,
            &params.end_time,
            tags_ref,
            processing_config,
        );

        // 检查缓存
        if !force_refresh {
            if let Some(cached_records) = self.cache.get(&cache_key).await {
                let query_time_ms = start_time.elapsed().as_millis() as u64;
                let total_processed = cached_records.len();

                info!(target: "industry_vis::query_service",
                    "缓存命中 V2，返回 {} 条记录，耗时 {}ms",
                    total_processed, query_time_ms
                );

                let series = processing::records_to_series(&cached_records);
                return Ok(QueryResultV2 {
                    series,
                    total_raw: total_processed,
                    total_processed,
                    cache_hit: true,
                    query_time_ms,
                });
            }
        }

        // 从数据库查询
        let records = self
            .source
            .query_history(
                &self.default_table,
                &params.start_time,
                &params.end_time,
                tags_ref,
            )
            .await?;

        let total_raw = records.len();
        info!(target: "industry_vis::query_service", "查询到 {} 条原始记录", total_raw);

        // 数据处理
        let processed_records = processing::process_query_result(records, processing_config)?;
        let total_processed = processed_records.len();

        // 存入缓存
        self.cache.put(cache_key, processed_records.clone()).await;

        // 转换为 series 格式
        let series = processing::records_to_series(&processed_records);
        let query_time_ms = start_time.elapsed().as_millis() as u64;

        info!(target: "industry_vis::query_service",
            "处理后返回 {} 条记录，{} 个系列，耗时 {}ms",
            total_processed, series.len(), query_time_ms
        );

        Ok(QueryResultV2 {
            series,
            total_raw,
            total_processed,
            cache_hit: false,
            query_time_ms,
        })
    }
}

/// 应用分页参数
fn apply_pagination(
    records: Vec<HistoryRecord>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Vec<HistoryRecord> {
    match (offset, limit) {
        (Some(offset), Some(limit)) => records.into_iter().skip(offset).take(limit).collect(),
        (Some(offset), None) => records.into_iter().skip(offset).collect(),
        (None, Some(limit)) => records.into_iter().take(limit).collect(),
        (None, None) => records,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HistoryRecord;

    #[test]
    fn test_apply_pagination() {
        let records: Vec<HistoryRecord> = (0..10)
            .map(|i| {
                HistoryRecord::new(
                    format!("2024-01-01T00:{:02}:00", i),
                    "Tag1".to_string(),
                    i as f64,
                    "Good".to_string(),
                )
            })
            .collect();

        // 无分页
        let result = apply_pagination(records.clone(), None, None);
        assert_eq!(result.len(), 10);

        // 只有 limit
        let result = apply_pagination(records.clone(), None, Some(5));
        assert_eq!(result.len(), 5);

        // 只有 offset
        let result = apply_pagination(records.clone(), Some(3), None);
        assert_eq!(result.len(), 7);

        // 有 offset 和 limit
        let result = apply_pagination(records.clone(), Some(2), Some(3));
        assert_eq!(result.len(), 3);
    }
}
