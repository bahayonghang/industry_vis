//! 标签分组管理模块
//! 
//! 支持将多个标签组合成一个分组，便于批量查询和管理。
//! 分组配置保存在 `tag_groups.toml` 文件中。

use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::models::DataProcessingConfig;

/// 单个标签分组
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagGroup {
    /// 唯一标识符
    pub id: String,
    /// 分组名称
    pub name: String,
    /// 包含的标签列表（最多 20 个）
    pub tags: Vec<String>,
    /// 数据处理配置
    #[serde(default)]
    pub processing_config: DataProcessingConfig,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl TagGroup {
    /// 创建新分组（允许空标签列表）
    pub fn new(name: String, tags: Vec<String>) -> AppResult<Self> {
        if tags.len() > 20 {
            return Err(AppError::Validation(
                "每个分组最多包含 20 个标签".to_string()
            ));
        }
        
        if name.trim().is_empty() {
            return Err(AppError::Validation(
                "分组名称不能为空".to_string()
            ));
        }
        
        let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let id = format!("g{}", Local::now().timestamp_millis());
        
        Ok(Self {
            id,
            name: name.trim().to_string(),
            tags,  // 允许为空
            processing_config: DataProcessingConfig::default(),
            created_at: now.clone(),
            updated_at: now,
        })
    }
    
    /// 更新分组
    pub fn update(
        &mut self, 
        name: String, 
        tags: Vec<String>,
        processing_config: Option<DataProcessingConfig>,
    ) -> AppResult<()> {
        if tags.len() > 20 {
            return Err(AppError::Validation(
                "每个分组最多包含 20 个标签".to_string()
            ));
        }
        
        if name.trim().is_empty() {
            return Err(AppError::Validation(
                "分组名称不能为空".to_string()
            ));
        }
        
        self.name = name.trim().to_string();
        self.tags = tags;  // 允许为空
        if let Some(config) = processing_config {
            self.processing_config = config;
        }
        self.updated_at = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        
        Ok(())
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
    /// 获取 exe 同目录的配置路径（便携模式）
    fn portable_config_path() -> Option<PathBuf> {
        std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .map(|d| d.join("tag_groups.toml"))
    }

    /// 获取 AppData 目录的配置路径（安装模式）
    fn appdata_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("IndustryVis").join("tag_groups.toml"))
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
        
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: TagGroupConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // 返回默认空配置
            Ok(Self::default())
        }
    }
    
    /// 保存配置到文件
    pub fn save(&self) -> AppResult<()> {
        let path = Self::save_config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }
    
    /// 获取所有分组
    pub fn list_groups(&self) -> &[TagGroup] {
        &self.groups
    }
    
    /// 根据 ID 获取分组
    pub fn get_group(&self, id: &str) -> Option<&TagGroup> {
        self.groups.iter().find(|g| g.id == id)
    }
    
    /// 根据 ID 获取可变分组
    pub fn get_group_mut(&mut self, id: &str) -> Option<&mut TagGroup> {
        self.groups.iter_mut().find(|g| g.id == id)
    }
    
    /// 创建新分组
    pub fn create_group(&mut self, name: String, tags: Vec<String>) -> AppResult<TagGroup> {
        // 检查名称是否重复
        if self.groups.iter().any(|g| g.name == name.trim()) {
            return Err(AppError::Validation(
                format!("分组名称 '{}' 已存在", name.trim())
            ));
        }
        
        let group = TagGroup::new(name, tags)?;
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
        tags: Vec<String>,
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
        
        group.update(name, tags, processing_config)?;
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
    fn test_create_tag_group() {
        let group = TagGroup::new(
            "测试分组".to_string(),
            vec!["tag1".to_string(), "tag2".to_string()]
        ).unwrap();
        
        assert_eq!(group.name, "测试分组");
        assert_eq!(group.tags.len(), 2);
        assert!(group.id.starts_with("g"));
    }
    
    #[test]
    fn test_tag_limit() {
        let tags: Vec<String> = (0..21).map(|i| format!("tag{}", i)).collect();
        let result = TagGroup::new("超限分组".to_string(), tags);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_name() {
        let result = TagGroup::new("  ".to_string(), vec!["tag1".to_string()]);
        assert!(result.is_err());
    }
}
