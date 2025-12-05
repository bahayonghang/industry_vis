//! 数据查询命令

use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::error::AppResult;
use crate::models::{DataProcessingConfig, HistoryRecord, QueryParams, QueryResult, QueryResultV2};
use crate::state::AppState;

/// 获取可用标签列表
#[tauri::command]
pub async fn get_available_tags(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<Vec<String>> {
    info!(target: "industry_vis::commands", "获取可用标签列表");
    let state = state.read().await;
    match state.query_service() {
        Some(service) => service.get_available_tags().await,
        None => {
            info!(target: "industry_vis::commands", "数据库未连接，返回空标签列表");
            Ok(vec![])
        }
    }
}

/// 模糊搜索标签
#[tauri::command]
pub async fn search_tags(
    keyword: String,
    limit: Option<u32>,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<Vec<String>> {
    info!(target: "industry_vis::commands", "搜索标签 - 关键词: {}", keyword);
    let limit = limit.unwrap_or(50) as usize;
    let state = state.read().await;
    match state.query_service() {
        Some(service) => service.search_tags(&keyword, limit).await,
        None => {
            info!(target: "industry_vis::commands", "数据库未连接，返回空搜索结果");
            Ok(vec![])
        }
    }
}

/// 查询历史数据
#[tauri::command]
pub async fn query_history(
    params: QueryParams,
    processing_config: Option<DataProcessingConfig>,
    force_refresh: Option<bool>,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<QueryResult> {
    let tag_count = params.tags.as_ref().map(|t| t.len()).unwrap_or(0);
    let force_refresh = force_refresh.unwrap_or(false);

    info!(target: "industry_vis::commands",
        "查询历史数据 - 时间: {} ~ {}, 标签数: {}, 强制刷新: {}",
        params.start_time, params.end_time, tag_count, force_refresh
    );

    let state = state.read().await;
    match state.query_service() {
        Some(service) => {
            service
                .query_history(&params, processing_config.as_ref(), force_refresh)
                .await
        }
        None => {
            info!(target: "industry_vis::commands", "数据库未连接，无法查询历史数据");
            Err(crate::error::AppError::DatabaseNotConnected)
        }
    }
}

/// 查询历史数据 V2 (预分组格式)
#[tauri::command]
pub async fn query_history_v2(
    params: QueryParams,
    processing_config: Option<DataProcessingConfig>,
    force_refresh: Option<bool>,
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<QueryResultV2> {
    let tag_count = params.tags.as_ref().map(|t| t.len()).unwrap_or(0);
    let force_refresh = force_refresh.unwrap_or(false);

    info!(target: "industry_vis::commands",
        "查询历史数据 V2 - 时间: {} ~ {}, 标签数: {}, 强制刷新: {}",
        params.start_time, params.end_time, tag_count, force_refresh
    );

    let state = state.read().await;
    match state.query_service() {
        Some(service) => {
            service
                .query_history_v2(&params, processing_config.as_ref(), force_refresh)
                .await
        }
        None => {
            info!(target: "industry_vis::commands", "数据库未连接，无法查询历史数据");
            Err(crate::error::AppError::DatabaseNotConnected)
        }
    }
}

/// 导出数据到 CSV
#[tauri::command]
pub async fn export_to_csv(records: Vec<HistoryRecord>, file_path: String) -> AppResult<()> {
    info!(target: "industry_vis::commands",
        "导出CSV - 路径: {}, 记录数: {}",
        file_path, records.len()
    );

    let mut file = File::create(&file_path)?;

    // Write header
    writeln!(file, "DateTime,TagName,TagVal,TagQuality")?;

    // Write records
    for record in records {
        writeln!(
            file,
            "{},{},{},{}",
            record.date_time,
            record.tag_name.replace(',', ";"),
            record.tag_val,
            record.tag_quality.replace(',', ";")
        )?;
    }

    info!(target: "industry_vis::commands", "CSV导出完成");
    Ok(())
}
