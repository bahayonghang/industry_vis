//! 查询缓存模块
//!
//! 使用 LRU 缓存 + TTL 过期策略缓存查询结果。
//! 适合桌面应用场景，避免重复查询数据库。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use lru::LruCache;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::models::{DataProcessingConfig, HistoryRecord};

/// 缓存配置
#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 缓存过期时间（秒）
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 50,      // 最多缓存 50 个查询结果
            ttl_seconds: 300,     // 5 分钟过期
        }
    }
}

/// 缓存键
///
/// 基于表名、时间范围、标签列表、处理配置生成唯一键
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub table: String,
    pub start_time: String,
    pub end_time: String,
    pub tags: Vec<String>,           // 已排序
    pub processing_config_hash: u64, // 处理配置的哈希值
}

impl CacheKey {
    /// 创建缓存键
    ///
    /// 标签列表会自动排序，确保顺序无关
    pub fn new(
        table: &str,
        start_time: &str,
        end_time: &str,
        tags: Option<&[String]>,
        processing_config: Option<&DataProcessingConfig>,
    ) -> Self {
        let mut sorted_tags: Vec<String> = tags
            .map(|t| t.to_vec())
            .unwrap_or_default();
        sorted_tags.sort();

        let processing_config_hash = processing_config
            .map(|c| {
                let mut hasher = DefaultHasher::new();
                // 哈希 outlier_removal 配置
                c.outlier_removal.enabled.hash(&mut hasher);
                c.outlier_removal.method.hash(&mut hasher);
                // 哈希 resample 配置
                c.resample.enabled.hash(&mut hasher);
                c.resample.interval.hash(&mut hasher);
                c.resample.method.hash(&mut hasher);
                // 哈希 smoothing 配置
                c.smoothing.enabled.hash(&mut hasher);
                c.smoothing.method.hash(&mut hasher);
                c.smoothing.window.hash(&mut hasher);
                hasher.finish()
            })
            .unwrap_or(0);

        Self {
            table: table.to_string(),
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
            tags: sorted_tags,
            processing_config_hash,
        }
    }
}

/// 缓存条目
struct CacheEntry {
    data: Vec<HistoryRecord>,
    created_at: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn new(data: Vec<HistoryRecord>, ttl: Duration) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// 缓存统计信息
#[derive(Clone, Debug, serde::Serialize)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 命中率（百分比）
    pub hit_rate: f64,
    /// 当前缓存条目数
    pub entries: usize,
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 估计内存使用（字节）
    pub estimated_memory_bytes: usize,
}

/// 查询结果缓存
///
/// 线程安全的 LRU 缓存，支持 TTL 过期
pub struct QueryCache {
    cache: Arc<RwLock<LruCache<CacheKey, CacheEntry>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStatsInternal>>,
}

struct CacheStatsInternal {
    hits: u64,
    misses: u64,
}

impl QueryCache {
    /// 创建新的查询缓存
    pub fn new(config: CacheConfig) -> Self {
        let cache = LruCache::new(
            std::num::NonZeroUsize::new(config.max_entries).unwrap_or(
                std::num::NonZeroUsize::new(50).unwrap()
            )
        );
        
        Self {
            cache: Arc::new(RwLock::new(cache)),
            config,
            stats: Arc::new(RwLock::new(CacheStatsInternal { hits: 0, misses: 0 })),
        }
    }

