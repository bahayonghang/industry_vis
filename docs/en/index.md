---
layout: home

hero:
  name: Industry Vis
  text: Industrial Data Viewer
  tagline: High-performance industrial data visualization desktop application built with Tauri
  actions:
    - theme: brand
      text: Get Started
      link: /en/guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/your-repo/industry-vis

features:
  - icon: 🔌
    title: SQL Server Connection
    details: Connect to SQL Server database with SQL authentication for secure industrial data access
  - icon: 📊
    title: Time Series Visualization
    details: ECharts-based line charts with zoom, pan, and multi-tag comparison
  - icon: 📋
    title: High-Performance Data Table
    details: Virtual scrolling technology handles large datasets with sorting and filtering
  - icon: ⚙️
    title: Flexible Configuration
    details: TOML configuration file with default database and table settings
---

## Tech Stack

- **Desktop Framework**: Tauri 2.x
- **Backend**: Rust + Polars
- **Frontend**: Vue 3 + Vite + Naive UI + ECharts
- **Database**: SQL Server (tiberius)
