//! 配置相关命令

use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::config::{AppConfig, DatabaseConfig};
use crate::datasource::{ConnectionPool, PoolConfig};
use crate::error::AppResult;
use crate::models::ConnectionTestResult;
use crate::state::AppState;

/// 加载配置
#[tauri::command]
pub async fn load_config(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<AppConfig> {
    info!(target: "industry_vis::commands", "加载配置");
    let state = state.read().await;
    Ok(state.config().app_config())
}

/// 保存配置
#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<()> {
    info!(target: "industry_vis::commands",
        "保存配置 - 服务器: {}:{}",
        config.database.server, config.database.port
    );
    let state = state.read().await;
    state.config().update_app_config(config)
}

/// 测试数据库连接
#[tauri::command]
pub async fn test_connection(config: DatabaseConfig) -> AppResult<ConnectionTestResult> {
    info!(target: "industry_vis::commands",
        "测试连接 - {}:{}/{}",
        config.server, config.port, config.database
    );

    // 创建临时连接池测试连接
    match ConnectionPool::new(config, PoolConfig::for_desktop()).await {
        Ok(pool) => {
            // 尝试获取连接
            match pool.get().await {
                Ok(_) => {
                    info!(target: "industry_vis::commands", "连接测试成功");
                    Ok(ConnectionTestResult::success())
                }
                Err(e) => {
                    error!(target: "industry_vis::commands", "连接测试失败: {}", e);
                    Ok(ConnectionTestResult::failure(e.to_string()))
                }
            }
        }
        Err(e) => {
            error!(target: "industry_vis::commands", "创建连接池失败: {}", e);
            Ok(ConnectionTestResult::failure(e.to_string()))
        }
    }
}

/// 获取当前连接状态（是否已初始化连接池）
#[tauri::command]
pub async fn get_connection_status(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<bool> {
    let state = state.read().await;
    Ok(state.is_pool_initialized())
}

/// 获取连接池状态
#[tauri::command]
pub async fn get_pool_state(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<Option<crate::datasource::PoolState>> {
    let state = state.read().await;
    Ok(state.get_pool_state())
}