    /// 使用默认配置创建缓存
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// 获取缓存数据
    ///
    /// 如果缓存命中且未过期，返回 Some(data)；否则返回 None
    pub async fn get(&self, key: &CacheKey) -> Option<Vec<HistoryRecord>> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get(key) {
            if entry.is_expired() {
                // 过期了，移除并返回 None
                cache.pop(key);
                let mut stats = self.stats.write().await;
                stats.misses += 1;
                debug!(target: "industry_vis_lib::cache", 
                    "缓存过期 - table={}, tags={:?}", 
                    key.table, key.tags
                );
                None
            } else {
                // 命中
                let mut stats = self.stats.write().await;
                stats.hits += 1;
                debug!(target: "industry_vis_lib::cache", 
                    "缓存命中 - table={}, tags={:?}, records={}", 
                    key.table, key.tags, entry.data.len()
                );
                Some(entry.data.clone())
            }
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            debug!(target: "industry_vis_lib::cache", 
                "缓存未命中 - table={}, tags={:?}", 
                key.table, key.tags
            );
            None
        }
    }

    /// 存入缓存
    pub async fn put(&self, key: CacheKey, data: Vec<HistoryRecord>) {
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let entry = CacheEntry::new(data.clone(), ttl);
        
        let mut cache = self.cache.write().await;
        cache.put(key.clone(), entry);
        
        debug!(target: "industry_vis_lib::cache", 
            "缓存写入 - table={}, tags={:?}, records={}", 
            key.table, key.tags, data.len()
        );
    }

    /// 清空所有缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        
        let mut stats = self.stats.write().await;
        stats.hits = 0;
        stats.misses = 0;
        
        info!(target: "industry_vis_lib::cache", "缓存已清空");
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let stats = self.stats.read().await;
        
        let total = stats.hits + stats.misses;
        let hit_rate = if total > 0 {
            (stats.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // 估算内存使用
        // 每条 HistoryRecord 大约 100 字节（包括字符串分配）
        let estimated_memory_bytes = cache.iter()
            .map(|(_, entry)| entry.data.len() * 100)
            .sum();

        CacheStats {
            hits: stats.hits,
            misses: stats.misses,
            hit_rate,
            entries: cache.len(),
            max_entries: self.config.max_entries,
            estimated_memory_bytes,
        }
    }

    /// 移除过期条目
    ///
    /// 遍历缓存并移除所有过期条目
    pub async fn evict_expired(&self) {
        let mut cache = self.cache.write().await;
        let keys_to_remove: Vec<CacheKey> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
        
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            cache.pop(&key);
        }
        
        if count > 0 {
            debug!(target: "industry_vis_lib::cache", "清理 {} 个过期条目", count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let key1 = CacheKey::new(
            "History",
            "2024-01-01",
            "2024-01-02",
            Some(&["tag1".to_string(), "tag2".to_string()]),
            None,
        );
        
        // 标签顺序不同，但排序后应该相等
        let key2 = CacheKey::new(
            "History",
            "2024-01-01",
            "2024-01-02",
            Some(&["tag2".to_string(), "tag1".to_string()]),
            None,
        );
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_configs() {
        use crate::models::{OutlierRemovalConfig, ResampleConfig, SmoothingConfig};
        
        let config1 = DataProcessingConfig {
            outlier_removal: OutlierRemovalConfig {
                enabled: true,
                method: "3sigma".to_string(),
            },
            resample: ResampleConfig::default(),
            smoothing: SmoothingConfig::default(),
        };
        
        let config2 = DataProcessingConfig {
            outlier_removal: OutlierRemovalConfig {
                enabled: false,
                method: "3sigma".to_string(),
            },
            resample: ResampleConfig::default(),
            smoothing: SmoothingConfig::default(),
        };
        
        let key1 = CacheKey::new("History", "2024-01-01", "2024-01-02", None, Some(&config1));
        let key2 = CacheKey::new("History", "2024-01-01", "2024-01-02", None, Some(&config2));
        
        assert_ne!(key1, key2);
    }

    #[tokio::test]
    async fn test_cache_put_get() {
        let cache = QueryCache::with_defaults();
        let key = CacheKey::new("History", "2024-01-01", "2024-01-02", None, None);
        
        let records = vec![
            HistoryRecord {
                date_time: "2024-01-01T00:00:00".to_string(),
                tag_name: "tag1".to_string(),
                tag_val: 1.0,
                tag_quality: "Good".to_string(),
            },
        ];
        
        cache.put(key.clone(), records.clone()).await;
        
        let result = cache.get(&key).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = QueryCache::with_defaults();
        let key = CacheKey::new("History", "2024-01-01", "2024-01-02", None, None);
        
        cache.put(key.clone(), vec![]).await;
        cache.clear().await;
        
        let result = cache.get(&key).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = QueryCache::with_defaults();
        let key = CacheKey::new("History", "2024-01-01", "2024-01-02", None, None);
        
        // Miss
        let _ = cache.get(&key).await;
        
        // Put
        cache.put(key.clone(), vec![]).await;
        
        // Hit
        let _ = cache.get(&key).await;
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 50.0);
    }

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        // 创建一个只能存 3 个条目的缓存
        let config = CacheConfig {
            max_entries: 3,
            ttl_seconds: 300,
        };
        let cache = QueryCache::new(config);
        
        // 添加 4 个条目，第一个应该被淘汰
        for i in 0..4 {
            let key = CacheKey::new(
                "History",
                &format!("2024-01-0{}", i + 1),
                "2024-01-10",
                None,
                None,
            );
            cache.put(key, vec![]).await;
        }
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.entries, 3, "缓存应该只有 3 个条目");
        
        // 第一个键应该被淘汰
        let first_key = CacheKey::new("History", "2024-01-01", "2024-01-10", None, None);
        let result = cache.get(&first_key).await;
        assert!(result.is_none(), "第一个条目应该被 LRU 淘汰");
        
        // 最后一个键应该存在
        let last_key = CacheKey::new("History", "2024-01-04", "2024-01-10", None, None);
        let result = cache.get(&last_key).await;
        assert!(result.is_some(), "最后一个条目应该存在");
    }

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        // 创建一个 TTL 为 1 秒的缓存
        let config = CacheConfig {
            max_entries: 10,
            ttl_seconds: 1,
        };
        let cache = QueryCache::new(config);
        
        let key = CacheKey::new("History", "2024-01-01", "2024-01-02", None, None);
        cache.put(key.clone(), vec![
            HistoryRecord {
                date_time: "2024-01-01T00:00:00".to_string(),
                tag_name: "tag1".to_string(),
                tag_val: 1.0,
                tag_quality: "Good".to_string(),
            },
        ]).await;
        
        // 立即获取应该命中
        let result = cache.get(&key).await;
        assert!(result.is_some(), "立即获取应该命中缓存");
        
        // 等待 TTL 过期
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // 过期后应该未命中
        let result = cache.get(&key).await;
        assert!(result.is_none(), "TTL 过期后应该未命中缓存");
    }

    #[tokio::test]
    async fn test_cache_evict_expired() {
        let config = CacheConfig {
            max_entries: 10,
            ttl_seconds: 1,
        };
        let cache = QueryCache::new(config);
        
        // 添加多个条目
        for i in 0..3 {
            let key = CacheKey::new(
                "History",
                &format!("2024-01-0{}", i + 1),
                "2024-01-10",
                None,
                None,
            );
            cache.put(key, vec![]).await;
        }
        
        let stats_before = cache.get_stats().await;
        assert_eq!(stats_before.entries, 3);
        
        // 等待过期
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // 清理过期条目
        cache.evict_expired().await;
        
        let stats_after = cache.get_stats().await;
        assert_eq!(stats_after.entries, 0, "所有过期条目应该被清理");
    }
}
