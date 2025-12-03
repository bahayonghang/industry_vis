use serde::{Deserialize, Serialize};

/// 历史表记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub date_time: String,
    pub tag_name: String,
    pub tag_val: f64,
    pub tag_quality: String,
}

/// 查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub start_time: String,
    pub end_time: String,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

/// 查询结果 (V1 兼容格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub records: Vec<HistoryRecord>,
    pub total: usize,
}

/// 图表系列数据 (V2 格式，按标签预分组)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartSeriesData {
    /// 标签名称
    pub tag_name: String,
    /// 数据点 [[timestamp_ms, value], ...]
    pub data: Vec<[f64; 2]>,
}

/// 查询结果 V2 (预分组格式，优化前端渲染)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResultV2 {
    /// 按标签分组的系列数据
    pub series: Vec<ChartSeriesData>,
    /// 原始数据总量
    pub total_raw: usize,
    /// 处理后数据量
    pub total_processed: usize,
    /// 是否命中缓存
    pub cache_hit: bool,
    /// 查询耗时（毫秒）
    pub query_time_ms: u64,
}

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
}

/// 异常值剔除配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutlierRemovalConfig {
    pub enabled: bool,
    #[serde(default = "default_outlier_method")]
    pub method: String,  // "3sigma"
}

fn default_outlier_method() -> String {
    "3sigma".to_string()
}

/// 重采样配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResampleConfig {
    pub enabled: bool,
    #[serde(default = "default_resample_interval")]
    pub interval: u32,  // 秒
    #[serde(default = "default_resample_method")]
    pub method: String,  // "mean"
}

fn default_resample_interval() -> u32 {
    60  // 默认1分钟
}

fn default_resample_method() -> String {
    "mean".to_string()
}

/// 平滑滤波配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SmoothingConfig {
    pub enabled: bool,
    #[serde(default = "default_smoothing_method")]
    pub method: String,  // "moving_avg"
    #[serde(default = "default_smoothing_window")]
    pub window: usize,   // 窗口大小
}

fn default_smoothing_method() -> String {
    "moving_avg".to_string()
}

fn default_smoothing_window() -> usize {
    5
}

/// 数据处理配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataProcessingConfig {
    #[serde(default)]
    pub outlier_removal: OutlierRemovalConfig,
    #[serde(default)]
    pub resample: ResampleConfig,
    #[serde(default)]
    pub smoothing: SmoothingConfig,
}
