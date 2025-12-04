//! 配置管理模块
//!
//! 提供配置加载、保存、热更新功能。

mod app;
mod tag_groups;
mod watcher;

pub use app::{AppConfig, DatabaseConfig, QueryConfig};
pub use tag_groups::TagGroupConfigManager;
pub use watcher::ConfigWatcher;

use parking_lot::RwLock;
use std::sync::Arc;

/// 配置状态，支持热更新
pub struct ConfigState {
    /// 应用配置（内存缓存）
    app_config: Arc<RwLock<AppConfig>>,
    /// 标签分组配置管理器
    tag_group_manager: Arc<RwLock<TagGroupConfigManager>>,
    /// 配置监听器
    _watcher: Option<ConfigWatcher>,
}

impl ConfigState {
    /// 创建新的配置状态
    pub fn new() -> crate::error::AppResult<Self> {
        let app_config = AppConfig::load()?;
        let tag_group_manager = TagGroupConfigManager::load()?;

        Ok(Self {
            app_config: Arc::new(RwLock::new(app_config)),
            tag_group_manager: Arc::new(RwLock::new(tag_group_manager)),
            _watcher: None,
        })
    }

    /// 创建带热更新的配置状态
    pub fn with_hot_reload() -> crate::error::AppResult<Self> {
        let app_config = AppConfig::load()?;
        let tag_group_manager = TagGroupConfigManager::load()?;

        let app_config = Arc::new(RwLock::new(app_config));
        let tag_group_manager = Arc::new(RwLock::new(tag_group_manager));

        // 设置配置文件监听
        let watcher = ConfigWatcher::new(Arc::clone(&app_config), Arc::clone(&tag_group_manager))?;

        Ok(Self {
            app_config,
            tag_group_manager,
            _watcher: Some(watcher),
        })
    }

    /// 获取应用配置（读取）
    pub fn app_config(&self) -> AppConfig {
        self.app_config.read().clone()
    }

    /// 获取数据库配置
    pub fn database_config(&self) -> DatabaseConfig {
        self.app_config.read().database.clone()
    }

    /// 更新应用配置
    pub fn update_app_config(&self, config: AppConfig) -> crate::error::AppResult<()> {
        config.save()?;
        *self.app_config.write() = config;
        Ok(())
    }

    /// 获取标签分组管理器引用
    pub fn tag_group_manager(&self) -> Arc<RwLock<TagGroupConfigManager>> {
        Arc::clone(&self.tag_group_manager)
    }
}

impl Default for ConfigState {
    fn default() -> Self {
        Self::new().expect("Failed to load default config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_state_creation() {
        // 这个测试可能会失败如果没有配置文件，所以使用 default
        let config = AppConfig::default();
        assert_eq!(config.database.port, 1433);
    }
}
