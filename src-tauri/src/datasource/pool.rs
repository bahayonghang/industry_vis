//! bb8 数据库连接池实现
//!
//! 为 SQL Server (tiberius) 提供异步连接池支持。

use async_trait::async_trait;
use bb8::{Pool, PooledConnection};
use std::sync::Arc;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use tracing::{debug, info};

use crate::config::DatabaseConfig;
use crate::error::{AppError, AppResult};

/// Tiberius 客户端类型
pub type TiberiusClient = Client<Compat<TcpStream>>;

/// 连接池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 最大连接数
    pub max_size: u32,
    /// 最小空闲连接数
    pub min_idle: Option<u32>,
    /// 连接超时（秒）
    pub connection_timeout_secs: u64,
    /// 空闲超时（秒）
    pub idle_timeout_secs: Option<u64>,
    /// 最大生命周期（秒）
    pub max_lifetime_secs: Option<u64>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 5,
            min_idle: Some(1),
            connection_timeout_secs: 30,
            idle_timeout_secs: Some(600),  // 10 分钟
            max_lifetime_secs: Some(1800), // 30 分钟
        }
    }
}

impl PoolConfig {
    /// 创建适合桌面应用的配置
    pub fn for_desktop() -> Self {
        Self {
            max_size: 3, // 支持并发查询（多图表场景）
            min_idle: Some(1),
            connection_timeout_secs: 15,  // 缩短超时，快速失败
            idle_timeout_secs: Some(300), // 5 分钟
            max_lifetime_secs: Some(900), // 15 分钟
        }
    }
}

/// bb8 连接管理器
pub struct ConnectionManager {
    config: DatabaseConfig,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new(config: DatabaseConfig) -> Self {
        Self { config }
    }

    /// 创建数据库连接
    async fn create_connection(&self) -> AppResult<TiberiusClient> {
        let mut tiberius_config = Config::new();
        tiberius_config.host(&self.config.server);
        tiberius_config.port(self.config.port);
        tiberius_config.database(&self.config.database);
        tiberius_config.authentication(AuthMethod::sql_server(
            &self.config.username,
            &self.config.password,
        ));
        tiberius_config.trust_cert();

        debug!(target: "industry_vis::pool",
            "创建新连接 - {}:{}/{}",
            self.config.server, self.config.port, self.config.database
        );

        let tcp = TcpStream::connect(tiberius_config.get_addr())
            .await
            .map_err(|e| AppError::Connection(format!("TCP 连接失败: {}", e)))?;

        tcp.set_nodelay(true)
            .map_err(|e| AppError::Connection(format!("设置 TCP_NODELAY 失败: {}", e)))?;

        let client = Client::connect(tiberius_config, tcp.compat_write())
            .await
            .map_err(|e| AppError::connection_with_hint(&e.to_string(), &self.config.database))?;

        info!(target: "industry_vis::pool", "数据库连接创建成功");
        Ok(client)
    }
}

#[async_trait]
impl bb8::ManageConnection for ConnectionManager {
    type Connection = TiberiusClient;
    type Error = AppError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        self.create_connection().await
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // 执行简单查询验证连接
        use tiberius::Query;
        let query = Query::new("SELECT 1");
        query
            .execute(conn)
            .await
            .map_err(|e| AppError::Connection(format!("连接验证失败: {}", e)))?;
        debug!(target: "industry_vis::pool", "连接验证通过");
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

/// 数据库连接池
pub struct ConnectionPool {
    pool: Pool<ConnectionManager>,
    config: DatabaseConfig,
    max_size: u32,
}

impl ConnectionPool {
    /// 创建新的连接池
    pub async fn new(db_config: DatabaseConfig, pool_config: PoolConfig) -> AppResult<Self> {
        let manager = ConnectionManager::new(db_config.clone());

        let pool = Pool::builder()
            .max_size(pool_config.max_size)
            .min_idle(pool_config.min_idle)
            .connection_timeout(std::time::Duration::from_secs(
                pool_config.connection_timeout_secs,
            ))
            .idle_timeout(
                pool_config
                    .idle_timeout_secs
                    .map(std::time::Duration::from_secs),
            )
            .max_lifetime(
                pool_config
                    .max_lifetime_secs
                    .map(std::time::Duration::from_secs),
            )
            .build(manager)
            .await
            .map_err(|e| AppError::Pool(format!("创建连接池失败: {}", e)))?;

        info!(target: "industry_vis::pool",
            "连接池已创建 - max_size={}, server={}:{}",
            pool_config.max_size, db_config.server, db_config.port
        );

        Ok(Self {
            pool,
            config: db_config,
            max_size: pool_config.max_size,
        })
    }

    /// 使用默认配置创建连接池
    pub async fn with_defaults(db_config: DatabaseConfig) -> AppResult<Self> {
        Self::new(db_config, PoolConfig::for_desktop()).await
    }

    /// 获取一个连接
    pub async fn get(&self) -> AppResult<PooledConnection<'_, ConnectionManager>> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::Pool(format!("获取连接失败: {}", e)))
    }

    /// 获取连接池状态
    pub fn state(&self) -> PoolState {
        let state = self.pool.state();
        PoolState {
            connections: state.connections,
            idle_connections: state.idle_connections,
            active_connections: state.connections.saturating_sub(state.idle_connections),
            max_size: self.max_size,
        }
    }

    /// 获取数据库配置
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }
}

/// 连接池状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolState {
    /// 总连接数
    pub connections: u32,
    /// 空闲连接数
    pub idle_connections: u32,
    /// 活跃连接数（总连接数 - 空闲连接数）
    pub active_connections: u32,
    /// 最大连接数
    pub max_size: u32,
}

/// 可共享的连接池（用于 Tauri 状态）
#[allow(dead_code)]
pub type SharedPool = Arc<ConnectionPool>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_size, 5);
        assert_eq!(config.min_idle, Some(1));
    }

    #[test]
    fn test_pool_config_desktop() {
        let config = PoolConfig::for_desktop();
        assert_eq!(config.max_size, 3); // 优化后支持并发查询
        assert_eq!(config.connection_timeout_secs, 15); // 快速失败策略
    }

    #[test]
    fn test_connection_manager_creation() {
        let db_config = DatabaseConfig::default();
        let manager = ConnectionManager::new(db_config.clone());
        assert_eq!(manager.config.server, db_config.server);
    }

    // 连接池的集成测试需要实际的数据库连接，在集成测试中进行
}
