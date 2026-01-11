//! 性能配置模块
//!
//! 提供可配置的性能参数，支持缓存、连接池、数据处理等配置。

use serde::{Deserialize, Serialize};
use tracing::debug;

/// 缓存性能配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CachePerformanceConfig {
    /// 最大缓存条目数
    #[serde(default = "CachePerformanceConfig::default_max_entries")]
    pub max_entries: usize,
    /// 缓存过期时间（秒）
    #[serde(default = "CachePerformanceConfig::default_ttl_seconds")]
    pub ttl_seconds: u64,
    /// 是否启用缓存预热
    #[serde(default)]
    pub warmup_enabled: bool,
}

impl CachePerformanceConfig {
    fn default_max_entries() -> usize {
        200
    }

    fn default_ttl_seconds() -> u64 {
        1800 // 30 分钟
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.max_entries < 10 {
            return Err("max_entries 最小值为 10".to_string());
        }
        if self.max_entries > 1000 {
            return Err("max_entries 最大值为 1000".to_string());
        }
        if self.ttl_seconds < 60 {
            return Err("ttl_seconds 最小值为 60 秒".to_string());
        }
        if self.ttl_seconds > 7200 {
            return Err("ttl_seconds 最大值为 7200 秒（2小时）".to_string());
        }
        Ok(())
    }
}

impl Default for CachePerformanceConfig {
    fn default() -> Self {
        Self {
            max_entries: Self::default_max_entries(),
            ttl_seconds: Self::default_ttl_seconds(),
            warmup_enabled: false,
        }
    }
}

/// 连接池性能配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PoolPerformanceConfig {
    /// 最大连接数
    #[serde(default = "PoolPerformanceConfig::default_max_size")]
    pub max_size: u32,
    /// 最小空闲连接数
    #[serde(default = "PoolPerformanceConfig::default_min_idle")]
    pub min_idle: u32,
    /// 连接超时（秒）
    #[serde(default = "PoolPerformanceConfig::default_connection_timeout_secs")]
    pub connection_timeout_secs: u64,
    /// 空闲超时（秒）
    #[serde(default = "PoolPerformanceConfig::default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
    /// 最大生命周期（秒）
    #[serde(default = "PoolPerformanceConfig::default_max_lifetime_secs")]
    pub max_lifetime_secs: u64,
}

impl PoolPerformanceConfig {
    fn default_max_size() -> u32 {
        3
    }

    fn default_min_idle() -> u32 {
        1
    }

    fn default_connection_timeout_secs() -> u64 {
        15
    }

    fn default_idle_timeout_secs() -> u64 {
        300 // 5 分钟
    }

    fn default_max_lifetime_secs() -> u64 {
        900 // 15 分钟
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.max_size < 1 {
            return Err("max_size 最小值为 1".to_string());
        }
        if self.max_size > 10 {
            return Err("max_size 最大值为 10（桌面应用场景）".to_string());
        }
        if self.min_idle > self.max_size {
            return Err("min_idle 不能大于 max_size".to_string());
        }
        if self.connection_timeout_secs < 5 {
            return Err("connection_timeout_secs 最小值为 5 秒".to_string());
        }
        if self.connection_timeout_secs > 60 {
            return Err("connection_timeout_secs 最大值为 60 秒".to_string());
        }
        Ok(())
    }
}

impl Default for PoolPerformanceConfig {
    fn default() -> Self {
        Self {
            max_size: Self::default_max_size(),
            min_idle: Self::default_min_idle(),
            connection_timeout_secs: Self::default_connection_timeout_secs(),
            idle_timeout_secs: Self::default_idle_timeout_secs(),
            max_lifetime_secs: Self::default_max_lifetime_secs(),
        }
    }
}

/// 数据处理性能配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingPerformanceConfig {
    /// 是否使用统一 LazyFrame 管道（推荐）
    #[serde(default = "ProcessingPerformanceConfig::default_use_unified_pipeline")]
    pub use_unified_pipeline: bool,
    /// 大数据集阈值（超过此值启用优化策略）
    #[serde(default = "ProcessingPerformanceConfig::default_large_dataset_threshold")]
    pub large_dataset_threshold: usize,
}

impl ProcessingPerformanceConfig {
    fn default_use_unified_pipeline() -> bool {
        true
    }

    fn default_large_dataset_threshold() -> usize {
        10000
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.large_dataset_threshold < 1000 {
            return Err("large_dataset_threshold 最小值为 1000".to_string());
        }
        Ok(())
    }
}

impl Default for ProcessingPerformanceConfig {
    fn default() -> Self {
        Self {
            use_unified_pipeline: Self::default_use_unified_pipeline(),
            large_dataset_threshold: Self::default_large_dataset_threshold(),
        }
    }
}

