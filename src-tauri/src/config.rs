use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 查询配置
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 完整应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub query: QueryConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            query: QueryConfig::default(),
        }
    }
}

impl AppConfig {
    /// 获取配置文件路径
    pub fn config_path() -> AppResult<PathBuf> {
        let exe_dir = std::env::current_exe()
            .map_err(|e| AppError::Config(format!("Failed to get exe path: {}", e)))?
            .parent()
            .ok_or_else(|| AppError::Config("Failed to get exe directory".to_string()))?
            .to_path_buf();
        
        Ok(exe_dir.join("config.toml"))
    }

    /// 从文件加载配置
    pub fn load() -> AppResult<Self> {
        let path = Self::config_path()?;
        
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // 返回默认配置
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
        assert_eq!(parsed.database.server, config.database.server);
    }
}
