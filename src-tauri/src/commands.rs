use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tauri::State;
use tracing::{info, error, debug};

use crate::cache::{CacheKey, CacheStats, QueryCache};
use crate::config::{AppConfig, DatabaseConfig};
use crate::data_processing;
use crate::datasource::{DataSource, SqlServerSource};
use crate::error::AppResult;
use crate::models::{ConnectionTestResult, DataProcessingConfig, HistoryRecord, QueryParams, QueryResult, QueryResultV2};
use crate::tag_group::{ChartConfig, TagGroup, TagGroupConfig};

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
/// 
/// 支持缓存：相同参数的查询会复用缓存结果
#[tauri::command]
pub async fn query_history(
    params: QueryParams,
    processing_config: Option<DataProcessingConfig>,
    force_refresh: Option<bool>,
    cache: State<'_, Arc<QueryCache>>,
) -> AppResult<QueryResult> {
    let tag_count = params.tags.as_ref().map(|t| t.len()).unwrap_or(0);
    let force_refresh = force_refresh.unwrap_or(false);
    
    info!(target: "industry_vis_lib::commands", 
        "查询历史数据 - 时间: {} ~ {}, 标签数: {}, 强制刷新: {}", 
        params.start_time, params.end_time, tag_count, force_refresh
    );
    
    let config = AppConfig::load()?;
    let tags_ref = params.tags.as_ref().map(|v| v.as_slice());
    
    // 构建缓存键
    let cache_key = CacheKey::new(
        &config.query.default_table,
        &params.start_time,
        &params.end_time,
        tags_ref,
        processing_config.as_ref(),
    );
    
    // 检查缓存（非强制刷新时）
    if !force_refresh {
        if let Some(cached_records) = cache.get(&cache_key).await {
            info!(target: "industry_vis_lib::commands", 
                "缓存命中，返回 {} 条记录", cached_records.len()
            );
            
            // 应用分页
            let total = cached_records.len();
            let records = apply_pagination(cached_records, params.offset, params.limit);
            
            return Ok(QueryResult { records, total });
        }
    }
    
    // 缓存未命中或强制刷新，从数据库查询
    let source = SqlServerSource::new(config.database);
    let records = source.query_history(
        &config.query.default_table,
        &params.start_time,
        &params.end_time,
        tags_ref,
    ).await?;
    
    let total = records.len();
    info!(target: "industry_vis_lib::commands", "查询到 {} 条原始记录", total);
    
    // 使用数据处理模块进行处理（异常值剔除、重采样、平滑滤波、降采样）
    let processed_records = data_processing::process_query_result(records, processing_config.as_ref())?;
    
    // 将处理后的数据存入缓存
    cache.put(cache_key, processed_records.clone()).await;
    
    // 应用分页
    let records = apply_pagination(processed_records, params.offset, params.limit);
    
    info!(target: "industry_vis_lib::commands", "处理后返回 {} 条记录", records.len());
    
    // 返回原始总数，便于前端知道数据是否被处理
    Ok(QueryResult { 
        records, 
        total,  // 原始数据量
    })
}

