//! 应用配置管理

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::error::{AppError, AppResult};

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseConfig {
    pub server: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            server: "localhost".to_string(),
            port: 1433,
            database: "控制器数据库".to_string(),
            username: "sa".to_string(),
            password: String::new(),
        }
    }
}

impl DatabaseConfig {
    /// 获取连接字符串（用于显示，隐藏密码）
    pub fn connection_string_masked(&self) -> String {
        format!(
            "Server={};Port={};Database={};User={}",
            self.server, self.port, self.database, self.username
        )
    }
}

/// 查询配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueryConfig {
    pub default_table: String,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            default_table: "历史表".to_string(),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub query: QueryConfig,
}

impl AppConfig {
    /// 配置文件名
    const CONFIG_FILENAME: &'static str = "config.toml";

    /// 获取 exe 同目录的配置路径（便携模式）
    pub fn portable_config_path() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .map(|d| d.join(Self::CONFIG_FILENAME))
    }

    /// 获取 AppData 目录的配置路径（安装模式）
    pub fn appdata_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("IndustryVis").join(Self::CONFIG_FILENAME))
    }

    /// 获取配置文件路径
    pub fn config_path() -> AppResult<PathBuf> {
        // 优先使用 exe 同目录（便携模式）
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
                let test_file = parent.join(".config_write_test");
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
        debug!(target: "industry_vis::config", "配置文件路径: {}", path.display());

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            info!(target: "industry_vis::config",
                "加载配置成功 - 服务器: {}:{}, 数据库: {}",
                config.database.server, config.database.port, config.database.database
            );
            Ok(config)
        } else {
            info!(target: "industry_vis::config", "配置文件不存在，使用默认配置");
            Ok(Self::default())
        }
    }

    /// 从指定路径加载配置
    pub fn load_from(path: &PathBuf) -> AppResult<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Err(AppError::Config(format!(
                "配置文件不存在: {}",
                path.display()
            )))
        }
    }

    /// 保存配置到文件
    pub fn save(&self) -> AppResult<()> {
        let path = Self::save_config_path()?;
        info!(target: "industry_vis::config",
            "保存配置到: {} - 服务器: {}:{}, 数据库: {}",
            path.display(), self.database.server, self.database.port, self.database.database
        );
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        info!(target: "industry_vis::config", "配置保存成功");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.server, "localhost");
        assert_eq!(config.database.port, 1433);
        assert_eq!(config.database.database, "控制器数据库");
        assert_eq!(config.query.default_table, "历史表");
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_database_config_masked() {
        let config = DatabaseConfig {
            server: "192.168.1.1".to_string(),
            port: 1433,
            database: "TestDB".to_string(),
            username: "admin".to_string(),
            password: "secret123".to_string(),
        };
        let masked = config.connection_string_masked();
        assert!(masked.contains("192.168.1.1"));
        assert!(!masked.contains("secret123")); // 密码不应出现
    }
}
