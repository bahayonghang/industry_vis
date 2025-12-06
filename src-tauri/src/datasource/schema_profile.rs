//! Schema Profile 抽象
//!
//! 提供数据库 Schema 配置的抽象接口，支持不同厂商的表结构和字段映射。

use crate::error::AppResult;
use crate::models::HistoryRecord;

/// Schema Profile trait
///
/// 定义数据库 Schema 的配置接口，包括 SQL 模板和字段映射。
/// 不同厂商实现此 trait 以适配各自的表结构。
///
/// # Example
///
/// ```ignore
/// use industry_vis::datasource::SchemaProfile;
///
/// struct CustomProfile;
///
/// impl SchemaProfile for CustomProfile {
///     fn name(&self) -> &str { "custom" }
///     fn tag_search_sql(&self, limit: usize) -> String {
///         format!("SELECT TOP {} TagName FROM MyTagTable WHERE TagName LIKE @P1", limit)
///     }
///     // ... 其他方法
/// }
/// ```
pub trait SchemaProfile: Send + Sync {
    /// Profile 标识名（用于配置选择和日志）
    fn name(&self) -> &str;

    /// 生成标签搜索 SQL
    ///
    /// # Arguments
    /// * `limit` - 返回结果数量限制
    ///
    /// # Returns
    /// SQL 查询字符串，使用 `@P1` 作为搜索关键词的参数占位符（LIKE 模式）
    fn tag_search_sql(&self, limit: usize) -> String;

    /// 生成历史数据查询 SQL
    ///
    /// # Arguments
    /// * `table` - 历史表名（已转义）
    /// * `start_time` - 开始时间字符串
    /// * `end_time` - 结束时间字符串
    /// * `tag_filter` - 可选的标签过滤条件（如 `AND TagName IN ('tag1', 'tag2')`）
    ///
    /// # Returns
    /// SQL 查询字符串
    fn history_query_sql(
        &self,
        table: &str,
        start_time: &str,
        end_time: &str,
        tag_filter: &str,
    ) -> String;

    /// 将数据库行映射为 HistoryRecord
    ///
    /// # Arguments
    /// * `row` - tiberius 查询结果行
    ///
    /// # Returns
    /// 映射后的 HistoryRecord
    fn map_history_row(&self, row: &tiberius::Row) -> AppResult<HistoryRecord>;

    /// 生成标签过滤条件
    ///
    /// # Arguments
    /// * `tags` - 标签列表
    ///
    /// # Returns
    /// SQL WHERE 子句片段（如 `AND TagName IN ('tag1', 'tag2')`），无标签时返回空字符串
    fn build_tag_filter(&self, tags: Option<&[String]>) -> String {
        match tags {
            Some(t) if !t.is_empty() => {
                let tag_list = t
                    .iter()
                    .map(|s| format!("'{}'", s.replace('\'', "''")))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("AND {} IN ({})", self.tag_column_name(), tag_list)
            }
            _ => String::new(),
        }
    }

    /// 获取标签列名（默认 TagName）
    fn tag_column_name(&self) -> &str {
        "TagName"
    }

    /// 获取时间列名（默认 DateTime）
    fn datetime_column_name(&self) -> &str {
        "DateTime"
    }

    /// 获取数值列名（默认 TagVal）
    fn value_column_name(&self) -> &str {
        "TagVal"
    }

    /// 获取质量列名（默认 TagQuality）
    fn quality_column_name(&self) -> &str {
        "TagQuality"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProfile;

    impl SchemaProfile for TestProfile {
        fn name(&self) -> &str {
            "test"
        }

        fn tag_search_sql(&self, limit: usize) -> String {
            format!(
                "SELECT TOP {} TagName FROM TestTags WHERE TagName LIKE @P1",
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
            format!(
                "SELECT * FROM [{}] WHERE DateTime BETWEEN '{}' AND '{}' {}",
                table, start_time, end_time, tag_filter
            )
        }

        fn map_history_row(&self, _row: &tiberius::Row) -> AppResult<HistoryRecord> {
            Ok(HistoryRecord::new(
                "2024-01-01T00:00:00".to_string(),
                "TestTag".to_string(),
                0.0,
                "Good".to_string(),
            ))
        }
    }

    #[test]
    fn test_profile_name() {
        let profile = TestProfile;
        assert_eq!(profile.name(), "test");
    }

    #[test]
    fn test_tag_search_sql() {
        let profile = TestProfile;
        let sql = profile.tag_search_sql(50);
        assert!(sql.contains("TOP 50"));
        assert!(sql.contains("@P1"));
    }

    #[test]
    fn test_build_tag_filter_empty() {
        let profile = TestProfile;
        assert_eq!(profile.build_tag_filter(None), "");
        assert_eq!(profile.build_tag_filter(Some(&[])), "");
    }

    #[test]
    fn test_build_tag_filter_with_tags() {
        let profile = TestProfile;
        let tags = vec!["Tag1".to_string(), "Tag2".to_string()];
        let filter = profile.build_tag_filter(Some(&tags));
        assert!(filter.contains("TagName IN"));
        assert!(filter.contains("'Tag1'"));
        assert!(filter.contains("'Tag2'"));
    }

    #[test]
    fn test_build_tag_filter_escapes_quotes() {
        let profile = TestProfile;
        let tags = vec!["Tag'With'Quotes".to_string()];
        let filter = profile.build_tag_filter(Some(&tags));
        assert!(filter.contains("Tag''With''Quotes"));
    }
}
