//! Polars 数据处理实现

use polars::prelude::*;
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::error::{AppError, AppResult};
use crate::models::{DataProcessingConfig, HistoryRecord};

/// 将 HistoryRecord 列表转换为 Polars DataFrame
pub fn records_to_dataframe(records: &[HistoryRecord]) -> AppResult<DataFrame> {
    if records.is_empty() {
        return DataFrame::new(vec![
            Column::new_empty(
                "datetime".into(),
                &DataType::Datetime(TimeUnit::Milliseconds, None),
            ),
            Column::new_empty("tag_name".into(), &DataType::String),
            Column::new_empty("tag_val".into(), &DataType::Float64),
            Column::new_empty("tag_quality".into(), &DataType::String),
        ])
        .map_err(|e| AppError::DataProcessing(e.to_string()));
    }

    // 解析时间戳
    let timestamps: Vec<i64> = records
        .iter()
        .map(|r| parse_timestamp_ms(&r.date_time).unwrap_or(0) as i64)
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
    ])
    .map_err(|e| AppError::DataProcessing(e.to_string()))?;

    debug!(target: "industry_vis::processing",
        "转换 {} 条记录为 DataFrame", records.len());

    Ok(df)
}

/// 将 Polars DataFrame 转换回 HistoryRecord 列表
pub fn dataframe_to_records(df: &DataFrame) -> AppResult<Vec<HistoryRecord>> {
    let datetime_col = df
        .column("datetime")
        .map_err(|e| AppError::DataProcessing(format!("缺少 datetime 列: {}", e)))?;
    let tag_name_col = df
        .column("tag_name")
        .map_err(|e| AppError::DataProcessing(format!("缺少 tag_name 列: {}", e)))?;
    let tag_val_col = df
        .column("tag_val")
        .map_err(|e| AppError::DataProcessing(format!("缺少 tag_val 列: {}", e)))?;
    let tag_quality_col = df
        .column("tag_quality")
        .map_err(|e| AppError::DataProcessing(format!("缺少 tag_quality 列: {}", e)))?;

    let datetimes = datetime_col
        .datetime()
        .map_err(|e| AppError::DataProcessing(format!("datetime 列类型错误: {}", e)))?;
    let tag_names = tag_name_col
        .str()
        .map_err(|e| AppError::DataProcessing(format!("tag_name 列类型错误: {}", e)))?;
    let tag_vals = tag_val_col
        .f64()
        .map_err(|e| AppError::DataProcessing(format!("tag_val 列类型错误: {}", e)))?;
    let tag_qualities = tag_quality_col
        .str()
        .map_err(|e| AppError::DataProcessing(format!("tag_quality 列类型错误: {}", e)))?;

    let mut records = Vec::with_capacity(df.height());

    for i in 0..df.height() {
        let ts_ms = datetimes.get(i).unwrap_or(0);
        let dt = chrono::DateTime::from_timestamp_millis(ts_ms)
            .map(|utc| utc.with_timezone(&chrono::Local).naive_local())
            .unwrap_or_default();

        records.push(HistoryRecord::new(
            dt.format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
            tag_names.get(i).unwrap_or("").to_string(),
            tag_vals.get(i).unwrap_or(0.0),
            tag_qualities.get(i).unwrap_or("").to_string(),
        ));
    }

    debug!(target: "industry_vis::processing",
        "转换 DataFrame ({} 行) 为记录", records.len());

    Ok(records)
}

