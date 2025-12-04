//! 标签分组命令

use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::AppResult;
use crate::models::{ChartConfig, DataProcessingConfig, TagGroup};
use crate::state::AppState;

/// 获取所有标签分组
#[tauri::command]
pub async fn list_tag_groups(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<Vec<TagGroup>> {
    debug!(target: "industry_vis::commands", "获取标签分组列表");
    let state = state.read().await;
    Ok(state.tag_group_service().list_groups())
}

/// 创建标签分组
#[tauri::command]
pub async fn create_tag_group(
    name: String,
    charts: Vec<ChartConfig>,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<TagGroup> {
    info!(target: "industry_vis::commands",
        "创建分组 - 名称: {}, 图表数: {}",
        name, charts.len()
    );
    let state = state.read().await;
    state.tag_group_service().create_group(name, charts)
}

/// 更新标签分组
#[tauri::command]
pub async fn update_tag_group(
    id: String,
    name: String,
    charts: Vec<ChartConfig>,
    processing_config: Option<DataProcessingConfig>,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<TagGroup> {
    info!(target: "industry_vis::commands",
        "更新分组 - ID: {}, 名称: {}, 图表数: {}",
        id, name, charts.len()
    );
    let state = state.read().await;
    state
        .tag_group_service()
        .update_group(&id, name, charts, processing_config)
}

/// 删除标签分组
#[tauri::command]
pub async fn delete_tag_group(
    id: String,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<()> {
    info!(target: "industry_vis::commands", "删除分组 - ID: {}", id);
    let state = state.read().await;
    state.tag_group_service().delete_group(&id)
}
