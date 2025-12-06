//! 默认 Schema Profile
//!
//! 实现当前厂商（控制器数据库）的表结构和字段映射。

use crate::datasource::SchemaProfile;
use crate::error::AppResult;
use crate::models::HistoryRecord;

/// 默认 Schema Profile
///
/// 适配当前厂商的数据库结构：
/// - 标签表：`TagDataBase`，字段 `TagName`
/// - 历史表：可配置（默认 `历史表`），字段 `DateTime, TagName, TagVal, TagQuality`
#[derive(Debug, Clone, Default)]
pub struct DefaultProfile;

impl DefaultProfile {
    /// 创建新的默认 Profile
    pub fn new() -> Self {
        Self
    }
}

impl SchemaProfile for DefaultProfile {
    fn name(&self) -> &str {
        "default"
    }

    fn tag_search_sql(&self, limit: usize) -> String {
        format!(
            r#"SELECT DISTINCT TOP {} TagName 
               FROM [TagDataBase] 
               WHERE TagName LIKE @P1
               ORDER BY TagName"#,
            limit
        )
    }

    fn history_query_sql(
        &self,
        table: &str,
        start_time: &str,
        end_time: &str,
        tag_filter: &str,
    ) -> String {
        // 优化 SQL：
        // 1. 使用 WITH (NOLOCK) 减少锁等待
        // 2. 只按 DateTime 排序，充分利用索引
        format!(
            r#"SELECT DateTime, TagName, TagVal, TagQuality 
               FROM [{}] WITH (NOLOCK)
               WHERE DateTime BETWEEN '{}' AND '{}'
               {}
               ORDER BY DateTime"#,
            table.replace(']', "]]"),
            start_time.replace('\'', "''"),
            end_time.replace('\'', "''"),
            tag_filter
        )
    }

    fn map_history_row(&self, row: &tiberius::Row) -> AppResult<HistoryRecord> {
        let dt: Option<chrono::NaiveDateTime> = row.get(0);
        let date_time = dt
            .map(|d| d.format("%Y-%m-%dT%H:%M:%S%.3f").to_string())
            .unwrap_or_default();

        Ok(HistoryRecord::new(
            date_time,
            row.get::<&str, _>(1).unwrap_or("").trim().to_string(),
            row.get::<f32, _>(2).unwrap_or(0.0) as f64,
            row.get::<&str, _>(3).unwrap_or("").trim().to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile_name() {
        let profile = DefaultProfile::new();
        assert_eq!(profile.name(), "default");
    }

    #[test]
    fn test_tag_search_sql_format() {
        let profile = DefaultProfile::new();
        let sql = profile.tag_search_sql(100);

        assert!(sql.contains("TOP 100"));
        assert!(sql.contains("[TagDataBase]"));
        assert!(sql.contains("TagName LIKE @P1"));
        assert!(sql.contains("ORDER BY TagName"));
    }

    #[test]
    fn test_history_query_sql_format() {
        let profile = DefaultProfile::new();
        let sql =
            profile.history_query_sql("历史表", "2024-01-01T00:00:00", "2024-01-02T00:00:00", "");

        assert!(sql.contains("[历史表]"));
        assert!(sql.contains("WITH (NOLOCK)"));
        assert!(sql.contains("DateTime BETWEEN"));
        assert!(sql.contains("ORDER BY DateTime"));
    }

    #[test]
    fn test_history_query_sql_with_tag_filter() {
        let profile = DefaultProfile::new();
        let filter = profile.build_tag_filter(Some(&["Tag1".to_string(), "Tag2".to_string()]));
        let sql = profile.history_query_sql(
            "历史表",
            "2024-01-01T00:00:00",
            "2024-01-02T00:00:00",
            &filter,
        );

        assert!(sql.contains("AND TagName IN"));
        assert!(sql.contains("'Tag1'"));
        assert!(sql.contains("'Tag2'"));
    }

    #[test]
    fn test_history_query_sql_escapes_table_name() {
        let profile = DefaultProfile::new();
        let sql = profile.history_query_sql(
            "Table]Name",
            "2024-01-01T00:00:00",
            "2024-01-02T00:00:00",
            "",
        );

        // ] 应该被转义为 ]]
        assert!(sql.contains("[Table]]Name]"));
    }

    #[test]
    fn test_history_query_sql_escapes_time_quotes() {
        let profile = DefaultProfile::new();
        let sql =
            profile.history_query_sql("历史表", "2024-01-01'T00:00:00", "2024-01-02T00:00:00", "");

        // ' 应该被转义为 ''
        assert!(sql.contains("2024-01-01''T00:00:00"));
    }
}