/// 应用分页参数
fn apply_pagination(
    records: Vec<HistoryRecord>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Vec<HistoryRecord> {
    match (offset, limit) {
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
    }
}

/// 查询历史数据 V2 (预分组格式，优化前端渲染)
/// 
/// 返回按标签分组的 series 数据，可直接用于 ECharts
#[tauri::command]
pub async fn query_history_v2(
    params: QueryParams,
    processing_config: Option<DataProcessingConfig>,
    force_refresh: Option<bool>,
    cache: State<'_, Arc<QueryCache>>,
) -> AppResult<QueryResultV2> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    let tag_count = params.tags.as_ref().map(|t| t.len()).unwrap_or(0);
    let force_refresh = force_refresh.unwrap_or(false);
    
    info!(target: "industry_vis_lib::commands", 
        "查询历史数据 V2 - 时间: {} ~ {}, 标签数: {}, 强制刷新: {}", 
        params.start_time, params.end_time, tag_count, force_refresh
    );
    
    let config = AppConfig::load()?;
    let tags_ref = params.tags.as_ref().map(|v| v.as_slice());
    
    // 构建缓存键
    let cache_key = CacheKey::new(
        &config.query.default_table,
        &params.start_time,
        &params.end_time,
        tags_ref,
        processing_config.as_ref(),
    );
    
    // 检查缓存（非强制刷新时）
    if !force_refresh {
        if let Some(cached_records) = cache.get(&cache_key).await {
            let query_time_ms = start_time.elapsed().as_millis() as u64;
            let total_processed = cached_records.len();
            
            info!(target: "industry_vis_lib::commands", 
                "缓存命中 V2，返回 {} 条记录，耗时 {}ms", total_processed, query_time_ms
            );
            
            // 转换为 series 格式
            let series = data_processing::records_to_series(&cached_records);
            
            return Ok(QueryResultV2 {
                series,
                total_raw: total_processed, // 缓存中已是处理后的数据
                total_processed,
                cache_hit: true,
                query_time_ms,
            });
        }
    }
    
    // 缓存未命中或强制刷新，从数据库查询
    let source = SqlServerSource::new(config.database);
    let records = source.query_history(
        &config.query.default_table,
        &params.start_time,
        &params.end_time,
        tags_ref,
    ).await?;
    
    let total_raw = records.len();
    info!(target: "industry_vis_lib::commands", "查询到 {} 条原始记录", total_raw);
    
    // 使用数据处理模块进行处理
    let processed_records = data_processing::process_query_result(records, processing_config.as_ref())?;
    let total_processed = processed_records.len();
    
    // 将处理后的数据存入缓存
    cache.put(cache_key, processed_records.clone()).await;
    
    // 转换为 series 格式
    let series = data_processing::records_to_series(&processed_records);
    
    let query_time_ms = start_time.elapsed().as_millis() as u64;
    
    info!(target: "industry_vis_lib::commands", 
        "处理后返回 {} 条记录，{} 个系列，耗时 {}ms", 
        total_processed, series.len(), query_time_ms
    );
    
    Ok(QueryResultV2 {
        series,
        total_raw,
        total_processed,
        cache_hit: false,
        query_time_ms,
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

// ============== 缓存管理命令 ==============

/// 清空查询缓存
#[tauri::command]
pub async fn clear_cache(cache: State<'_, Arc<QueryCache>>) -> AppResult<()> {
    info!(target: "industry_vis_lib::commands", "清空查询缓存");
    // 先清理过期条目，再清空全部
    cache.evict_expired().await;
    cache.clear().await;
    Ok(())
}

/// 获取缓存统计信息
#[tauri::command]
pub async fn get_cache_stats(cache: State<'_, Arc<QueryCache>>) -> AppResult<CacheStats> {
    debug!(target: "industry_vis_lib::commands", "获取缓存统计");
    Ok(cache.get_stats().await)
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
pub async fn create_tag_group(name: String, charts: Vec<ChartConfig>) -> AppResult<TagGroup> {
    info!(target: "industry_vis_lib::commands", "创建分组 - 名称: {}, 图表数: {}", name, charts.len());
    let mut config = TagGroupConfig::load()?;
    config.create_group(name, charts)
}

/// 更新标签分组
#[tauri::command]
pub async fn update_tag_group(
    id: String, 
    name: String, 
    charts: Vec<ChartConfig>,
    processing_config: Option<DataProcessingConfig>,
) -> AppResult<TagGroup> {
    info!(target: "industry_vis_lib::commands", "更新分组 - ID: {}, 名称: {}, 图表数: {}", id, name, charts.len());
    let mut config = TagGroupConfig::load()?;
    config.update_group(&id, name, charts, processing_config)
}

/// 删除标签分组
#[tauri::command]
pub async fn delete_tag_group(id: String) -> AppResult<()> {
    info!(target: "industry_vis_lib::commands", "删除分组 - ID: {}", id);
    let mut config = TagGroupConfig::load()?;
    config.delete_group(&id)
}
