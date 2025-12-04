//! 历史记录数据模型

use serde::{Deserialize, Serialize};

/// 历史表记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub date_time: String,
    pub tag_name: String,
    pub tag_val: f64,
    pub tag_quality: String,
}

impl HistoryRecord {
    /// 创建新记录
    pub fn new(date_time: String, tag_name: String, tag_val: f64, tag_quality: String) -> Self {
        Self {
            date_time,
            tag_name,
            tag_val,
            tag_quality,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_record_creation() {
        let record = HistoryRecord::new(
            "2024-01-01T00:00:00.000".to_string(),
            "Tag1".to_string(),
            123.45,
            "Good".to_string(),
        );
        assert_eq!(record.tag_name, "Tag1");
        assert_eq!(record.tag_val, 123.45);
    }

    #[test]
    fn test_history_record_serialization() {
        let record = HistoryRecord::new(
            "2024-01-01T00:00:00.000".to_string(),
            "Tag1".to_string(),
            123.45,
            "Good".to_string(),
        );
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("dateTime"));
        assert!(json.contains("tagName"));

        let parsed: HistoryRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, record);
    }
}
