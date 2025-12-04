//! 标签分组服务

use parking_lot::RwLock;
use std::sync::Arc;
use tracing::info;

use crate::config::TagGroupConfigManager;
use crate::error::AppResult;
use crate::models::{ChartConfig, DataProcessingConfig, TagGroup};

/// 标签分组服务
pub struct TagGroupService {
    manager: Arc<RwLock<TagGroupConfigManager>>,
}

impl TagGroupService {
    /// 创建新的服务
    pub fn new(manager: Arc<RwLock<TagGroupConfigManager>>) -> Self {
        Self { manager }
    }

    /// 获取所有分组
    pub fn list_groups(&self) -> Vec<TagGroup> {
        self.manager.read().list_groups().to_vec()
    }

    /// 获取指定分组
    pub fn get_group(&self, id: &str) -> Option<TagGroup> {
        self.manager.read().get_group(id).cloned()
    }

    /// 创建分组
    pub fn create_group(&self, name: String, charts: Vec<ChartConfig>) -> AppResult<TagGroup> {
        info!(target: "industry_vis::tag_group_service",
            "创建分组 - 名称: {}, 图表数: {}", name, charts.len()
        );
        self.manager.write().create_group(name, charts)
    }

    /// 更新分组
    pub fn update_group(
        &self,
        id: &str,
        name: String,
        charts: Vec<ChartConfig>,
        processing_config: Option<DataProcessingConfig>,
    ) -> AppResult<TagGroup> {
        info!(target: "industry_vis::tag_group_service",
            "更新分组 - ID: {}, 名称: {}, 图表数: {}", id, name, charts.len()
        );
        self.manager
            .write()
            .update_group(id, name, charts, processing_config)
    }

    /// 删除分组
    pub fn delete_group(&self, id: &str) -> AppResult<()> {
        info!(target: "industry_vis::tag_group_service", "删除分组 - ID: {}", id);
        self.manager.write().delete_group(id)
    }
}

#[cfg(test)]
mod tests {
    // TagGroupService 的测试在集成测试中进行，因为需要文件系统
}