/// 使用 Polars lazy API 处理数据
pub fn process_data_polars(
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

    let mut all_results = Vec::new();

    for (tag_name, tag_records) in tag_groups {
        match process_tag_data_polars(tag_records, config) {
            Ok(processed) => all_results.extend(processed),
            Err(e) => {
                warn!(target: "industry_vis::processing",
                    "Polars 处理标签 {} 失败: {}", tag_name, e);
                // 回退时跳过该标签
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
    let df = records_to_dataframe(&records)?;
    let mut lf = df.lazy();

    // 1. 异常值剔除
    if config.outlier_removal.enabled {
        lf = remove_outliers_polars(lf)?;
    }

    // 2. 平滑滤波
    if config.smoothing.enabled && config.smoothing.window > 1 {
        lf = smooth_data_polars(lf, config.smoothing.window)?;
    }

    // 收集结果
    let result_df = lf
        .collect()
        .map_err(|e| AppError::DataProcessing(format!("Polars 执行失败: {}", e)))?;

    // 3. 重采样
    let final_df = if config.resample.enabled && config.resample.interval > 0 {
        resample_data_polars(&result_df, config.resample.interval)?
    } else {
        result_df
    };

    dataframe_to_records(&final_df)
}

/// Polars 版本的 3σ 异常值剔除
fn remove_outliers_polars(lf: LazyFrame) -> AppResult<LazyFrame> {
    let result = lf
        .with_columns([
            col("tag_val").mean().alias("_mean"),
            col("tag_val").std(1).alias("_std"),
        ])
        .filter(
            col("tag_val")
                .gt_eq(col("_mean") - lit(3.0) * col("_std"))
                .and(col("tag_val").lt_eq(col("_mean") + lit(3.0) * col("_std"))),
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
        .with_columns([col("tag_val").rolling_mean(options).alias("tag_val")]);

    Ok(result)
}

/// Polars 版本的时间序列重采样
fn resample_data_polars(df: &DataFrame, interval_seconds: u32) -> AppResult<DataFrame> {
    let interval_ms = interval_seconds as i64 * 1000;

    let result = df
        .clone()
        .lazy()
        .with_columns([(col("datetime").cast(DataType::Int64) / lit(interval_ms)
            * lit(interval_ms))
        .cast(DataType::Datetime(TimeUnit::Milliseconds, None))
        .alias("datetime")])
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

/// 解析时间字符串为毫秒时间戳
fn parse_timestamp_ms(date_time: &str) -> Option<i64> {
    use chrono::{Local, TimeZone};

    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_time, "%Y-%m-%dT%H:%M:%S%.3f") {
        if let Some(local_dt) = Local.from_local_datetime(&dt).single() {
            return Some(local_dt.timestamp_millis());
        }
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_time, "%Y-%m-%dT%H:%M:%S") {
        if let Some(local_dt) = Local.from_local_datetime(&dt).single() {
            return Some(local_dt.timestamp_millis());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records(count: usize, tags: usize) -> Vec<HistoryRecord> {
        let base_time = chrono::NaiveDateTime::parse_from_str(
            "2024-01-01T00:00:00.000",
            "%Y-%m-%dT%H:%M:%S%.3f",
        )
        .unwrap();

        let mut records = Vec::with_capacity(count * tags);
        for tag_idx in 0..tags {
            for i in 0..count {
                let dt = base_time + chrono::Duration::seconds(i as i64);
                records.push(HistoryRecord::new(
                    dt.format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
                    format!("Tag{}", tag_idx),
                    (i as f64) * 0.1 + (tag_idx as f64) * 100.0,
                    "Good".to_string(),
                ));
            }
        }
        records
    }

    #[test]
    fn test_records_to_dataframe() {
        let records = create_test_records(100, 2);
        let df = records_to_dataframe(&records).unwrap();
        assert_eq!(df.height(), 200);
    }

    #[test]
    fn test_dataframe_to_records() {
        let records = create_test_records(100, 1);
        let df = records_to_dataframe(&records).unwrap();
        let result = dataframe_to_records(&df).unwrap();
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_process_data_polars() {
        let records = create_test_records(100, 2);
        let config = DataProcessingConfig::new()
            .with_outlier_removal("3sigma")
            .with_smoothing(5, "moving_avg");

        let result = process_data_polars(records, &config).unwrap();
        assert!(!result.is_empty());
    }
}
