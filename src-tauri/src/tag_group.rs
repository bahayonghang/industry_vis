//! 标签分组管理模块
//! 
//! 支持将多个标签组合成一个分组，便于批量查询和管理。
//! 分组配置保存在 `tag_groups.toml` 文件中。

use chrono::{Local, DateTime};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};

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
            created_at: now.clone(),
            updated_at: now,
        })
    }
    
    /// 更新分组
    pub fn update(&mut self, name: String, tags: Vec<String>) -> AppResult<()> {
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
    /// 获取配置文件路径
    pub fn config_path() -> AppResult<PathBuf> {
        let exe_dir = std::env::current_exe()
            .map_err(|e| AppError::Config(format!("获取程序路径失败: {}", e)))?
            .parent()
            .ok_or_else(|| AppError::Config("获取程序目录失败".to_string()))?
            .to_path_buf();
        
        Ok(exe_dir.join("tag_groups.toml"))
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
        let path = Self::config_path()?;
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
    pub fn update_group(&mut self, id: &str, name: String, tags: Vec<String>) -> AppResult<TagGroup> {
        // 检查名称是否与其他分组重复
        if self.groups.iter().any(|g| g.id != id && g.name == name.trim()) {
            return Err(AppError::Validation(
                format!("分组名称 '{}' 已存在", name.trim())
            ));
        }
        
        let group = self.get_group_mut(id)
            .ok_or_else(|| AppError::NotFound(format!("分组 '{}' 不存在", id)))?;
        
        group.update(name, tags)?;
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
