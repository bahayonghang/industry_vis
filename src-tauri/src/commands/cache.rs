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

/// 预热指定分组的缓存（1天数据）
///
/// 异步执行，不阻塞前端。用于进入分组时提前加载数据。
#[tauri::command]
pub async fn warmup_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    group_id: String,
) -> AppResult<()> {
    debug!(target: "industry_vis::commands", "预热分组缓存: {}", group_id);
    let state = state.read().await;
    state.warmup_group(&group_id).await
}
