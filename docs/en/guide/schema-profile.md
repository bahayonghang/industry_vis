# Schema Profile Guide

## Overview

Schema Profile is an abstraction layer for adapting to different vendor database structures. By implementing the `SchemaProfile` trait, you can:

- Define custom SQL query templates
- Map different table structures and column names
- Keep data processing and visualization logic unchanged

## Architecture

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
│  │           (uses injected Profile)                │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Configuration

### Basic Configuration

Specify the Profile to use in `config.toml`:

```toml
[schema]
profile = "default"
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `profile` | string | `"default"` | Profile name |

## SchemaProfile Trait

```rust
pub trait SchemaProfile: Send + Sync {
    /// Profile identifier name
    fn name(&self) -> &str;

    /// Generate tag search SQL
    /// @P1 is the parameter placeholder for search keyword (LIKE pattern)
    fn tag_search_sql(&self, limit: usize) -> String;

    /// Generate history data query SQL
    fn history_query_sql(
        &self,
        table: &str,
        start_time: &str,
        end_time: &str,
        tag_filter: &str,
    ) -> String;

    /// Map database row to HistoryRecord
    fn map_history_row(&self, row: &tiberius::Row) -> AppResult<HistoryRecord>;

    /// Build tag filter clause (has default implementation)
    fn build_tag_filter(&self, tags: Option<&[String]>) -> String;

    /// Column name configuration (has default implementations)
    fn tag_column_name(&self) -> &str;      // Default "TagName"
    fn datetime_column_name(&self) -> &str; // Default "DateTime"
    fn value_column_name(&self) -> &str;    // Default "TagVal"
    fn quality_column_name(&self) -> &str;  // Default "TagQuality"
}
```

## Adding a New Profile

### Step 1: Create Profile File

Create a new file in `src-tauri/src/datasource/profiles/`:

```rust
// src-tauri/src/datasource/profiles/vendor_x.rs

use crate::datasource::SchemaProfile;
use crate::error::AppResult;
use crate::models::HistoryRecord;

/// VendorX Schema Profile
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
        // VendorX uses different tag table and field names
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
        // VendorX uses different field names
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
            row.get::<f64, _>(2).unwrap_or(0.0),  // VendorX uses f64
            row.get::<&str, _>(3).unwrap_or("").trim().to_string(),
        ))
    }

    // Override default column names
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

### Step 2: Export in Module

Update `src-tauri/src/datasource/profiles/mod.rs`:

```rust
mod default;
mod registry;
mod vendor_x;  // Add new module

pub use default::DefaultProfile;
pub use registry::ProfileRegistry;
pub use vendor_x::VendorXProfile;  // Export new Profile
```

### Step 3: Register in ProfileRegistry

Update `src-tauri/src/datasource/profiles/registry.rs`:

```rust
use super::{DefaultProfile, VendorXProfile};

impl ProfileRegistry {
    pub fn get(name: &str) -> AppResult<Arc<dyn SchemaProfile>> {
        match name {
            "default" => Ok(Arc::new(DefaultProfile::new())),
            "vendor_x" => Ok(Arc::new(VendorXProfile::new())),  // Register new Profile
            _ => Err(AppError::Config(format!(
                "Unknown Schema Profile: '{}'. Available profiles: default, vendor_x",
                name
            ))),
        }
    }

    pub fn available_profiles() -> &'static [&'static str] {
        &["default", "vendor_x"]  // Update available list
    }
}
```

### Step 4: Update Configuration

Switch Profile in `config.toml`:

```toml
[schema]
profile = "vendor_x"
```

### Step 5: Restart Application

After restarting the application, the system will use the new Profile for data queries.

## Testing New Profile

It's recommended to add unit tests for new Profiles:

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

## Best Practices

### SQL Security

- Always escape `]` characters in table names: `table.replace(']', "]]")`
- Always escape single quotes in time strings: `time.replace('\'', "''")`
- Use parameterized queries (`@P1`) for user input

### Performance Optimization

- Use `WITH (NOLOCK)` to reduce lock contention (for SQL Server)
- Select only necessary columns
- Ensure time fields are indexed

### Backward Compatibility

- Adding new Profiles doesn't affect existing configurations
- Uses `"default"` when no Profile is specified
- Automatically falls back to default Profile if loading fails

## FAQ

### Q: How to debug SQL queries?

Enable debug log level, SQL statements will be logged:

```toml
# Set in tauri.conf.json or environment variable
RUST_LOG=industry_vis::datasource=debug
```

### Q: New Profile not taking effect?

1. Check if the `profile` name in `config.toml` is correct
2. Confirm it's registered in `ProfileRegistry`
3. Restart the application for configuration to take effect

### Q: How to handle different data types?

Convert according to actual database types in `map_history_row`:

```rust
// Integer to f64
row.get::<i32, _>(2).unwrap_or(0) as f64

// String to f64
row.get::<&str, _>(2)
    .and_then(|s| s.parse::<f64>().ok())
    .unwrap_or(0.0)
```
