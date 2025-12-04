//! 缓存管理命令

use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::cache::CacheStats;
use crate::error::AppResult;
use crate::state::AppState;

/// 清空查询缓存
#[tauri::command]
pub async fn clear_cache(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<()> {
    info!(target: "industry_vis::commands", "清空查询缓存");
    let state = state.read().await;
    state.cache().evict_expired().await;
    state.cache().clear().await;
    Ok(())
}

/// 获取缓存统计信息
#[tauri::command]
pub async fn get_cache_stats(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<CacheStats> {
    debug!(target: "industry_vis::commands", "获取缓存统计");
    let state = state.read().await;
    Ok(state.cache().get_stats().await)
}
