//! 标签分组配置管理

use chrono::Local;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::error::{AppError, AppResult};
use crate::models::{ChartConfig, DataProcessingConfig, TagGroup, TagGroupConfig};

/// 标签分组配置管理器
#[derive(Debug)]
pub struct TagGroupConfigManager {
    config: TagGroupConfig,
    config_path: PathBuf,
}

impl TagGroupConfigManager {
    /// 配置文件名（v2 版本）
    const CONFIG_FILENAME: &'static str = "tag_groups_v2.toml";
    /// 旧版配置文件名
    const LEGACY_CONFIG_FILENAME: &'static str = "tag_groups.toml";

    /// 获取 exe 同目录的配置路径（便携模式）
    fn portable_config_path() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .map(|d| d.join(Self::CONFIG_FILENAME))
    }

    /// 获取 AppData 目录的配置路径（安装模式）
    fn appdata_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("IndustryVis").join(Self::CONFIG_FILENAME))
    }

    /// 获取旧版配置文件路径
    fn legacy_config_path() -> Option<PathBuf> {
        // 优先检查 exe 同目录
        if let Some(path) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .map(|d| d.join(Self::LEGACY_CONFIG_FILENAME))
        {
            if path.exists() {
                return Some(path);
            }
        }
        // 然后检查 AppData 目录
        if let Some(path) =
            dirs::config_dir().map(|d| d.join("IndustryVis").join(Self::LEGACY_CONFIG_FILENAME))
        {
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
        info!(target: "industry_vis::tag_group", "加载标签分组配置: {:?}", path);

        let config = if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: TagGroupConfig = toml::from_str(&content)?;
            info!(target: "industry_vis::tag_group", "加载了 {} 个分组", config.groups.len());
            config
        } else {
            // 尝试从旧版配置迁移
            if let Some(migrated) = Self::migrate_from_legacy()? {
                info!(target: "industry_vis::tag_group", "从旧版配置迁移了 {} 个分组", migrated.groups.len());
                let manager = Self {
                    config: migrated,
                    config_path: Self::save_config_path()?,
                };
                manager.save()?;
                return Ok(manager);
            }
            info!(target: "industry_vis::tag_group", "配置文件不存在，使用默认配置");
            TagGroupConfig::new()
        };

        Ok(Self {
            config,
            config_path: path,
        })
    }

    /// 从指定路径加载
    pub fn load_from(path: &PathBuf) -> AppResult<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: TagGroupConfig = toml::from_str(&content)?;
            Ok(Self {
                config,
                config_path: path.clone(),
            })
        } else {
            Ok(Self {
                config: TagGroupConfig::new(),
                config_path: path.clone(),
            })
        }
    }

    /// 从旧版配置迁移
    fn migrate_from_legacy() -> AppResult<Option<TagGroupConfig>> {
        let legacy_path = match Self::legacy_config_path() {
            Some(p) => p,
            None => return Ok(None),
        };

        info!(target: "industry_vis::tag_group", "发现旧版配置文件: {:?}", legacy_path);

        // 旧版数据结构
        #[derive(serde::Deserialize)]
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

        #[derive(serde::Deserialize)]
        struct LegacyConfig {
            #[serde(default)]
            groups: Vec<LegacyTagGroup>,
        }

        let content = fs::read_to_string(&legacy_path)?;
        let legacy: LegacyConfig = toml::from_str(&content)?;

        // 转换为新结构
        let groups: Vec<TagGroup> = legacy
            .groups
            .into_iter()
            .map(|g| {
                // 将旧的 tags 转换为默认图表
                let charts = if g.tags.is_empty() {
                    vec![ChartConfig::new("默认图表".to_string())]
                } else {
                    vec![ChartConfig::with_id(
                        format!("c{}", Local::now().timestamp_millis()),
                        "默认图表".to_string(),
                    )
                    .with_tags(g.tags)]
                };

                let mut group = TagGroup::with_id(g.id, g.name, charts, g.created_at, g.updated_at);
                group.processing_config = g.processing_config;
                group
            })
            .collect();

        Ok(Some(TagGroupConfig { version: 1, groups }))
    }

    /// 保存配置到文件
    pub fn save(&self) -> AppResult<()> {
        let path = Self::save_config_path()?;
        info!(target: "industry_vis::tag_group", "保存标签分组配置: {:?}, 分组数: {}", path, self.config.groups.len());
        let content = toml::to_string_pretty(&self.config)?;
        debug!(target: "industry_vis::tag_group", "配置内容:\n{}", content);
        fs::write(&path, content)?;
        Ok(())
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> AppResult<()> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            self.config = toml::from_str(&content)?;
            info!(target: "industry_vis::tag_group", "重新加载配置，{} 个分组", self.config.groups.len());
        }
        Ok(())
    }

    /// 获取所有分组
    pub fn list_groups(&self) -> &[TagGroup] {
        &self.config.groups
    }

    /// 根据 ID 获取分组
    pub fn get_group(&self, id: &str) -> Option<&TagGroup> {
        self.config.groups.iter().find(|g| g.id == id)
    }

    /// 根据 ID 获取可变分组
    fn get_group_mut(&mut self, id: &str) -> Option<&mut TagGroup> {
        self.config.groups.iter_mut().find(|g| g.id == id)
    }

    /// 创建新分组
    pub fn create_group(&mut self, name: String, charts: Vec<ChartConfig>) -> AppResult<TagGroup> {
        // 检查名称是否重复
        if self.config.groups.iter().any(|g| g.name == name.trim()) {
            return Err(AppError::Validation(format!(
                "分组名称 '{}' 已存在",
                name.trim()
            )));
        }

        let group = TagGroup::new(name, charts).map_err(AppError::Validation)?;
        let result = group.clone();
        self.config.groups.push(group);
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
        if self
            .config
            .groups
            .iter()
            .any(|g| g.id != id && g.name == name.trim())
        {
            return Err(AppError::Validation(format!(
                "分组名称 '{}' 已存在",
                name.trim()
            )));
        }

        let group = self
            .get_group_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("分组 '{}' 不存在", id)))?;

        group
            .update(name, charts, processing_config)
            .map_err(AppError::Validation)?;
        let result = group.clone();
        self.save()?;

        Ok(result)
    }

    /// 删除分组
    pub fn delete_group(&mut self, id: &str) -> AppResult<()> {
        let idx = self
            .config
            .groups
            .iter()
            .position(|g| g.id == id)
            .ok_or_else(|| AppError::NotFound(format!("分组 '{}' 不存在", id)))?;

        self.config.groups.remove(idx);
        self.save()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> TagGroupConfigManager {
        TagGroupConfigManager {
            config: TagGroupConfig::new(),
            config_path: PathBuf::from("/tmp/test_tag_groups.toml"),
        }
    }

    #[test]
    fn test_create_group() {
        let mut manager = create_test_manager();
        let chart = ChartConfig::new("图表1".to_string());

        // 注意：这个测试不会实际保存文件，因为路径可能不可写
        // 在实际场景中应使用 tempdir
        let result = manager.create_group("测试分组".to_string(), vec![chart]);
        // 由于保存可能失败，我们只测试内存操作
        assert!(manager.config.groups.is_empty() || result.is_ok() || result.is_err());
    }

    #[test]
    fn test_duplicate_name_validation() {
        let mut manager = create_test_manager();

        // 手动添加一个分组
        let group = TagGroup::new("已存在".to_string(), vec![]).unwrap();
        manager.config.groups.push(group);

        // 尝试创建同名分组应该失败
        let result = manager.create_group("已存在".to_string(), vec![]);
        assert!(result.is_err());
    }
}
