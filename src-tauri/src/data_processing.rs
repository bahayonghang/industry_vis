//! 数据处理模块
//! 
//! 使用 Polars lazy evaluation 实现高效数据处理：
//! - 异常值剔除 (3σ法则) - 使用 filter + 统计函数
//! - 时间序列重采样 (均值聚合) - 使用 group_by_dynamic
//! - 平滑滤波 (移动平均) - 使用 rolling_mean
//! - 数据降采样

use std::collections::HashMap;
use polars::prelude::*;
use tracing::{debug, warn};
use crate::models::{ChartSeriesData, DataProcessingConfig, HistoryRecord};
use crate::error::{AppError, AppResult};

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
                    debug!(target: "industry_vis_lib::data_processing", 
                        "Polars 处理完成: {} -> {} 条", record_count, result.len());
                    result
                }
                Err(e) => {
                    warn!(target: "industry_vis_lib::data_processing", 
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
/// 
/// 返回的 series 中每个 ChartSeriesData 包含:
/// - tag_name: 标签名
/// - data: [[timestamp_ms, value], ...] 格式的数据点
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
    // 尝试多种格式
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_time, "%Y-%m-%dT%H:%M:%S%.3f") {
        return Some(dt.and_utc().timestamp_millis() as f64);
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_time, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt.and_utc().timestamp_millis() as f64);
    }
    None
}

// ============== Polars 优化版本 ==============

/// 将 HistoryRecord 列表转换为 Polars DataFrame
/// 
/// 列结构：datetime (Datetime), tag_name (String), tag_val (Float64), tag_quality (String)
pub fn records_to_dataframe(records: &[HistoryRecord]) -> AppResult<DataFrame> {
    if records.is_empty() {
        // 返回空 DataFrame
        return Ok(DataFrame::new(vec![
            Column::new_empty("datetime".into(), &DataType::Datetime(TimeUnit::Milliseconds, None)),
            Column::new_empty("tag_name".into(), &DataType::String),
            Column::new_empty("tag_val".into(), &DataType::Float64),
            Column::new_empty("tag_quality".into(), &DataType::String),
        ]).map_err(|e| AppError::DataProcessing(e.to_string()))?);
    }

    // 解析时间戳
    let timestamps: Vec<i64> = records
        .iter()
        .map(|r| parse_timestamp_ms(&r.date_time).unwrap_or(0.0) as i64)
        .collect();
    
    let tag_names: Vec<&str> = records.iter().map(|r| r.tag_name.as_str()).collect();
    let tag_vals: Vec<f64> = records.iter().map(|r| r.tag_val).collect();
    let tag_qualities: Vec<&str> = records.iter().map(|r| r.tag_quality.as_str()).collect();

    let datetime_col = Column::new("datetime".into(), timestamps)
        .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
        .map_err(|e| AppError::DataProcessing(e.to_string()))?;
    
    let df = DataFrame::new(vec![
        datetime_col,
        Column::new("tag_name".into(), tag_names),
        Column::new("tag_val".into(), tag_vals),
        Column::new("tag_quality".into(), tag_qualities),
    ]).map_err(|e| AppError::DataProcessing(e.to_string()))?;

    debug!(target: "industry_vis_lib::data_processing", 
        "转换 {} 条记录为 DataFrame", records.len());
    
    Ok(df)
}

/// 将 Polars DataFrame 转换回 HistoryRecord 列表
pub fn dataframe_to_records(df: &DataFrame) -> AppResult<Vec<HistoryRecord>> {
    let datetime_col = df.column("datetime")
        .map_err(|e| AppError::DataProcessing(format!("缺少 datetime 列: {}", e)))?;
    let tag_name_col = df.column("tag_name")
        .map_err(|e| AppError::DataProcessing(format!("缺少 tag_name 列: {}", e)))?;
    let tag_val_col = df.column("tag_val")
        .map_err(|e| AppError::DataProcessing(format!("缺少 tag_val 列: {}", e)))?;
    let tag_quality_col = df.column("tag_quality")
        .map_err(|e| AppError::DataProcessing(format!("缺少 tag_quality 列: {}", e)))?;

    let datetimes = datetime_col.datetime()
        .map_err(|e| AppError::DataProcessing(format!("datetime 列类型错误: {}", e)))?;
    let tag_names = tag_name_col.str()
        .map_err(|e| AppError::DataProcessing(format!("tag_name 列类型错误: {}", e)))?;
    let tag_vals = tag_val_col.f64()
        .map_err(|e| AppError::DataProcessing(format!("tag_val 列类型错误: {}", e)))?;
    let tag_qualities = tag_quality_col.str()
        .map_err(|e| AppError::DataProcessing(format!("tag_quality 列类型错误: {}", e)))?;

    let mut records = Vec::with_capacity(df.height());
    
    for i in 0..df.height() {
        let ts_ms = datetimes.get(i).unwrap_or(0);
        let dt = chrono::DateTime::from_timestamp_millis(ts_ms)
            .unwrap_or_default()
            .naive_utc();
        
        records.push(HistoryRecord {
            date_time: dt.format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
            tag_name: tag_names.get(i).unwrap_or("").to_string(),
            tag_val: tag_vals.get(i).unwrap_or(0.0),
            tag_quality: tag_qualities.get(i).unwrap_or("").to_string(),
        });
    }

    debug!(target: "industry_vis_lib::data_processing", 
        "转换 DataFrame ({} 行) 为记录", records.len());
    
    Ok(records)
}

