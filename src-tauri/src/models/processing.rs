//! 数据处理配置模型

use serde::{Deserialize, Serialize};

/// 异常值剔除配置
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OutlierRemovalConfig {
    pub enabled: bool,
    #[serde(default = "default_outlier_method")]
    pub method: String, // "3sigma"
}

fn default_outlier_method() -> String {
    "3sigma".to_string()
}

/// 重采样配置
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResampleConfig {
    pub enabled: bool,
    #[serde(default = "default_resample_interval")]
    pub interval: u32, // 秒
    #[serde(default = "default_resample_method")]
    pub method: String, // "mean"
}

fn default_resample_interval() -> u32 {
    60 // 默认1分钟
}

fn default_resample_method() -> String {
    "mean".to_string()
}

/// 平滑滤波配置
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SmoothingConfig {
    pub enabled: bool,
    #[serde(default = "default_smoothing_method")]
    pub method: String, // "moving_avg"
    #[serde(default = "default_smoothing_window")]
    pub window: usize, // 窗口大小
}

fn default_smoothing_method() -> String {
    "moving_avg".to_string()
}

fn default_smoothing_window() -> usize {
    5
}

/// 数据处理配置
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DataProcessingConfig {
    #[serde(default)]
    pub outlier_removal: OutlierRemovalConfig,
    #[serde(default)]
    pub resample: ResampleConfig,
    #[serde(default)]
    pub smoothing: SmoothingConfig,
}

impl DataProcessingConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 启用异常值剔除
    pub fn with_outlier_removal(mut self, method: &str) -> Self {
        self.outlier_removal.enabled = true;
        self.outlier_removal.method = method.to_string();
        self
    }

    /// 启用重采样
    pub fn with_resample(mut self, interval: u32, method: &str) -> Self {
        self.resample.enabled = true;
        self.resample.interval = interval;
        self.resample.method = method.to_string();
        self
    }

    /// 启用平滑滤波
    pub fn with_smoothing(mut self, window: usize, method: &str) -> Self {
        self.smoothing.enabled = true;
        self.smoothing.window = window;
        self.smoothing.method = method.to_string();
        self
    }

    /// 检查是否有任何处理启用
    pub fn has_any_enabled(&self) -> bool {
        self.outlier_removal.enabled || self.resample.enabled || self.smoothing.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DataProcessingConfig::default();
        assert!(!config.outlier_removal.enabled);
        assert!(!config.resample.enabled);
        assert!(!config.smoothing.enabled);
        assert!(!config.has_any_enabled());
    }

    #[test]
    fn test_config_builder() {
        let config = DataProcessingConfig::new()
            .with_outlier_removal("3sigma")
            .with_resample(60, "mean")
            .with_smoothing(5, "moving_avg");

        assert!(config.outlier_removal.enabled);
        assert_eq!(config.outlier_removal.method, "3sigma");
        assert!(config.resample.enabled);
        assert_eq!(config.resample.interval, 60);
        assert!(config.smoothing.enabled);
        assert_eq!(config.smoothing.window, 5);
        assert!(config.has_any_enabled());
    }

    #[test]
    fn test_config_serialization() {
        let config = DataProcessingConfig::new().with_outlier_removal("3sigma");
        let json = serde_json::to_string(&config).unwrap();
        let parsed: DataProcessingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }
}
