//! 配置文件热更新监听器

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::{AppConfig, TagGroupConfigManager};
use crate::error::{AppError, AppResult};

/// 配置文件监听器
pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    /// 创建新的配置监听器
    pub fn new(
        app_config: Arc<RwLock<AppConfig>>,
        tag_group_manager: Arc<RwLock<TagGroupConfigManager>>,
    ) -> AppResult<Self> {
        // 获取需要监听的路径
        let app_config_path = AppConfig::config_path().ok();
        let tag_group_path = TagGroupConfigManager::config_path().ok();

        // 为闭包克隆路径
        let app_config_path_clone = app_config_path.clone();
        let tag_group_path_clone = tag_group_path.clone();

        let app_config_clone = Arc::clone(&app_config);
        let tag_group_clone = Arc::clone(&tag_group_manager);

        // 创建监听器
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| match result {
                Ok(event) => {
                    Self::handle_event(
                        event,
                        &app_config_clone,
                        &tag_group_clone,
                        app_config_path_clone.as_ref(),
                        tag_group_path_clone.as_ref(),
                    );
                }
                Err(e) => {
                    error!(target: "industry_vis::config_watcher", "监听错误: {}", e);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| AppError::ConfigWatch(format!("创建监听器失败: {}", e)))?;

        // 添加监听路径
        if let Some(ref path) = app_config_path {
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    watcher
                        .watch(parent, RecursiveMode::NonRecursive)
                        .map_err(|e| AppError::ConfigWatch(format!("监听应用配置失败: {}", e)))?;
                    info!(target: "industry_vis::config_watcher", "开始监听应用配置: {:?}", parent);
                }
            }
        }

        if let Some(ref path) = tag_group_path {
            if let Some(parent) = path.parent() {
                // 避免重复监听同一目录
                let is_different_dir = app_config_path
                    .as_ref()
                    .and_then(|p| p.parent())
                    .map(|p| p != parent)
                    .unwrap_or(true);

                if is_different_dir && parent.exists() {
                    watcher
                        .watch(parent, RecursiveMode::NonRecursive)
                        .map_err(|e| AppError::ConfigWatch(format!("监听分组配置失败: {}", e)))?;
                    info!(target: "industry_vis::config_watcher", "开始监听分组配置: {:?}", parent);
                }
            }
        }

        Ok(Self { _watcher: watcher })
    }

    /// 处理文件变更事件
    fn handle_event(
        event: Event,
        app_config: &Arc<RwLock<AppConfig>>,
        tag_group_manager: &Arc<RwLock<TagGroupConfigManager>>,
        app_config_path: Option<&PathBuf>,
        tag_group_path: Option<&PathBuf>,
    ) {
        use notify::EventKind;

        // 只处理修改和创建事件
        if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
            return;
        }

        for path in &event.paths {
            debug!(target: "industry_vis::config_watcher", "检测到文件变更: {:?}", path);

            // 检查是否是应用配置文件
            if let Some(app_path) = app_config_path {
                if path == app_path {
                    info!(target: "industry_vis::config_watcher", "应用配置已变更，重新加载");
                    match AppConfig::load() {
                        Ok(new_config) => {
                            *app_config.write() = new_config;
                            info!(target: "industry_vis::config_watcher", "应用配置重新加载成功");
                        }
                        Err(e) => {
                            warn!(target: "industry_vis::config_watcher", "重新加载应用配置失败: {}", e);
                        }
                    }
                }
            }

            // 检查是否是分组配置文件
            if let Some(tg_path) = tag_group_path {
                if path == tg_path {
                    info!(target: "industry_vis::config_watcher", "分组配置已变更，重新加载");
                    let mut manager = tag_group_manager.write();
                    if let Err(e) = manager.reload() {
                        warn!(target: "industry_vis::config_watcher", "重新加载分组配置失败: {}", e);
                    } else {
                        info!(target: "industry_vis::config_watcher", "分组配置重新加载成功");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // ConfigWatcher 测试需要文件系统操作，在集成测试中进行
}
