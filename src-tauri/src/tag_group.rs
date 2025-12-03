//! 标签分组管理模块
//! 
//! 支持将多个标签组合成一个分组，每个分组包含多个图表。
//! 分组配置保存在 `tag_groups.toml` 文件中。
//!
//! ## 数据结构
//! - TagGroup: 分组，包含最多 10 个图表
//! - ChartConfig: 图表配置，包含最多 5 个标签

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::models::DataProcessingConfig;
use tracing::{info, debug};

/// 图表配置（分组内的单个图表）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartConfig {
    /// 图表唯一标识符
    pub id: String,
    /// 图表名称
    pub name: String,
    /// 包含的标签列表（最多 5 个）
    #[serde(default)]
    pub tags: Vec<String>,
}

impl ChartConfig {
    /// 创建新图表
    pub fn new(name: String) -> Self {
        Self {
            id: format!("c{}", Local::now().timestamp_millis()),
            name,
            tags: Vec::new(),
        }
    }
    
    /// 验证图表配置
    pub fn validate(&self) -> AppResult<()> {
        if self.tags.len() > 5 {
            return Err(AppError::Validation(
                format!("图表 '{}' 最多包含 5 个标签，当前 {}", self.name, self.tags.len())
            ));
        }
        Ok(())
    }
}

/// 标签分组（包含多个图表）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagGroup {
    /// 唯一标识符
    pub id: String,
    /// 分组名称
    pub name: String,
    /// 包含的图表列表（最多 10 个）
    #[serde(default)]
    pub charts: Vec<ChartConfig>,
    /// 数据处理配置
    #[serde(default)]
    pub processing_config: DataProcessingConfig,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl TagGroup {
    /// 创建新分组（允许空图表列表）
    pub fn new(name: String, charts: Vec<ChartConfig>) -> AppResult<Self> {
        if name.trim().is_empty() {
            return Err(AppError::Validation(
                "分组名称不能为空".to_string()
            ));
        }
        
        if charts.len() > 10 {
            return Err(AppError::Validation(
                "每个分组最多包含 10 个图表".to_string()
            ));
        }
        
        // 验证每个图表
        for chart in &charts {
            chart.validate()?;
        }
        
        let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let id = format!("g{}", Local::now().timestamp_millis());
        
        Ok(Self {
            id,
            name: name.trim().to_string(),
            charts,
            processing_config: DataProcessingConfig::default(),
            created_at: now.clone(),
            updated_at: now,
        })
    }
    
    /// 更新分组
    pub fn update(
        &mut self, 
        name: String, 
        charts: Vec<ChartConfig>,
        processing_config: Option<DataProcessingConfig>,
    ) -> AppResult<()> {
        if name.trim().is_empty() {
            return Err(AppError::Validation(
                "分组名称不能为空".to_string()
            ));
        }
        
        if charts.len() > 10 {
            return Err(AppError::Validation(
                "每个分组最多包含 10 个图表".to_string()
            ));
        }
        
        // 验证每个图表
        for chart in &charts {
            chart.validate()?;
        }
        
        self.name = name.trim().to_string();
        self.charts = charts;
        if let Some(config) = processing_config {
            self.processing_config = config;
        }
        self.updated_at = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        
        Ok(())
    }
    
    /// 获取所有图表中的标签（去重）
    #[allow(dead_code)]
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.charts
            .iter()
            .flat_map(|c| c.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }
}

/// 标签分组配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagGroupConfig {
    /// 配置文件版本
    pub version: u32,
    /// 所有分组
    #[serde(default)]
    pub groups: Vec<TagGroup>,
}

impl Default for TagGroupConfig {
    fn default() -> Self {
        Self {
            version: 1,
            groups: Vec::new(),
        }
    }
}

impl TagGroupConfig {
    /// 配置文件名（v2 版本）
    const CONFIG_FILENAME: &'static str = "tag_groups_v2.toml";
    /// 旧版配置文件名
    const LEGACY_CONFIG_FILENAME: &'static str = "tag_groups.toml";
    
    /// 获取 exe 同目录的配置路径（便携模式）
    fn portable_config_path() -> Option<PathBuf> {
        std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .map(|d| d.join(Self::CONFIG_FILENAME))
    }