/// 图表渲染性能配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChartPerformanceConfig {
    /// 是否启用脏矩形渲染
    #[serde(default = "ChartPerformanceConfig::default_use_dirty_rect")]
    pub use_dirty_rect: bool,
    /// 大数据优化阈值
    #[serde(default = "ChartPerformanceConfig::default_large_threshold")]
    pub large_threshold: usize,
    /// 渐进渲染阈值
    #[serde(default = "ChartPerformanceConfig::default_progressive_threshold")]
    pub progressive_threshold: usize,
}

impl ChartPerformanceConfig {
    fn default_use_dirty_rect() -> bool {
        true
    }

    fn default_large_threshold() -> usize {
        2000
    }

    fn default_progressive_threshold() -> usize {
        3000
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.large_threshold < 500 {
            return Err("large_threshold 最小值为 500".to_string());
        }
        if self.progressive_threshold < self.large_threshold {
            return Err("progressive_threshold 应大于等于 large_threshold".to_string());
        }
        Ok(())
    }
}

impl Default for ChartPerformanceConfig {
    fn default() -> Self {
        Self {
            use_dirty_rect: Self::default_use_dirty_rect(),
            large_threshold: Self::default_large_threshold(),
            progressive_threshold: Self::default_progressive_threshold(),
        }
    }
}

/// 综合性能配置
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceConfig {
    /// 缓存配置
    #[serde(default)]
    pub cache: CachePerformanceConfig,
    /// 连接池配置
    #[serde(default)]
    pub pool: PoolPerformanceConfig,
    /// 数据处理配置
    #[serde(default)]
    pub processing: ProcessingPerformanceConfig,
    /// 图表渲染配置
    #[serde(default)]
    pub chart: ChartPerformanceConfig,
}

impl PerformanceConfig {
    /// 验证所有配置
    pub fn validate(&self) -> Result<(), String> {
        self.cache.validate()?;
        self.pool.validate()?;
        self.processing.validate()?;
        self.chart.validate()?;
        debug!(target: "industry_vis::config", "性能配置验证通过");
        Ok(())
    }

    /// 创建适合桌面应用的配置
    pub fn for_desktop() -> Self {
        Self::default()
    }

    /// 创建高性能配置（更多资源消耗）
    pub fn high_performance() -> Self {
        Self {
            cache: CachePerformanceConfig {
                max_entries: 500,
                ttl_seconds: 3600,
                warmup_enabled: true,
            },
            pool: PoolPerformanceConfig {
                max_size: 5,
                min_idle: 2,
                connection_timeout_secs: 10,
                idle_timeout_secs: 600,
                max_lifetime_secs: 1800,
            },
            processing: ProcessingPerformanceConfig {
                use_unified_pipeline: true,
                large_dataset_threshold: 5000,
            },
            chart: ChartPerformanceConfig {
                use_dirty_rect: true,
                large_threshold: 1000,
                progressive_threshold: 2000,
            },
        }
    }

    /// 创建低资源配置（节省内存）
    pub fn low_resource() -> Self {
        Self {
            cache: CachePerformanceConfig {
                max_entries: 50,
                ttl_seconds: 600,
                warmup_enabled: false,
            },
            pool: PoolPerformanceConfig {
                max_size: 1,
                min_idle: 0,
                connection_timeout_secs: 30,
                idle_timeout_secs: 120,
                max_lifetime_secs: 300,
            },
            processing: ProcessingPerformanceConfig {
                use_unified_pipeline: true,
                large_dataset_threshold: 20000,
            },
            chart: ChartPerformanceConfig {
                use_dirty_rect: true,
                large_threshold: 5000,
                progressive_threshold: 8000,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_performance_config() {
        let config = PerformanceConfig::default();
        assert_eq!(config.cache.max_entries, 200);
        assert_eq!(config.cache.ttl_seconds, 1800);
        assert_eq!(config.pool.max_size, 3);
        assert_eq!(config.pool.connection_timeout_secs, 15);
        assert!(config.processing.use_unified_pipeline);
        assert!(config.chart.use_dirty_rect);
    }

    #[test]
    fn test_cache_config_validation() {
        let mut config = CachePerformanceConfig::default();
        assert!(config.validate().is_ok());

        config.max_entries = 5;
        assert!(config.validate().is_err());

        config.max_entries = 100;
        config.ttl_seconds = 30;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_pool_config_validation() {
        let mut config = PoolPerformanceConfig::default();
        assert!(config.validate().is_ok());

        config.max_size = 0;
        assert!(config.validate().is_err());

        config.max_size = 3;
        config.min_idle = 5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_high_performance_preset() {
        let config = PerformanceConfig::high_performance();
        assert_eq!(config.cache.max_entries, 500);
        assert_eq!(config.pool.max_size, 5);
        assert!(config.cache.warmup_enabled);
    }

    #[test]
    fn test_low_resource_preset() {
        let config = PerformanceConfig::low_resource();
        assert_eq!(config.cache.max_entries, 50);
        assert_eq!(config.pool.max_size, 1);
        assert!(!config.cache.warmup_enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = PerformanceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: PerformanceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }
}