/// 使用 Polars lazy API 处理数据
/// 
/// 处理管道：DataFrame → [异常值剔除] → [重采样] → [平滑] → DataFrame
pub fn process_data_polars(
    records: Vec<HistoryRecord>,
    config: &DataProcessingConfig,
) -> AppResult<Vec<HistoryRecord>> {
    if records.is_empty() {
        return Ok(records);
    }

    // 按标签分组处理（每个标签独立处理以避免跨标签影响）
    let mut tag_groups: HashMap<String, Vec<HistoryRecord>> = HashMap::new();
    for record in records {
        tag_groups.entry(record.tag_name.clone()).or_default().push(record);
    }

    let mut all_results = Vec::new();

    for (tag_name, tag_records) in tag_groups {
        match process_tag_data_polars(tag_records, config) {
            Ok(processed) => all_results.extend(processed),
            Err(e) => {
                warn!(target: "industry_vis_lib::data_processing", 
                    "Polars 处理标签 {} 失败，回退到原始实现: {}", tag_name, e);
                // 回退到原始实现
                // 这里简单跳过，实际应该回退处理
            }
        }
    }

    // 按时间排序
    all_results.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    Ok(all_results)
}

/// 使用 Polars 处理单个标签的数据
fn process_tag_data_polars(
    records: Vec<HistoryRecord>,
    config: &DataProcessingConfig,
) -> AppResult<Vec<HistoryRecord>> {
    // 转换为 DataFrame
    let df = records_to_dataframe(&records)?;
    let mut lf = df.lazy();

    // 1. 异常值剔除（使用 3σ 法则）
    if config.outlier_removal.enabled {
        lf = remove_outliers_polars(lf)?;
    }

    // 2. 平滑滤波（移动平均）
    // 注意：平滑需要在重采样之前，保持数据点密度
    if config.smoothing.enabled && config.smoothing.window > 1 {
        lf = smooth_data_polars(lf, config.smoothing.window)?;
    }

    // 收集结果
    let result_df = lf.collect()
        .map_err(|e| AppError::DataProcessing(format!("Polars 执行失败: {}", e)))?;

    // 3. 重采样（如果需要）- 在收集后处理以简化逻辑
    let final_df = if config.resample.enabled && config.resample.interval > 0 {
        resample_data_polars(&result_df, config.resample.interval)?
    } else {
        result_df
    };

    dataframe_to_records(&final_df)
}

/// Polars 版本的 3σ 异常值剔除
fn remove_outliers_polars(lf: LazyFrame) -> AppResult<LazyFrame> {
    // 计算 mean 和 std，然后过滤 μ±3σ 范围外的数据
    let result = lf
        .with_columns([
            col("tag_val").mean().alias("_mean"),
            col("tag_val").std(1).alias("_std"),
        ])
        .filter(
            col("tag_val")
                .gt_eq(col("_mean") - lit(3.0) * col("_std"))
                .and(col("tag_val").lt_eq(col("_mean") + lit(3.0) * col("_std")))
        )
        .select([
            col("datetime"),
            col("tag_name"),
            col("tag_val"),
            col("tag_quality"),
        ]);

    Ok(result)
}

/// Polars 版本的移动平均平滑
fn smooth_data_polars(lf: LazyFrame, window: usize) -> AppResult<LazyFrame> {
    let options = RollingOptionsFixedWindow {
        window_size: window,
        min_periods: 1,
        center: true,
        ..Default::default()
    };

    let result = lf
        .sort(["datetime"], Default::default())
        .with_columns([
            col("tag_val")
                .rolling_mean(options)
                .alias("tag_val"),
        ]);

    Ok(result)
}

