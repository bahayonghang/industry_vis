//! 应用状态管理
//!
//! 统一管理应用的共享状态。

use parking_lot::RwLock;
use std::sync::Arc;

use crate::cache::{CacheConfig, QueryCache, SharedCache};
use crate::config::ConfigState;
use crate::datasource::{
    ConnectionPool, DataSource, PoolConfig, ProfileRegistry, SchemaProfile, SqlServerSource,
};
use crate::error::AppResult;
use crate::models::{DataProcessingConfig, QueryParams, QueryResult, QueryResultV2};
use crate::processing;
use crate::services::{QueryService, TagGroupService};

/// 应用状态
pub struct AppState {
    /// 配置状态
    config: ConfigState,
    /// 查询缓存
    cache: SharedCache,
    /// 连接池
    pool: Option<Arc<ConnectionPool>>,
    /// 查询服务
    query_service: RwLock<Option<QueryService>>,
    /// 标签分组服务
    tag_group_service: TagGroupService,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new() -> AppResult<Self> {
        // 加载配置（带热更新）
        let config = ConfigState::with_hot_reload()?;

        // 创建缓存
        let cache = Arc::new(QueryCache::new(CacheConfig::default()));

        // 启动缓存自动清理
        let cache_clone = Arc::clone(&cache);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                cache_clone.evict_expired().await;
            }
        });

        // 创建标签分组服务
        let tag_group_service = TagGroupService::new(config.tag_group_manager());

        Ok(Self {
            config,
            cache,
            pool: None,
            query_service: RwLock::new(None),
            tag_group_service,
        })
    }

    /// 初始化连接池和查询服务
    pub async fn init_pool(&mut self) -> AppResult<()> {
        let db_config = self.config.database_config();
        let pool = ConnectionPool::new(db_config, PoolConfig::for_desktop()).await?;
        let pool = Arc::new(pool);

        let default_table = self.config.app_config().query.default_table.clone();
        let query_service =
            QueryService::new(Arc::clone(&pool), Arc::clone(&self.cache), default_table);

        self.pool = Some(pool);
        *self.query_service.write() = Some(query_service);

        Ok(())
    }

    /// 获取配置状态
    pub fn config(&self) -> &ConfigState {
        &self.config
    }

    /// 获取缓存
    pub fn cache(&self) -> &SharedCache {
        &self.cache
    }

    /// 获取查询服务（克隆一份引用）
    /// 如果连接池未初始化，返回 None
    pub fn query_service(&self) -> Option<QueryServiceHandle> {
        let guard = self.query_service.read();
        let service = guard.as_ref()?;

        // 从配置获取 Profile
        let profile = self.get_schema_profile();

        Some(QueryServiceHandle {
            source: SqlServerSource::from_pool_with_profile(Arc::clone(service.pool()), profile),
            cache: Arc::clone(&self.cache),
            default_table: service.default_table().to_string(),
        })
    }

    /// 获取当前配置的 Schema Profile
    fn get_schema_profile(&self) -> Arc<dyn SchemaProfile> {
        let profile_name = &self.config.app_config().schema.profile;
        ProfileRegistry::get(profile_name).unwrap_or_else(|e| {
            tracing::warn!(
                target: "industry_vis::state",
                error = %e,
                profile = %profile_name,
                "无法获取指定的 Profile，使用默认 Profile"
            );
            ProfileRegistry::default_profile()
        })
    }

    /// 检查连接池是否已初始化
    pub fn is_pool_initialized(&self) -> bool {
        self.pool.is_some()
    }

    /// 获取标签分组服务
    pub fn tag_group_service(&self) -> &TagGroupService {
        &self.tag_group_service
    }

    /// 重新初始化连接池（配置变更时）
    pub async fn reinit_pool(&mut self) -> AppResult<()> {
        self.init_pool().await
    }
}

/// 简化的应用状态（用于无需连接池的场景）
pub struct AppStateSimple {
    config: ConfigState,
    cache: SharedCache,
    tag_group_service: TagGroupService,
}

impl AppStateSimple {
    pub fn new() -> AppResult<Self> {
        let config = ConfigState::new()?;
        let cache = Arc::new(QueryCache::with_defaults());
        let tag_group_service = TagGroupService::new(config.tag_group_manager());

        Ok(Self {
            config,
            cache,
            tag_group_service,
        })
    }

    pub fn config(&self) -> &ConfigState {
        &self.config
    }

    pub fn cache(&self) -> &SharedCache {
        &self.cache
    }

    pub fn tag_group_service(&self) -> &TagGroupService {
        &self.tag_group_service
    }
}

/// 查询服务句柄（独立于 AppState 的生命周期）
pub struct QueryServiceHandle {
    source: SqlServerSource,
    cache: SharedCache,
    default_table: String,
}

impl QueryServiceHandle {
    /// 获取可用标签列表
    pub async fn get_available_tags(&self) -> AppResult<Vec<String>> {
        self.source.get_available_tags(&self.default_table).await
    }

    /// 搜索标签
    pub async fn search_tags(&self, keyword: &str, limit: usize) -> AppResult<Vec<String>> {
        self.source.search_tags(keyword, limit).await
    }

    /// 查询历史数据 (V1 格式)
    pub async fn query_history(
        &self,
        params: &QueryParams,
        processing_config: Option<&DataProcessingConfig>,
        force_refresh: bool,
    ) -> AppResult<QueryResult> {
        use crate::cache::CacheKey;

        use tracing::info;

        let tags_ref = params.tags.as_deref();

        let cache_key = CacheKey::new(
            &self.default_table,
            &params.start_time,
            &params.end_time,
            tags_ref,
            processing_config,
        );

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
        let processed_records = processing::process_query_result(records, processing_config)?;
        self.cache.put(cache_key, processed_records.clone()).await;
        let records = apply_pagination(processed_records, params.offset, params.limit);

        Ok(QueryResult { records, total })
    }

    /// 查询历史数据 V2
    pub async fn query_history_v2(
        &self,
        params: &QueryParams,
        processing_config: Option<&DataProcessingConfig>,
        force_refresh: bool,
    ) -> AppResult<QueryResultV2> {
        use crate::cache::CacheKey;
        use std::time::Instant;

        let start_time = Instant::now();
        let tags_ref = params.tags.as_deref();

        let cache_key = CacheKey::new(
            &self.default_table,
            &params.start_time,
            &params.end_time,
            tags_ref,
            processing_config,
        );

        if !force_refresh {
            if let Some(cached_records) = self.cache.get(&cache_key).await {
                let query_time_ms = start_time.elapsed().as_millis() as u64;
                let total_processed = cached_records.len();
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
        let processed_records = processing::process_query_result(records, processing_config)?;
        let total_processed = processed_records.len();
        self.cache.put(cache_key, processed_records.clone()).await;
        let series = processing::records_to_series(&processed_records);
        let query_time_ms = start_time.elapsed().as_millis() as u64;

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
    records: Vec<crate::models::HistoryRecord>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Vec<crate::models::HistoryRecord> {
    match (offset, limit) {
        (Some(offset), Some(limit)) => records.into_iter().skip(offset).take(limit).collect(),
        (Some(offset), None) => records.into_iter().skip(offset).collect(),
        (None, Some(limit)) => records.into_iter().take(limit).collect(),
        (None, None) => records,
    }
}
