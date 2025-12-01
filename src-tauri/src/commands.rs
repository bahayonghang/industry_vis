use std::fs::File;
use std::io::Write;

use crate::config::{AppConfig, DatabaseConfig};
use crate::datasource::{DataSource, SqlServerSource};
use crate::error::AppResult;
use crate::models::{ConnectionTestResult, HistoryRecord, QueryParams, QueryResult};

/// 加载配置
#[tauri::command]
pub async fn load_config() -> AppResult<AppConfig> {
    AppConfig::load()
}

/// 保存配置
#[tauri::command]
pub async fn save_config(config: AppConfig) -> AppResult<()> {
    config.save()
}

/// 测试数据库连接
#[tauri::command]
pub async fn test_connection(config: DatabaseConfig) -> AppResult<ConnectionTestResult> {
    let source = SqlServerSource::new(config);
    
    match source.test_connection().await {
        Ok(_) => Ok(ConnectionTestResult {
            success: true,
            message: "连接成功".to_string(),
        }),
        Err(e) => Ok(ConnectionTestResult {
            success: false,
            message: e.to_string(),
        }),
    }
}

/// 获取可用标签列表
#[tauri::command]
pub async fn get_available_tags() -> AppResult<Vec<String>> {
    let config = AppConfig::load()?;
    let source = SqlServerSource::new(config.database);
    source.get_available_tags(&config.query.default_table).await
}

/// 查询历史数据
#[tauri::command]
pub async fn query_history(params: QueryParams) -> AppResult<QueryResult> {
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
    
    Ok(QueryResult { records, total })
}

/// 导出数据到 CSV
#[tauri::command]
pub async fn export_to_csv(records: Vec<HistoryRecord>, file_path: String) -> AppResult<()> {
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
    
    Ok(())
}
