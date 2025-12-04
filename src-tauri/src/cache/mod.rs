//! 缓存模块
//!
//! 提供查询结果缓存，支持 LRU 淘汰和 TTL 过期。

mod query_cache;

pub use query_cache::{CacheConfig, CacheKey, CacheStats, QueryCache};

use std::sync::Arc;

/// 可共享的缓存实例
pub type SharedCache = Arc<QueryCache>;

/// 创建带自动清理的缓存
pub fn create_cache_with_cleanup(config: CacheConfig) -> SharedCache {
    let cache = Arc::new(QueryCache::new(config));

    // 启动后台清理任务
    let cache_clone = Arc::clone(&cache);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cache_clone.evict_expired().await;
        }
    });

    cache
}

/// 使用默认配置创建缓存
pub fn create_default_cache() -> SharedCache {
    create_cache_with_cleanup(CacheConfig::default())
}
