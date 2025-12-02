//! 数据处理模块
//! 
//! 使用 Polars 实现高效数据处理：
//! - 异常值剔除 (3σ法则)
//! - 时间序列重采样 (均值聚合)
//! - 平滑滤波 (移动平均)
//! - 数据降采样

use std::collections::HashMap;
use crate::models::{DataProcessingConfig, HistoryRecord};
use crate::error::AppResult;

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
        tag_groups.entry(record.tag_name.clone()).or_default().push(record);
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

/// 3σ法则异常值剔除
/// 移除超出 μ±3σ 范围的数据点
fn remove_outliers(records: Vec<HistoryRecord>) -> AppResult<Vec<HistoryRecord>> {
    if records.len() < 3 {
        return Ok(records);
    }

    // 计算均值和标准差
    let values: Vec<f64> = records.iter().map(|r| r.tag_val).collect();
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    // 3σ 边界
    let lower = mean - 3.0 * std_dev;
    let upper = mean + 3.0 * std_dev;

    // 过滤异常值
    let result: Vec<HistoryRecord> = records
        .into_iter()
        .filter(|r| r.tag_val >= lower && r.tag_val <= upper)
        .collect();

    Ok(result)
}

/// 时间序列重采样（均值聚合）
/// interval: 重采样间隔（秒）
fn resample_data(records: Vec<HistoryRecord>, interval: u32) -> AppResult<Vec<HistoryRecord>> {
    if records.is_empty() {
        return Ok(records);
    }

    // 解析时间并按时间窗口分组
    let interval_ms = interval as i64 * 1000;
    let mut windows: HashMap<i64, Vec<&HistoryRecord>> = HashMap::new();

    for record in &records {
        // 解析 ISO 时间字符串
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&record.date_time, "%Y-%m-%dT%H:%M:%S%.3f") {
            let timestamp_ms = dt.and_utc().timestamp_millis();
            let window_key = (timestamp_ms / interval_ms) * interval_ms;
            windows.entry(window_key).or_default().push(record);
        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&record.date_time, "%Y-%m-%dT%H:%M:%S") {
            let timestamp_ms = dt.and_utc().timestamp_millis();
            let window_key = (timestamp_ms / interval_ms) * interval_ms;
            windows.entry(window_key).or_default().push(record);
        }
    }

    // 对每个窗口计算均值
    let mut result: Vec<HistoryRecord> = windows
        .into_iter()
        .map(|(window_key, window_records)| {
            let avg_val = window_records.iter().map(|r| r.tag_val).sum::<f64>() 
                / window_records.len() as f64;
            
            // 使用窗口开始时间作为时间戳
            let dt = chrono::DateTime::from_timestamp_millis(window_key)
                .unwrap_or_default()
                .naive_utc();
            
            HistoryRecord {
                date_time: dt.format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
                tag_name: window_records[0].tag_name.clone(),
                tag_val: avg_val,
                tag_quality: window_records[0].tag_quality.clone(),
            }
        })
        .collect();

    // 按时间排序
    result.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    Ok(result)
}

/// 移动平均平滑滤波
fn smooth_data(records: Vec<HistoryRecord>, window: usize) -> AppResult<Vec<HistoryRecord>> {
    if records.len() < window || window < 2 {
        return Ok(records);
    }

    let values: Vec<f64> = records.iter().map(|r| r.tag_val).collect();
    let mut smoothed_values = Vec::with_capacity(values.len());

    // 计算移动平均
    for i in 0..values.len() {
        let start = if i >= window / 2 { i - window / 2 } else { 0 };
        let end = (i + window / 2 + 1).min(values.len());
        let window_vals: Vec<f64> = values[start..end].to_vec();
        let avg = window_vals.iter().sum::<f64>() / window_vals.len() as f64;
        smoothed_values.push(avg);
    }

    // 更新记录值
    let result: Vec<HistoryRecord> = records
        .into_iter()
        .enumerate()
        .map(|(i, mut record)| {
            record.tag_val = smoothed_values[i];
            record
        })
        .collect();

    Ok(result)
}

/// 降采样（保持向后兼容）
pub fn downsample(records: Vec<HistoryRecord>, max_points_per_tag: usize) -> AppResult<Vec<HistoryRecord>> {
    if records.is_empty() {
        return Ok(records);
    }

    // 按标签名分组
    let mut tag_groups: HashMap<String, Vec<HistoryRecord>> = HashMap::new();
    for record in records {
        tag_groups.entry(record.tag_name.clone()).or_default().push(record);
    }

    let mut result = Vec::new();

    // 对每个标签进行降采样
    for (_tag, tag_records) in tag_groups {
        let count = tag_records.len();

        if count <= max_points_per_tag {
            result.extend(tag_records);
        } else {
            let step = count / max_points_per_tag;
            for (i, record) in tag_records.into_iter().enumerate() {
                if i % step == 0 {
                    result.push(record);
                }
            }
        }
    }

    // 按时间排序
    result.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    Ok(result)
}

/// 完整数据处理流程
/// 包含数据处理和降采样
pub fn process_query_result(
    records: Vec<HistoryRecord>,
    config: Option<&DataProcessingConfig>,
) -> AppResult<Vec<HistoryRecord>> {
    let records = if let Some(cfg) = config {
        process_data(records, cfg)?
    } else {
        records
    };

    // 最后进行降采样，避免前端渲染过多数据
    downsample(records, 5000)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records(count: usize) -> Vec<HistoryRecord> {
        (0..count)
            .map(|i| HistoryRecord {
                date_time: format!("2024-01-01T00:{:02}:00.000", i),
                tag_name: "Tag1".to_string(),
                tag_val: (i as f64) + 10.0,
                tag_quality: "Good".to_string(),
            })
            .collect()
    }

    #[test]
    fn test_remove_outliers() {
        let mut records = create_test_records(10);
        // 添加一个异常值
        records.push(HistoryRecord {
            date_time: "2024-01-01T00:10:00.000".to_string(),
            tag_name: "Tag1".to_string(),
            tag_val: 1000.0,  // 明显异常
            tag_quality: "Good".to_string(),
        });

        let result = remove_outliers(records).unwrap();
        // 异常值应该被移除
        assert!(result.iter().all(|r| r.tag_val < 100.0));
    }

    #[test]
    fn test_smooth_data() {
        let records = create_test_records(10);
        let result = smooth_data(records, 3).unwrap();
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_resample_data() {
        let records = create_test_records(10);
        let result = resample_data(records, 120).unwrap();  // 2分钟间隔
        // 10分钟数据，2分钟间隔，应该约5个点
        assert!(result.len() <= 6);
    }
}
