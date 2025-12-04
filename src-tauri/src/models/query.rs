//! 查询相关数据模型

use super::HistoryRecord;
use serde::{Deserialize, Serialize};

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

impl QueryParams {
    /// 创建新的查询参数
    pub fn new(start_time: String, end_time: String) -> Self {
        Self {
            start_time,
            end_time,
            tags: None,
            limit: None,
            offset: None,
        }
    }

    /// 设置标签过滤
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// 设置分页
    pub fn with_pagination(mut self, offset: usize, limit: usize) -> Self {
        self.offset = Some(offset);
        self.limit = Some(limit);
        self
    }
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

impl ConnectionTestResult {
    pub fn success() -> Self {
        Self {
            success: true,
            message: "连接成功".to_string(),
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_params_builder() {
        let params = QueryParams::new(
            "2024-01-01T00:00:00".to_string(),
            "2024-01-02T00:00:00".to_string(),
        )
        .with_tags(vec!["Tag1".to_string(), "Tag2".to_string()])
        .with_pagination(0, 100);

        assert_eq!(params.tags.as_ref().unwrap().len(), 2);
        assert_eq!(params.offset, Some(0));
        assert_eq!(params.limit, Some(100));
    }

    #[test]
    fn test_connection_test_result() {
        let success = ConnectionTestResult::success();
        assert!(success.success);

        let failure = ConnectionTestResult::failure("连接超时".to_string());
        assert!(!failure.success);
        assert_eq!(failure.message, "连接超时");
    }
}
