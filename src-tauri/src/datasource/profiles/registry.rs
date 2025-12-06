//! Profile 注册表
//!
//! 提供 Profile 的注册和获取功能。

use std::sync::Arc;

use crate::datasource::SchemaProfile;
use crate::error::{AppError, AppResult};

use super::DefaultProfile;

/// Profile 注册表
///
/// 提供根据名称获取 Profile 实例的工厂方法。
pub struct ProfileRegistry;

impl ProfileRegistry {
    /// 根据名称获取 Profile
    ///
    /// # Arguments
    /// * `name` - Profile 名称
    ///
    /// # Returns
    /// * `Ok(Arc<dyn SchemaProfile>)` - Profile 实例
    /// * `Err(AppError)` - 未找到指定名称的 Profile
    ///
    /// # Supported Profiles
    /// - `"default"` - 默认 Profile（当前厂商）
    ///
    /// # Example
    /// ```ignore
    /// let profile = ProfileRegistry::get("default")?;
    /// println!("Using profile: {}", profile.name());
    /// ```
    pub fn get(name: &str) -> AppResult<Arc<dyn SchemaProfile>> {
        match name {
            "default" => Ok(Arc::new(DefaultProfile::new())),
            _ => Err(AppError::Config(format!(
                "未知的 Schema Profile: '{}'. 可用的 Profile: default",
                name
            ))),
        }
    }

    /// 获取默认 Profile
    ///
    /// 快捷方法，等同于 `ProfileRegistry::get("default")`
    pub fn default_profile() -> Arc<dyn SchemaProfile> {
        Arc::new(DefaultProfile::new())
    }

    /// 列出所有可用的 Profile 名称
    pub fn available_profiles() -> &'static [&'static str] {
        &["default"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_profile() {
        let profile = ProfileRegistry::get("default").unwrap();
        assert_eq!(profile.name(), "default");
    }

    #[test]
    fn test_get_unknown_profile() {
        let result = ProfileRegistry::get("unknown");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("未知的 Schema Profile"));
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn test_default_profile_shortcut() {
        let profile = ProfileRegistry::default_profile();
        assert_eq!(profile.name(), "default");
    }

    #[test]
    fn test_available_profiles() {
        let profiles = ProfileRegistry::available_profiles();
        assert!(profiles.contains(&"default"));
    }
}
