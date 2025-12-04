//! 标签分组数据模型

use super::DataProcessingConfig;
use chrono::Local;
use serde::{Deserialize, Serialize};

/// 图表配置（分组内的单个图表）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    /// 创建带 ID 的图表（用于测试）
    pub fn with_id(id: String, name: String) -> Self {
        Self {
            id,
            name,
            tags: Vec::new(),
        }
    }

    /// 添加标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// 验证图表配置
    pub fn validate(&self) -> Result<(), String> {
        if self.tags.len() > 5 {
            return Err(format!(
                "图表 '{}' 最多包含 5 个标签，当前 {}",
                self.name,
                self.tags.len()
            ));
        }
        Ok(())
    }
}

/// 标签分组（包含多个图表）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// 创建新分组
    pub fn new(name: String, charts: Vec<ChartConfig>) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("分组名称不能为空".to_string());
        }

        if charts.len() > 10 {
            return Err("每个分组最多包含 10 个图表".to_string());
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

    /// 创建带 ID 的分组（用于测试或迁移）
    pub fn with_id(
        id: String,
        name: String,
        charts: Vec<ChartConfig>,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            name,
            charts,
            processing_config: DataProcessingConfig::default(),
            created_at,
            updated_at,
        }
    }

    /// 更新分组
    pub fn update(
        &mut self,
        name: String,
        charts: Vec<ChartConfig>,
        processing_config: Option<DataProcessingConfig>,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("分组名称不能为空".to_string());
        }

        if charts.len() > 10 {
            return Err("每个分组最多包含 10 个图表".to_string());
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
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .charts
            .iter()
            .flat_map(|c| c.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }
}

/// 标签分组配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagGroupConfig {
    /// 配置文件版本
    pub version: u32,
    /// 所有分组
    #[serde(default)]
    pub groups: Vec<TagGroup>,
}

impl TagGroupConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self {
            version: 1,
            groups: Vec::new(),
        }
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
    fn test_chart_with_tags() {
        let chart = ChartConfig::new("温度".to_string())
            .with_tags(vec!["T1".to_string(), "T2".to_string()]);
        assert_eq!(chart.tags.len(), 2);
    }

    #[test]
    fn test_chart_validation() {
        let mut chart = ChartConfig::new("超限图表".to_string());
        chart.tags = (0..6).map(|i| format!("tag{}", i)).collect();
        assert!(chart.validate().is_err());
    }

    #[test]
    fn test_create_tag_group() {
        let chart = ChartConfig::new("图表1".to_string());
        let group = TagGroup::new("测试分组".to_string(), vec![chart]).unwrap();

        assert_eq!(group.name, "测试分组");
        assert_eq!(group.charts.len(), 1);
        assert!(group.id.starts_with("g"));
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
        let chart1 = ChartConfig::new("图表1".to_string())
            .with_tags(vec!["tag1".to_string(), "tag2".to_string()]);
        let chart2 = ChartConfig::new("图表2".to_string())
            .with_tags(vec!["tag2".to_string(), "tag3".to_string()]);

        let group = TagGroup::new("测试".to_string(), vec![chart1, chart2]).unwrap();
        let all_tags = group.all_tags();

        assert_eq!(all_tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_update_group() {
        let chart = ChartConfig::new("图表1".to_string());
        let mut group = TagGroup::new("原名称".to_string(), vec![chart]).unwrap();

        let new_chart = ChartConfig::new("新图表".to_string());
        group
            .update("新名称".to_string(), vec![new_chart], None)
            .unwrap();

        assert_eq!(group.name, "新名称");
        assert_eq!(group.charts.len(), 1);
        assert_eq!(group.charts[0].name, "新图表");
    }
}
