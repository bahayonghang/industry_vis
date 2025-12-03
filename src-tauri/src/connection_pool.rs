//! 数据库连接池模块
//!
//! 使用单连接复用模式，适合桌面应用单用户场景。
//! 支持连接健康检查和自动重连。
//!
//! 注意：此模块当前作为预留实现，将在后续版本中深度集成到 SqlServerSource。
//! 当前 SqlServerSource 仍使用每次查询创建新连接的方式。

#![allow(dead_code)]

use std::sync::Arc;
use tiberius::{AuthMethod, Client, Config, Query};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use tracing::{debug, error, info, warn};

use crate::config::DatabaseConfig;
use crate::error::{AppError, AppResult};

/// 客户端类型别名
pub type TiberiusClient = Client<Compat<TcpStream>>;

/// 数据库连接池
///
/// 使用 Mutex 保护的 Option<Client> 实现单连接复用：
/// - 首次使用时建立连接
/// - 后续请求复用已建立的连接
/// - 连接失效时自动重建
pub struct ConnectionPool {
    client: Arc<Mutex<Option<TiberiusClient>>>,
    config: Arc<RwLock<DatabaseConfig>>,
}

/// 池化连接包装器
///
/// 持有 MutexGuard，在作用域结束时自动释放
pub struct PooledConnection<'a> {
    guard: tokio::sync::MutexGuard<'a, Option<TiberiusClient>>,
}

impl<'a> PooledConnection<'a> {
    /// 获取内部客户端的可变引用
    pub fn client(&mut self) -> &mut TiberiusClient {
        self.guard.as_mut().expect("Connection should be established")
    }
}

impl ConnectionPool {
    /// 创建新的连接池
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// 更新数据库配置
    ///
    /// 更新配置后会断开现有连接，下次请求时使用新配置建立连接
    pub async fn update_config(&self, config: DatabaseConfig) {
        // 先更新配置
        {
            let mut cfg = self.config.write().await;
            *cfg = config.clone();
        }
        // 断开现有连接
        {
            let mut client = self.client.lock().await;
            *client = None;
        }
        info!(target: "industry_vis_lib::connection_pool", 
            "配置已更新，现有连接已断开，server={}:{}", 
            config.server, config.port
        );
    }

    /// 获取数据库连接
    ///
    /// 自动处理连接获取、健康检查、重连。
    /// 返回的连接在作用域结束时自动释放锁。
    pub async fn get_client(&self) -> AppResult<PooledConnection<'_>> {
        let mut client_guard = self.client.lock().await;
        
        // 检查现有连接是否可用
        if client_guard.is_some() {
            // 执行健康检查
            if !Self::health_check_inner(client_guard.as_mut().unwrap()).await {
                warn!(target: "industry_vis_lib::connection_pool", "连接健康检查失败，准备重连");
                *client_guard = None;
            } else {
                debug!(target: "industry_vis_lib::connection_pool", "复用现有连接");
            }
        }

        // 如果没有连接，建立新连接
        if client_guard.is_none() {
            let config = self.config.read().await;
            info!(target: "industry_vis_lib::connection_pool", 
                "建立新连接 - {}:{}/{}", 
                config.server, config.port, config.database
            );
            let new_client = Self::create_connection_inner(&config).await?;
            *client_guard = Some(new_client);
        }

        Ok(PooledConnection { guard: client_guard })
    }

    /// 创建新的数据库连接（内部方法）
    async fn create_connection_inner(config: &DatabaseConfig) -> AppResult<TiberiusClient> {
        let mut tiberius_config = Config::new();
        tiberius_config.host(&config.server);
        tiberius_config.port(config.port);
        tiberius_config.database(&config.database);
        tiberius_config.authentication(AuthMethod::sql_server(
            &config.username,
            &config.password,
        ));
        tiberius_config.trust_cert();

        let tcp = TcpStream::connect(tiberius_config.get_addr())
            .await
            .map_err(|e| {
                error!(target: "industry_vis_lib::connection_pool", "TCP 连接失败: {}", e);
                AppError::Connection(format!("TCP connection failed: {}", e))
            })?;

        tcp.set_nodelay(true)
            .map_err(|e| AppError::Connection(format!("Failed to set TCP_NODELAY: {}", e)))?;

        let client = Client::connect(tiberius_config, tcp.compat_write())
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                error!(target: "industry_vis_lib::connection_pool", "SQL Server 连接失败: {}", err_str);
                
                // 提供更友好的中文错误提示
                if err_str.contains("4060") {
                    AppError::Connection(format!(
                        "数据库 '{}' 不存在或无访问权限。请检查数据库名称是否正确。原始错误: {}",
                        config.database, err_str
                    ))
                } else if err_str.contains("18456") {
                    AppError::Connection(format!(
                        "用户名或密码错误。原始错误: {}", err_str
                    ))
                } else if err_str.contains("Login failed") {
                    AppError::Connection(format!(
                        "登录失败，请检查用户名和密码。原始错误: {}", err_str
                    ))
                } else {
                    AppError::Connection(format!("SQL Server 连接失败: {}", err_str))
                }
            })?;

        info!(target: "industry_vis_lib::connection_pool", "数据库连接建立成功");
        Ok(client)
    }

    /// 执行连接健康检查（内部方法）
    async fn health_check_inner(client: &mut TiberiusClient) -> bool {
        let query = Query::new("SELECT 1");
        match query.execute(client).await {
            Ok(_) => {
                debug!(target: "industry_vis_lib::connection_pool", "连接健康检查通过");
                true
            }
            Err(e) => {
                warn!(target: "industry_vis_lib::connection_pool", "连接健康检查失败: {}", e);
                false
            }
        }
    }

    /// 断开连接
    pub async fn disconnect(&self) {
        let mut client = self.client.lock().await;
        if client.is_some() {
            *client = None;
            info!(target: "industry_vis_lib::connection_pool", "连接已断开");
        }
    }

    /// 检查是否有活跃连接
    pub async fn is_connected(&self) -> bool {
        let client = self.client.lock().await;
        client.is_some()
    }

    /// 获取配置信息（用于日志和调试）
    pub async fn get_config_info(&self) -> String {
        let config = self.config.read().await;
        format!("{}:{}/{}", config.server, config.port, config.database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = DatabaseConfig {
            server: "localhost".to_string(),
            port: 1433,
            database: "test".to_string(),
            username: "sa".to_string(),
            password: "password".to_string(),
        };
        
        let pool = ConnectionPool::new(config);
        assert_eq!(pool.get_config_info().await, "localhost:1433/test");
    }

    #[tokio::test]
    async fn test_config_update() {
        let config1 = DatabaseConfig {
            server: "server1".to_string(),
            port: 1433,
            database: "db1".to_string(),
            username: "user1".to_string(),
            password: "pass1".to_string(),
        };
        
        let pool = ConnectionPool::new(config1);
        assert_eq!(pool.get_config_info().await, "server1:1433/db1");
        
        let config2 = DatabaseConfig {
            server: "server2".to_string(),
            port: 1434,
            database: "db2".to_string(),
            username: "user2".to_string(),
            password: "pass2".to_string(),
        };
        
        pool.update_config(config2).await;
        assert_eq!(pool.get_config_info().await, "server2:1434/db2");
    }
}