    /// 获取 AppData 目录的配置路径（安装模式）
    fn appdata_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("IndustryVis").join(Self::CONFIG_FILENAME))
    }
    
    /// 获取 exe 同目录的旧版配置文件路径（便携模式）
    fn legacy_portable_config_path() -> Option<PathBuf> {
        std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .map(|d| d.join(Self::LEGACY_CONFIG_FILENAME))
    }
    
    /// 获取 AppData 目录的旧版配置文件路径（安装模式）
    fn legacy_appdata_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("IndustryVis").join(Self::LEGACY_CONFIG_FILENAME))
    }
    
    /// 获取旧版配置文件路径（优先检查 exe 目录，然后 AppData）
    fn legacy_config_path() -> Option<PathBuf> {
        // 优先检查 exe 同目录
        if let Some(path) = Self::legacy_portable_config_path() {
            if path.exists() {
                return Some(path);
            }
        }
        // 然后检查 AppData 目录
        if let Some(path) = Self::legacy_appdata_config_path() {
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// 获取配置文件路径
    pub fn config_path() -> AppResult<PathBuf> {
        // 优先检查 exe 同目录是否已有配置（便携模式）
        if let Some(portable_path) = Self::portable_config_path() {
            if portable_path.exists() {
                return Ok(portable_path);
            }
        }

        // 检查 AppData 是否已有配置
        if let Some(appdata_path) = Self::appdata_config_path() {
            if appdata_path.exists() {
                return Ok(appdata_path);
            }
        }

        // 都不存在时，默认使用 exe 同目录
        Self::portable_config_path()
            .ok_or_else(|| AppError::Config("无法确定配置文件路径".to_string()))
    }

    /// 获取保存配置的路径
    fn save_config_path() -> AppResult<PathBuf> {
        // 优先尝试 exe 同目录（便携模式）
        if let Some(portable_path) = Self::portable_config_path() {
            if let Some(parent) = portable_path.parent() {
                let test_file = parent.join(".tag_groups_write_test");
                if fs::write(&test_file, "test").is_ok() {
                    let _ = fs::remove_file(&test_file);
                    return Ok(portable_path);
                }
            }
        }

        // 不可写时使用 AppData
        if let Some(appdata_path) = Self::appdata_config_path() {
            if let Some(parent) = appdata_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return Ok(appdata_path);
        }

        Err(AppError::Config("无法找到可写的配置文件路径".to_string()))
    }
    
    /// 从文件加载配置
    pub fn load() -> AppResult<Self> {
        let path = Self::config_path()?;
        info!(target: "industry_vis_lib::tag_group", "加载标签分组配置: {:?}", path);
        
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: TagGroupConfig = toml::from_str(&content)?;
            info!(target: "industry_vis_lib::tag_group", "加载了 {} 个分组", config.groups.len());
            Ok(config)
        } else {
            // 尝试从旧版配置迁移
            if let Some(migrated) = Self::migrate_from_legacy()? {
                info!(target: "industry_vis_lib::tag_group", "从旧版配置迁移了 {} 个分组", migrated.groups.len());
                migrated.save()?;
                return Ok(migrated);
            }
            info!(target: "industry_vis_lib::tag_group", "配置文件不存在，使用默认配置");
            Ok(Self::default())
        }
    }
    
    /// 从旧版配置迁移
    fn migrate_from_legacy() -> AppResult<Option<Self>> {
        let legacy_path = match Self::legacy_config_path() {
            Some(p) => p,  // legacy_config_path() 只在文件存在时才返回 Some
            None => return Ok(None),
        };
        
        info!(target: "industry_vis_lib::tag_group", "发现旧版配置文件: {:?}", legacy_path);
        
        // 旧版数据结构
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyTagGroup {
            id: String,
            name: String,
            #[serde(default)]
            tags: Vec<String>,
            #[serde(default)]
            processing_config: DataProcessingConfig,
            created_at: String,
            updated_at: String,
        }
        
        #[derive(Deserialize)]
        struct LegacyConfig {
            #[serde(default)]
            groups: Vec<LegacyTagGroup>,
        }
        
        let content = fs::read_to_string(&legacy_path)?;
        let legacy: LegacyConfig = toml::from_str(&content)?;
        
        // 转换为新结构
        let groups: Vec<TagGroup> = legacy.groups.into_iter().map(|g| {
            // 将旧的 tags 转换为默认图表
            let charts = if g.tags.is_empty() {
                vec![ChartConfig::new("默认图表".to_string())]
            } else {
                vec![ChartConfig {
                    id: format!("c{}", Local::now().timestamp_millis()),
                    name: "默认图表".to_string(),
                    tags: g.tags,
                }]
            };
            
            TagGroup {
                id: g.id,
                name: g.name,
                charts,
                processing_config: g.processing_config,
                created_at: g.created_at,
                updated_at: g.updated_at,
            }
        }).collect();
        
        Ok(Some(TagGroupConfig {
            version: 1,
            groups,
        }))
    }
    
    /// 保存配置到文件
    pub fn save(&self) -> AppResult<()> {
        let path = Self::save_config_path()?;
        info!(target: "industry_vis_lib::tag_group", "保存标签分组配置: {:?}, 分组数: {}", path, self.groups.len());
        let content = toml::to_string_pretty(self)?;
        debug!(target: "industry_vis_lib::tag_group", "配置内容:\n{}", content);
        fs::write(&path, content)?;
        Ok(())
    }
    
    /// 获取所有分组
    #[allow(dead_code)]
    pub fn list_groups(&self) -> &[TagGroup] {
        &self.groups
    }
    
    /// 根据 ID 获取分组
    #[allow(dead_code)]
    pub fn get_group(&self, id: &str) -> Option<&TagGroup> {
        self.groups.iter().find(|g| g.id == id)
    }
    
    /// 根据 ID 获取可变分组
    pub fn get_group_mut(&mut self, id: &str) -> Option<&mut TagGroup> {
        self.groups.iter_mut().find(|g| g.id == id)
    }
    
    /// 创建新分组
    pub fn create_group(&mut self, name: String, charts: Vec<ChartConfig>) -> AppResult<TagGroup> {
        // 检查名称是否重复
        if self.groups.iter().any(|g| g.name == name.trim()) {
            return Err(AppError::Validation(
                format!("分组名称 '{}' 已存在", name.trim())
            ));
        }
        
        let group = TagGroup::new(name, charts)?;
        let result = group.clone();
        self.groups.push(group);
        self.save()?;
        
        Ok(result)
    }
    
    /// 更新分组
    pub fn update_group(
        &mut self, 
        id: &str, 
        name: String, 
        charts: Vec<ChartConfig>,
        processing_config: Option<DataProcessingConfig>,
    ) -> AppResult<TagGroup> {
        // 检查名称是否与其他分组重复
        if self.groups.iter().any(|g| g.id != id && g.name == name.trim()) {
            return Err(AppError::Validation(
                format!("分组名称 '{}' 已存在", name.trim())
            ));
        }
        
        let group = self.get_group_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("分组 '{}' 不存在", id)))?;
        
        group.update(name, charts, processing_config)?;
        let result = group.clone();
        self.save()?;
        
        Ok(result)
    }
    
    /// 删除分组
    pub fn delete_group(&mut self, id: &str) -> AppResult<()> {
        let idx = self.groups.iter().position(|g| g.id == id)
            .ok_or_else(|| AppError::NotFound(format!("分组 '{}' 不存在", id)))?;
        
        self.groups.remove(idx);
        self.save()?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_chart_config() {
        let chart = ChartConfig::new("温度对比".to_string());
        assert_eq!(chart.name, "温度对比");
        assert!(chart.id.starts_with("c"));
        assert!(chart.tags.is_empty());
    }
    
    #[test]
    fn test_create_tag_group() {
        let chart = ChartConfig::new("图表1".to_string());
        let group = TagGroup::new(
            "测试分组".to_string(),
            vec![chart]
        ).unwrap();
        
        assert_eq!(group.name, "测试分组");
        assert_eq!(group.charts.len(), 1);
        assert!(group.id.starts_with("g"));
    }
    
    #[test]
    fn test_chart_tag_limit() {
        let mut chart = ChartConfig::new("超限图表".to_string());
        chart.tags = (0..6).map(|i| format!("tag{}", i)).collect();
        
        assert!(chart.validate().is_err());
    }
    
    #[test]
    fn test_group_chart_limit() {
        let charts: Vec<ChartConfig> = (0..11)
            .map(|i| ChartConfig::new(format!("图表{}", i)))
            .collect();
        let result = TagGroup::new("超限分组".to_string(), charts);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_name() {
        let result = TagGroup::new("  ".to_string(), vec![]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_all_tags() {
        let mut chart1 = ChartConfig::new("图表1".to_string());
        chart1.tags = vec!["tag1".to_string(), "tag2".to_string()];
        
        let mut chart2 = ChartConfig::new("图表2".to_string());
        chart2.tags = vec!["tag2".to_string(), "tag3".to_string()];
        
        let group = TagGroup::new("测试".to_string(), vec![chart1, chart2]).unwrap();
        let all_tags = group.all_tags();
        
        assert_eq!(all_tags, vec!["tag1", "tag2", "tag3"]);
    }
}
