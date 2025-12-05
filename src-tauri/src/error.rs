//! 错误类型定义模块
//!
//! 提供统一的错误处理机制，支持错误分类和 Tauri 序列化。

use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    // ============== 配置相关 ==============
    #[error("配置错误: {0}")]
    Config(String),

    #[error("配置文件监听错误: {0}")]
    ConfigWatch(String),

    // ============== 数据库相关 ==============
    #[error("数据库连接错误: {0}")]
    Connection(String),

    #[error("连接池错误: {0}")]
    Pool(String),

    #[error("查询执行错误: {0}")]
    Query(String),

    #[error("数据库未连接")]
    DatabaseNotConnected,

    // ============== 数据处理相关 ==============
    #[error("数据处理错误: {0}")]
    DataProcessing(String),

    // ============== 业务逻辑相关 ==============
    #[error("验证错误: {0}")]
    Validation(String),

    #[error("未找到: {0}")]
    NotFound(String),

    // ============== 系统错误 ==============
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML 解析错误: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML 序列化错误: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl AppError {
    /// 创建连接错误，带友好提示
    pub fn connection_with_hint(err: &str, database: &str) -> Self {
        if err.contains("4060") {
            Self::Connection(format!(
                "数据库 '{}' 不存在或无访问权限。请检查数据库名称是否正确。原始错误: {}",
                database, err
            ))
        } else if err.contains("18456") {
            Self::Connection(format!("用户名或密码错误。原始错误: {}", err))
        } else if err.contains("Login failed") {
            Self::Connection(format!("登录失败，请检查用户名和密码。原始错误: {}", err))
        } else {
            Self::Connection(format!("SQL Server 连接失败: {}", err))
        }
    }

    /// 是否为可重试错误
    pub fn is_retryable(&self) -> bool {
        matches!(self, AppError::Connection(_) | AppError::Pool(_))
    }

    /// 是否为用户可见错误
    pub fn is_user_facing(&self) -> bool {
        matches!(
            self,
            AppError::Config(_)
                | AppError::Connection(_)
                | AppError::Validation(_)
                | AppError::NotFound(_)
        )
    }
}

// 为 Tauri 序列化错误
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("error", &self.to_string())?;
        state.serialize_field("retryable", &self.is_retryable())?;
        state.end()
    }
}

/// 应用结果类型别名
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = AppError::Config("无效配置".to_string());
        assert!(err.to_string().contains("无效配置"));
    }

    #[test]
    fn test_connection_with_hint() {
        let err = AppError::connection_with_hint("error 4060", "TestDB");
        assert!(err.to_string().contains("不存在"));

        let err = AppError::connection_with_hint("error 18456", "TestDB");
        assert!(err.to_string().contains("用户名或密码"));
    }

    #[test]
    fn test_is_retryable() {
        assert!(AppError::Connection("timeout".to_string()).is_retryable());
        assert!(AppError::Pool("exhausted".to_string()).is_retryable());
        assert!(!AppError::Validation("invalid".to_string()).is_retryable());
    }

    #[test]
    fn test_is_user_facing() {
        assert!(AppError::Config("bad config".to_string()).is_user_facing());
        assert!(AppError::NotFound("item".to_string()).is_user_facing());
        assert!(!AppError::Internal("panic".to_string()).is_user_facing());
    }

    #[test]
    fn test_error_serialization() {
        let err = AppError::Connection("timeout".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("retryable"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }
}
