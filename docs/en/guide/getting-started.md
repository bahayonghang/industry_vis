# Quick Start

## Requirements

- **Node.js** >= 18
- **Rust** >= 1.70
- **Windows 10/11**
- [**WebView2 Runtime**](https://developer.microsoft.com/microsoft-edge/webview2/) (required, install if app fails to start)

## Install Dependencies

```bash
# Clone the project
git clone https://github.com/your-repo/industry-vis.git
cd industry-vis

# Install frontend dependencies
npm install

# Rust dependencies are installed automatically on first build
```

## Configure Database

1. Copy the configuration template:

```bash
cp config.example.toml config.toml
```

2. Edit `config.toml` with your database connection info:

```toml
[database]
server = "localhost"
port = 1433
database = "控制器数据库"
username = "sa"
password = "your_password"

[query]
default_table = "历史表"
```

## Development Mode

```bash
npm run tauri:dev
```

This starts both the Vite dev server and Tauri application.

## Production Build

```bash
npm run tauri:build
```

Build artifacts are located in `src-tauri/target/release/bundle/`.

## Using Just Commands

If you have [just](https://github.com/casey/just) installed:

```bash
# Development mode
just dev

# Build
just build

# Run tests
just test

# Start docs
just docs
```

## Next Steps

- [Configuration](/en/guide/configuration) - Detailed configuration guide
- [Data Query](/en/guide/data-query) - Using the data query feature
