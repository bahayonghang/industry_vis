//! 数据处理模块
//!
//! 提供数据处理功能：异常值剔除、重采样、平滑滤波、降采样。
//! 支持 Polars 和原生 Rust 两种实现。

mod native;
mod polars_impl;

pub use native::{downsample, remove_outliers, resample_data, smooth_data};
pub use polars_impl::{dataframe_to_records, process_data_polars, records_to_dataframe};

use crate::error::AppResult;
use crate::models::{ChartSeriesData, DataProcessingConfig, HistoryRecord};
use std::collections::HashMap;
use tracing::{debug, warn};

/// 处理查询结果
/// 处理顺序：异常值剔除 → 重采样 → 平滑滤波
pub fn process_data(
    records: Vec<HistoryRecord>,
    config: &DataProcessingConfig,
) -> AppResult<Vec<HistoryRecord>> {
    if records.is_empty() {
        return Ok(records);
    }

    // 按标签分组处理
    let mut tag_groups: HashMap<String, Vec<HistoryRecord>> = HashMap::new();
    for record in records {
        tag_groups
            .entry(record.tag_name.clone())
            .or_default()
            .push(record);
    }

    let mut result = Vec::new();

    for (tag_name, tag_records) in tag_groups {
        let processed = process_tag_data(tag_records, config, &tag_name)?;
        result.extend(processed);
    }

    // 按时间排序
    result.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    Ok(result)
}

/// 处理单个标签的数据
fn process_tag_data(
    mut records: Vec<HistoryRecord>,
    config: &DataProcessingConfig,
    _tag_name: &str,
) -> AppResult<Vec<HistoryRecord>> {
    // 1. 异常值剔除
    if config.outlier_removal.enabled {
        records = remove_outliers(records)?;
    }

    // 2. 重采样
    if config.resample.enabled && config.resample.interval > 0 {
        records = resample_data(records, config.resample.interval)?;
    }

    // 3. 平滑滤波
    if config.smoothing.enabled && config.smoothing.window > 1 {
        records = smooth_data(records, config.smoothing.window)?;
    }

    Ok(records)
}

/// 完整数据处理流程
/// 包含数据处理和降采样
///
/// 优先使用 Polars 优化版本，失败时回退到原生实现
pub fn process_query_result(
    records: Vec<HistoryRecord>,
    config: Option<&DataProcessingConfig>,
) -> AppResult<Vec<HistoryRecord>> {
    let record_count = records.len();

    let records = if let Some(cfg) = config {
        // 大数据量时优先使用 Polars（阈值: 1000 条）
        if record_count > 1000 {
            match process_data_polars(records.clone(), cfg) {
                Ok(result) => {
                    debug!(target: "industry_vis::processing",
                        "Polars 处理完成: {} -> {} 条", record_count, result.len());
                    result
                }
                Err(e) => {
                    warn!(target: "industry_vis::processing",
                        "Polars 处理失败，回退到原生实现: {}", e);
                    process_data(records, cfg)?
                }
            }
        } else {
            // 小数据量使用原生实现（避免 Polars 开销）
            process_data(records, cfg)?
        }
    } else {
        records
    };

    // 最后进行降采样，避免前端渲染过多数据
    downsample(records, 5000)
}

/// 将 HistoryRecord 列表转换为 V2 格式（按标签预分组）
pub fn records_to_series(records: &[HistoryRecord]) -> Vec<ChartSeriesData> {
    // 按标签分组
    let mut tag_groups: HashMap<String, Vec<[f64; 2]>> = HashMap::new();

    for record in records {
        // 解析时间戳
        let timestamp_ms = parse_timestamp_ms(&record.date_time).unwrap_or(0.0);

        tag_groups
            .entry(record.tag_name.clone())
            .or_default()
            .push([timestamp_ms, record.tag_val]);
    }

    // 转换为 Vec<ChartSeriesData>，按标签名排序
    let mut series: Vec<ChartSeriesData> = tag_groups
        .into_iter()
        .map(|(tag_name, mut data)| {
            // 按时间戳排序
            data.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));
            ChartSeriesData { tag_name, data }
        })
        .collect();

    series.sort_by(|a, b| a.tag_name.cmp(&b.tag_name));
    series
}

/// 解析时间字符串为毫秒时间戳
fn parse_timestamp_ms(date_time: &str) -> Option<f64> {
    use chrono::{Local, TimeZone};

    // 尝试多种格式
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_time, "%Y-%m-%dT%H:%M:%S%.3f") {
        if let Some(local_dt) = Local.from_local_datetime(&dt).single() {
            return Some(local_dt.timestamp_millis() as f64);
        }
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_time, "%Y-%m-%dT%H:%M:%S") {
        if let Some(local_dt) = Local.from_local_datetime(&dt).single() {
            return Some(local_dt.timestamp_millis() as f64);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records(count: usize) -> Vec<HistoryRecord> {
        (0..count)
            .map(|i| {
                HistoryRecord::new(
                    format!("2024-01-01T00:{:02}:00.000", i),
                    "Tag1".to_string(),
                    (i as f64) + 10.0,
                    "Good".to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn test_process_data_empty() {
        let records: Vec<HistoryRecord> = vec![];
        let config = DataProcessingConfig::default();
        let result = process_data(records, &config).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_records_to_series() {
        let records = create_test_records(5);
        let series = records_to_series(&records);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].tag_name, "Tag1");
        assert_eq!(series[0].data.len(), 5);
    }

    #[test]
    fn test_parse_timestamp() {
        let ts = parse_timestamp_ms("2024-01-01T00:00:00.000");
        assert!(ts.is_some());

        let ts = parse_timestamp_ms("2024-01-01T00:00:00");
        assert!(ts.is_some());

        let ts = parse_timestamp_ms("invalid");
        assert!(ts.is_none());
    }
}
