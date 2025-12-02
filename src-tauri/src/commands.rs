use std::fs::File;
use std::io::Write;
use tracing::{info, error, debug};

use crate::config::{AppConfig, DatabaseConfig};
use crate::data_processing;
use crate::datasource::{DataSource, SqlServerSource};
use crate::error::AppResult;
use crate::models::{ConnectionTestResult, DataProcessingConfig, HistoryRecord, QueryParams, QueryResult};
use crate::tag_group::{TagGroup, TagGroupConfig};

/// 加载配置
#[tauri::command]
pub async fn load_config() -> AppResult<AppConfig> {
    info!(target: "industry_vis_lib::commands", "加载配置");
    AppConfig::load()
}

/// 保存配置
#[tauri::command]
pub async fn save_config(config: AppConfig) -> AppResult<()> {
    info!(target: "industry_vis_lib::commands", "保存配置 - 服务器: {}:{}", config.database.server, config.database.port);
    config.save()
}

/// 测试数据库连接
#[tauri::command]
pub async fn test_connection(config: DatabaseConfig) -> AppResult<ConnectionTestResult> {
    info!(target: "industry_vis_lib::commands", "测试连接 - {}:{}/{}", config.server, config.port, config.database);
    let source = SqlServerSource::new(config);
    
    match source.test_connection().await {
        Ok(_) => {
            info!(target: "industry_vis_lib::commands", "连接测试成功");
            Ok(ConnectionTestResult {
                success: true,
                message: "连接成功".to_string(),
            })
        },
        Err(e) => {
            error!(target: "industry_vis_lib::commands", "连接测试失败: {}", e);
            Ok(ConnectionTestResult {
                success: false,
                message: e.to_string(),
            })
        },
    }
}

/// 获取可用标签列表
#[tauri::command]
pub async fn get_available_tags() -> AppResult<Vec<String>> {
    info!(target: "industry_vis_lib::commands", "获取可用标签列表");
    let config = AppConfig::load()?;
    let source = SqlServerSource::new(config.database);
    source.get_available_tags(&config.query.default_table).await
}

/// 查询历史数据
#[tauri::command]
pub async fn query_history(
    params: QueryParams,
    processing_config: Option<DataProcessingConfig>,
) -> AppResult<QueryResult> {
    let tag_count = params.tags.as_ref().map(|t| t.len()).unwrap_or(0);
    info!(target: "industry_vis_lib::commands", 
        "查询历史数据 - 时间: {} ~ {}, 标签数: {}", 
        params.start_time, params.end_time, tag_count
    );
    
    let config = AppConfig::load()?;
    let source = SqlServerSource::new(config.database);
    
    let tags_ref = params.tags.as_ref().map(|v| v.as_slice());
    
    let records = source.query_history(
        &config.query.default_table,
        &params.start_time,
        &params.end_time,
        tags_ref,
    ).await?;
    
    let total = records.len();
    info!(target: "industry_vis_lib::commands", "查询到 {} 条原始记录", total);
    
    // 使用数据处理模块进行处理（异常值剔除、重采样、平滑滤波、降采样）
    let records = data_processing::process_query_result(records, processing_config.as_ref())?;
    
    // Apply pagination if specified
    let records = match (params.offset, params.limit) {
        (Some(offset), Some(limit)) => {
            records.into_iter().skip(offset).take(limit).collect()
        }
        (Some(offset), None) => {
            records.into_iter().skip(offset).collect()
        }
        (None, Some(limit)) => {
            records.into_iter().take(limit).collect()
        }
        (None, None) => records,
    };
    
    info!(target: "industry_vis_lib::commands", "处理后返回 {} 条记录", records.len());
    
    // 返回原始总数，便于前端知道数据是否被处理
    Ok(QueryResult { 
        records, 
        total,  // 原始数据量
    })
}

/// 导出数据到 CSV
#[tauri::command]
pub async fn export_to_csv(records: Vec<HistoryRecord>, file_path: String) -> AppResult<()> {
    info!(target: "industry_vis_lib::commands", "导出CSV - 路径: {}, 记录数: {}", file_path, records.len());
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
    
    info!(target: "industry_vis_lib::commands", "CSV导出完成");
    Ok(())
}

// ============== 标签分组相关命令 ==============

/// 模糊搜索标签（从 TagDataBase 表）
#[tauri::command]
pub async fn search_tags(keyword: String, limit: Option<u32>) -> AppResult<Vec<String>> {
    info!(target: "industry_vis_lib::commands", "搜索标签 - 关键词: {}", keyword);
    let config = AppConfig::load()?;
    let source = SqlServerSource::new(config.database);
    
    let limit = limit.unwrap_or(50) as usize;
    let result = source.search_tags(&keyword, limit).await;
    
    match &result {
        Ok(tags) => info!(target: "industry_vis_lib::commands", "搜索到 {} 个标签", tags.len()),
        Err(e) => error!(target: "industry_vis_lib::commands", "搜索标签失败: {}", e),
    }
    
    result
}

/// 获取所有标签分组
#[tauri::command]
pub async fn list_tag_groups() -> AppResult<Vec<TagGroup>> {
    debug!(target: "industry_vis_lib::commands", "获取标签分组列表");
    let config = TagGroupConfig::load()?;
    Ok(config.groups)
}

/// 创建标签分组
#[tauri::command]
pub async fn create_tag_group(name: String, tags: Vec<String>) -> AppResult<TagGroup> {
    info!(target: "industry_vis_lib::commands", "创建分组 - 名称: {}, 标签数: {}", name, tags.len());
    let mut config = TagGroupConfig::load()?;
    config.create_group(name, tags)
}

/// 更新标签分组
#[tauri::command]
pub async fn update_tag_group(
    id: String, 
    name: String, 
    tags: Vec<String>,
    processing_config: Option<DataProcessingConfig>,
) -> AppResult<TagGroup> {
    info!(target: "industry_vis_lib::commands", "更新分组 - ID: {}, 名称: {}", id, name);
    let mut config = TagGroupConfig::load()?;
    config.update_group(&id, name, tags, processing_config)
}

/// 删除标签分组
#[tauri::command]
pub async fn delete_tag_group(id: String) -> AppResult<()> {
    info!(target: "industry_vis_lib::commands", "删除分组 - ID: {}", id);
    let mut config = TagGroupConfig::load()?;
    config.delete_group(&id)
}
