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

- Node.js >= 18
- Rust >= 1.70
- Windows 10/11 (WebView2)
- [Just](https://github.com/casey/just) (optional, for command simplification)

### Install Dependencies

```bash
npm install
```

### Development Mode

```bash
# Using just
just dev

# Or using npm directly
npm run tauri:dev
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