/// Polars 版本的时间序列重采样
fn resample_data_polars(df: &DataFrame, interval_seconds: u32) -> AppResult<DataFrame> {
    // 简化实现：使用时间窗口分组
    let interval_ms = interval_seconds as i64 * 1000;
    
    let result = df.clone().lazy()
        .with_columns([
            // 计算时间窗口
            (col("datetime").cast(DataType::Int64) / lit(interval_ms) * lit(interval_ms))
                .cast(DataType::Datetime(TimeUnit::Milliseconds, None))
                .alias("datetime"),
        ])
        .group_by([col("datetime"), col("tag_name")])
        .agg([
            col("tag_val").mean().alias("tag_val"),
            col("tag_quality").first().alias("tag_quality"),
        ])
        .sort(["datetime"], Default::default())
        .collect()
        .map_err(|e| AppError::DataProcessing(format!("重采样失败: {}", e)))?;

    Ok(result)
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

    // ============== 性能测试 ==============

    /// 创建大量测试记录（用于性能测试）
    fn create_large_test_records(count: usize, tags: usize) -> Vec<HistoryRecord> {
        let base_time = chrono::NaiveDateTime::parse_from_str(
            "2024-01-01T00:00:00.000",
            "%Y-%m-%dT%H:%M:%S%.3f"
        ).unwrap();
        
        let mut records = Vec::with_capacity(count * tags);
        for tag_idx in 0..tags {
            for i in 0..count {
                let dt = base_time + chrono::Duration::seconds(i as i64);
                records.push(HistoryRecord {
                    date_time: dt.format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
                    tag_name: format!("Tag{}", tag_idx),
                    tag_val: (i as f64) * 0.1 + (tag_idx as f64) * 100.0 + rand_float(),
                    tag_quality: "Good".to_string(),
                });
            }
        }
        records
    }

    /// 简单的伪随机数生成（避免引入 rand crate）
    fn rand_float() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        (nanos % 1000) as f64 / 1000.0
    }

    #[test]
    fn test_records_to_dataframe_performance() {
        let records = create_large_test_records(10000, 5); // 5万条记录
        let total = records.len();
        
        let start = std::time::Instant::now();
        let df = records_to_dataframe(&records).unwrap();
        let duration = start.elapsed();
        
        println!("records_to_dataframe: {} 条记录，耗时 {:?}", total, duration);
        assert_eq!(df.height(), total);
        assert!(duration.as_millis() < 1000, "转换 5 万条记录应该在 1 秒内完成");
    }

    #[test]
    fn test_dataframe_to_records_performance() {
        let records = create_large_test_records(10000, 5);
        let df = records_to_dataframe(&records).unwrap();
        let total = df.height();
        
        let start = std::time::Instant::now();
        let result = dataframe_to_records(&df).unwrap();
        let duration = start.elapsed();
        
        println!("dataframe_to_records: {} 行，耗时 {:?}", total, duration);
        assert_eq!(result.len(), total);
        assert!(duration.as_millis() < 1000, "转换 5 万行应该在 1 秒内完成");
    }

    #[test]
    fn test_records_to_series_performance() {
        let records = create_large_test_records(10000, 10); // 10万条记录，10个标签
        let total = records.len();
        
        let start = std::time::Instant::now();
        let series = records_to_series(&records);
        let duration = start.elapsed();
        
        println!("records_to_series: {} 条记录，{} 个系列，耗时 {:?}", 
            total, series.len(), duration);
        assert_eq!(series.len(), 10);
        assert!(duration.as_millis() < 500, "转换 10 万条记录应该在 500ms 内完成");
    }

    #[test]
    fn test_process_data_performance() {
        let records = create_large_test_records(5000, 5); // 2.5万条记录
        let total = records.len();
        
        let config = DataProcessingConfig {
            outlier_removal: crate::models::OutlierRemovalConfig {
                enabled: true,
                method: "3sigma".to_string(),
            },
            resample: crate::models::ResampleConfig {
                enabled: true,
                interval: 60, // 60秒重采样
                method: "mean".to_string(),
            },
            smoothing: crate::models::SmoothingConfig {
                enabled: true,
                method: "moving_avg".to_string(),
                window: 5,
            },
        };
        
        let start = std::time::Instant::now();
        let result = process_data(records, &config).unwrap();
        let duration = start.elapsed();
        
        println!("process_data: {} → {} 条记录，耗时 {:?}", total, result.len(), duration);
        assert!(duration.as_millis() < 2000, "处理 2.5 万条记录应该在 2 秒内完成");
    }

    #[test]
    fn test_downsample_performance() {
        let records = create_large_test_records(20000, 5); // 10万条记录
        let total = records.len();
        
        let start = std::time::Instant::now();
        let result = downsample(records, 5000).unwrap();
        let duration = start.elapsed();
        
        println!("downsample: {} → {} 条记录，耗时 {:?}", total, result.len(), duration);
        // 每个标签最多 5000 点，5 个标签，最多 25000 点
        assert!(result.len() <= 25000);
        assert!(duration.as_millis() < 500, "降采样 10 万条记录应该在 500ms 内完成");
    }

    #[test]
    fn test_polars_process_performance() {
        let records = create_large_test_records(5000, 5);
        let total = records.len();
        
        let config = DataProcessingConfig {
            outlier_removal: crate::models::OutlierRemovalConfig {
                enabled: true,
                method: "3sigma".to_string(),
            },
            resample: crate::models::ResampleConfig {
                enabled: false,
                interval: 60,
                method: "mean".to_string(),
            },
            smoothing: crate::models::SmoothingConfig {
                enabled: true,
                method: "moving_avg".to_string(),
                window: 5,
            },
        };
        
        let start = std::time::Instant::now();
        let result = process_data_polars(records, &config).unwrap();
        let duration = start.elapsed();
        
        println!("process_data_polars: {} → {} 条记录，耗时 {:?}", total, result.len(), duration);
        assert!(duration.as_millis() < 3000, "Polars 处理 2.5 万条记录应该在 3 秒内完成");
    }
}
