# Industry Vis

A Tauri-based industrial data visualization system for reading, processing, and displaying industrial operational data.

[简体中文](./README_CN.md) | English

## Features

### Data Connection
- 🔌 SQL Server database connection (SQL Authentication)
- 🔍 Tag fuzzy search (supports Chinese)

### Data Visualization
- 📊 Time-series line charts (ECharts)
- 🕐 Time range selection (presets + custom)
- 🏷️ Tag group management (up to 20 tags per group)

### Data Processing
- 🧹 Outlier removal (3σ rule)
- 📉 Time-series resampling (mean aggregation)
- 📈 Smoothing filter (moving average)
- ⚡ Data downsampling (auto-optimization for rendering)

### Other
- 💾 CSV data export
- ⚙️ Persistent configuration (portable/installed mode)
- 🌓 Light/Dark theme toggle

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Framework | Tauri 2.x |
| Backend | Rust + Polars |
| Frontend | Vue 3 + Vite + TypeScript |
| UI Components | Naive UI |
| Charts | ECharts |
| Database | SQL Server (tiberius) |

## Quick Start

### Requirements

- [Bun](https://bun.sh) >= 1.0 (or Node.js >= 18)
- Rust >= 1.70
- Windows 10/11
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (required, install if app fails to start)
- [Just](https://github.com/casey/just) (optional, for command simplification)

### Install Dependencies

```bash
bun install
```

### Development Mode

```bash
# Using just
just dev

# Or using bun directly
bun run tauri:dev
```

### Build

```bash
# Quick portable build (for daily development)
just build
# Output: src-tauri/target/release/industry-vis.exe

# Build installer (for release)
just release
# Output: src-tauri/target/release/bundle/nsis/Industry Vis_x.x.x_x64-setup.exe
```

## Configuration

After first launch, configure the database connection in the **Settings** page:

| Setting | Description | Default |
|---------|-------------|---------|
| Server | SQL Server address | localhost |
| Port | Database port | 1433 |
| Database | Database name | 控制器数据库 |
| Username | SQL authentication username | sa |
| Default Table | History data table name | 历史表 |

Configuration file location:
- **Portable mode**: `config.toml` in the same directory as exe
- **Installed mode**: `%APPDATA%\IndustryVis\config.toml`

## Database Schema

### History Table

| Column | Type | Description |
|--------|------|-------------|
| DateTime | datetime | Timestamp |
| TagName | nvarchar(50) | Tag name |
| TagVal | real | Value |
| TagQuality | nchar(10) | Quality flag |

### TagDataBase (for tag search)

| Column | Type | Description |
|--------|------|-------------|
| TagName | nvarchar(50) | Tag name |

## Custom Schema Profile

The system supports different database schemas from various vendors through **Schema Profile**. This allows you to adapt to different table structures and column names while keeping data processing and visualization consistent.

### Configuration

Add `schema` section in `config.toml`:

```toml
[schema]
profile = "default"  # Profile name, default is "default"
```

### Available Profiles

| Profile | Description |
|---------|-------------|
| `default` | Default profile for current vendor (TagDataBase + 历史表) |

### Adding a New Profile

To support a new vendor's database schema, follow these steps:

1. **Create Profile file**: `src-tauri/src/datasource/profiles/vendor_x.rs`

```rust
use crate::datasource::SchemaProfile;
use crate::error::AppResult;
use crate::models::HistoryRecord;

pub struct VendorXProfile;

impl SchemaProfile for VendorXProfile {
    fn name(&self) -> &str { "vendor_x" }
    
    fn tag_search_sql(&self, limit: usize) -> String {
        // Return SQL for tag search
        format!("SELECT TOP {} TagName FROM YourTagTable WHERE TagName LIKE @P1", limit)
    }
    
    fn history_query_sql(&self, table: &str, start: &str, end: &str, filter: &str) -> String {
        // Return SQL for history query
        format!("SELECT time_col, tag_col, value_col, quality_col FROM [{}] WHERE ...", table)
    }
    
    fn map_history_row(&self, row: &tiberius::Row) -> AppResult<HistoryRecord> {
        // Map database row to HistoryRecord
        Ok(HistoryRecord::new(/* ... */))
    }
}
```

2. **Register in ProfileRegistry**: `src-tauri/src/datasource/profiles/registry.rs`

```rust
pub fn get(name: &str) -> AppResult<Arc<dyn SchemaProfile>> {
    match name {
        "default" => Ok(Arc::new(DefaultProfile::new())),
        "vendor_x" => Ok(Arc::new(VendorXProfile::new())),  // Add this line
        _ => Err(AppError::Config(format!("Unknown profile: {}", name))),
    }
}
```

3. **Update config** and restart the application.

For detailed documentation, see [Schema Profile Guide](./docs/guide/schema-profile.md).

## Project Structure

```
industry_vis/
├── src/                      # Vue frontend source
│   ├── components/           # Components
│   │   ├── GroupEditView.vue # Group editing (with data processing)
│   │   ├── LineChart.vue     # Line chart
│   │   └── TagSearchModal.vue# Tag search modal
│   ├── views/                # Pages
│   ├── stores/               # Pinia state management
│   └── types/                # TypeScript types
├── src-tauri/                # Rust backend source
│   ├── src/
│   │   ├── commands.rs       # Tauri commands
│   │   ├── config.rs         # Configuration management
│   │   ├── data_processing.rs# Data processing module
│   │   ├── datasource/       # Data source abstraction
│   │   ├── models.rs         # Data models
│   │   └── tag_group.rs      # Tag group management
│   └── Cargo.toml
├── docs/                     # VitePress documentation
├── justfile                  # Just command configuration
└── package.json
```

## Documentation

For full documentation, visit the [docs/](./docs/) directory or run:

```bash
just docs
```

## License

MIT
