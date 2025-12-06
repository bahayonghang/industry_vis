# Schema Profile 指南

## 概述

Schema Profile 是一个抽象层，用于适配不同厂商的数据库结构。通过实现 `SchemaProfile` trait，您可以：

- 定义自定义的 SQL 查询模板
- 映射不同的表结构和字段名称
- 保持数据处理和可视化逻辑不变

## 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                      AppState                           │
│                         │                               │
│                         ▼                               │
│  ┌─────────────────────────────────────────────────┐   │
│  │              ProfileRegistry                     │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐         │   │
│  │  │ Default │  │VendorA  │  │VendorB  │  ...    │   │
│  │  │ Profile │  │Profile  │  │Profile  │         │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘         │   │
│  └───────┼────────────┼───────────┼───────────────┘   │
│          └────────────┼───────────┘                    │
│                       ▼                                │
│  ┌─────────────────────────────────────────────────┐   │
│  │              SqlServerSource                     │   │
│  │         (使用注入的 Profile)                     │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## 配置

### 基本配置

在 `config.toml` 中指定要使用的 Profile：

```toml
[schema]
profile = "default"
```

### 配置项说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `profile` | string | `"default"` | Profile 名称 |

## SchemaProfile Trait

```rust
pub trait SchemaProfile: Send + Sync {
    /// Profile 标识名
    fn name(&self) -> &str;

    /// 生成标签搜索 SQL
    /// @P1 为搜索关键词的参数占位符（LIKE 模式）
    fn tag_search_sql(&self, limit: usize) -> String;

    /// 生成历史数据查询 SQL
    fn history_query_sql(
        &self,
        table: &str,
        start_time: &str,
        end_time: &str,
        tag_filter: &str,
    ) -> String;

    /// 将数据库行映射为 HistoryRecord
    fn map_history_row(&self, row: &tiberius::Row) -> AppResult<HistoryRecord>;

    /// 生成标签过滤条件（有默认实现）
    fn build_tag_filter(&self, tags: Option<&[String]>) -> String;

    /// 列名配置（有默认实现）
    fn tag_column_name(&self) -> &str;      // 默认 "TagName"
    fn datetime_column_name(&self) -> &str; // 默认 "DateTime"
    fn value_column_name(&self) -> &str;    // 默认 "TagVal"
    fn quality_column_name(&self) -> &str;  // 默认 "TagQuality"
}
```

## 添加新的 Profile

### 步骤 1：创建 Profile 文件

在 `src-tauri/src/datasource/profiles/` 目录下创建新文件：

```rust
// src-tauri/src/datasource/profiles/vendor_x.rs

use crate::datasource::SchemaProfile;
use crate::error::AppResult;
use crate::models::HistoryRecord;

/// VendorX 厂商的 Schema Profile
#[derive(Debug, Clone, Default)]
pub struct VendorXProfile;

impl VendorXProfile {
    pub fn new() -> Self {
        Self
    }
}

impl SchemaProfile for VendorXProfile {
    fn name(&self) -> &str {
        "vendor_x"
    }

    fn tag_search_sql(&self, limit: usize) -> String {
        // VendorX 使用不同的标签表和字段名
        format!(
            r#"SELECT DISTINCT TOP {} PointName 
               FROM [PointDatabase] 
               WHERE PointName LIKE @P1
               ORDER BY PointName"#,
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
        // VendorX 使用不同的字段名
        format!(
            r#"SELECT Timestamp, PointName, Value, Status 
               FROM [{}] WITH (NOLOCK)
               WHERE Timestamp BETWEEN '{}' AND '{}'
               {}
               ORDER BY Timestamp"#,
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
            row.get::<f64, _>(2).unwrap_or(0.0),  // VendorX 使用 f64
            row.get::<&str, _>(3).unwrap_or("").trim().to_string(),
        ))
    }

    // 覆盖默认列名
    fn tag_column_name(&self) -> &str {
        "PointName"
    }

    fn datetime_column_name(&self) -> &str {
        "Timestamp"
    }

    fn value_column_name(&self) -> &str {
        "Value"
    }

    fn quality_column_name(&self) -> &str {
        "Status"
    }
}
```

