# Introduction

Industry Vis is an industrial data viewing system built with Tauri, designed for reading, processing, and displaying industrial operational data.

## Main Features

- **SQL Server Connection** - Connect to SQL Server database to read industrial history data
- **Time Range Selection** - Preset ranges (1h/6h/24h/7d) and custom time ranges
- **Tag Filtering** - Multi-select tags with search functionality
- **Data Table** - Virtual scrolling table showing time, tag name, value, and quality
- **Line Chart** - Multi-series line chart with zoom, pan, and tooltips
- **CSV Export** - Export filtered data to CSV file

## Data Model

The system queries the `历史表` (History Table) in `控制器数据库` (Controller Database) by default:

| Column | Type | Description |
|--------|------|-------------|
| DateTime | datetime | Timestamp |
| TagName | nchar(50) | Tag name |
| TagVal | real | Value |
| TagQuality | nchar(10) | Quality flag |

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                 Frontend (Vue 3)                     │
│  ┌─────────────┬─────────────┬─────────────────┐   │
│  │ TimeRange   │ TagSelector │ DataTable/Chart │   │
│  └─────────────┴─────────────┴─────────────────┘   │
└────────────────────────┬────────────────────────────┘
                         │ Tauri IPC
┌────────────────────────┴────────────────────────────┐
│                   Backend (Rust)                     │
│  ┌──────────────┬──────────────┬────────────────┐  │
│  │ Config (TOML)│ DataSource   │ Commands       │  │
│  └──────────────┴──────────────┴────────────────┘  │
└────────────────────────┬────────────────────────────┘
                         │ tiberius
                    ┌────┴────┐
                    │SQL Server│
                    └─────────┘
```

## Next Steps

- [Quick Start](/en/guide/getting-started) - Install and run the project
- [Configuration](/en/guide/configuration) - Configure database connection