### 步骤 2：在模块中导出

更新 `src-tauri/src/datasource/profiles/mod.rs`：

```rust
mod default;
mod registry;
mod vendor_x;  // 添加新模块

pub use default::DefaultProfile;
pub use registry::ProfileRegistry;
pub use vendor_x::VendorXProfile;  // 导出新 Profile
```

### 步骤 3：注册到 ProfileRegistry

更新 `src-tauri/src/datasource/profiles/registry.rs`：

```rust
use super::{DefaultProfile, VendorXProfile};

impl ProfileRegistry {
    pub fn get(name: &str) -> AppResult<Arc<dyn SchemaProfile>> {
        match name {
            "default" => Ok(Arc::new(DefaultProfile::new())),
            "vendor_x" => Ok(Arc::new(VendorXProfile::new())),  // 注册新 Profile
            _ => Err(AppError::Config(format!(
                "未知的 Schema Profile: '{}'. 可用的 Profile: default, vendor_x",
                name
            ))),
        }
    }

    pub fn available_profiles() -> &'static [&'static str] {
        &["default", "vendor_x"]  // 更新可用列表
    }
}
```

### 步骤 4：更新配置

在 `config.toml` 中切换 Profile：

```toml
[schema]
profile = "vendor_x"
```

### 步骤 5：重启应用

重启应用后，系统将使用新的 Profile 进行数据查询。

## 测试新 Profile

建议为新 Profile 添加单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_x_profile_name() {
        let profile = VendorXProfile::new();
        assert_eq!(profile.name(), "vendor_x");
    }

    #[test]
    fn test_tag_search_sql() {
        let profile = VendorXProfile::new();
        let sql = profile.tag_search_sql(100);
        assert!(sql.contains("TOP 100"));
        assert!(sql.contains("PointDatabase"));
        assert!(sql.contains("@P1"));
    }

    #[test]
    fn test_history_query_sql() {
        let profile = VendorXProfile::new();
        let sql = profile.history_query_sql(
            "HistoryData",
            "2024-01-01T00:00:00",
            "2024-01-02T00:00:00",
            "",
        );
        assert!(sql.contains("Timestamp"));
        assert!(sql.contains("PointName"));
        assert!(sql.contains("Value"));
    }
}
```

## 最佳实践

### SQL 安全

- 始终转义表名中的 `]` 字符：`table.replace(']', "]]")`
- 始终转义时间字符串中的单引号：`time.replace('\'', "''")`
- 使用参数化查询（`@P1`）处理用户输入

### 性能优化

- 使用 `WITH (NOLOCK)` 减少锁等待（适用于 SQL Server）
- 只选择必要的列
- 确保时间字段有索引

### 向后兼容

- 新增 Profile 不影响现有配置
- 未指定 Profile 时默认使用 `"default"`
- Profile 加载失败时自动回退到默认 Profile

## 常见问题

### Q: 如何调试 SQL 查询？

启用 debug 日志级别，SQL 语句会记录到日志中：

```toml
# 在 tauri.conf.json 或环境变量中设置
RUST_LOG=industry_vis::datasource=debug
```

### Q: 新 Profile 不生效？

1. 检查 `config.toml` 中的 `profile` 名称是否正确
2. 确认已在 `ProfileRegistry` 中注册
3. 重启应用使配置生效

### Q: 如何处理不同的数据类型？

在 `map_history_row` 中根据实际数据库类型进行转换：

```rust
// 整数转 f64
row.get::<i32, _>(2).unwrap_or(0) as f64

// 字符串转 f64
row.get::<&str, _>(2)
    .and_then(|s| s.parse::<f64>().ok())
    .unwrap_or(0.0)
```
