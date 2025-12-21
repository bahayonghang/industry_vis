---
layout: home

hero:
  name: Industry Vis
  text: Industrial Data Visualization
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
    details: Connect to SQL Server database with SQL authentication, supports Chinese tag fuzzy search
  - icon: 📊
    title: Time Series Visualization
    details: ECharts-based line charts with zoom, pan, multi-tag comparison and tag group management
  - icon: 🧹
    title: Data Processing
    details: Built-in outlier removal (3σ rule), time-series resampling (mean aggregation), smoothing filter (moving average)
  - icon: ⚙️
    title: Flexible Configuration
    details: Supports portable and installed modes, auto-persistent configuration, light/dark theme toggle
  - icon: 🎨
    title: Modern Design System
    details: Unified design token system, glassmorphism style, theme switching with performance optimization
---

## Tech Stack

| Layer | Technology | Version | Description |
|-------|------------|---------|-------------|
| Desktop Framework | Tauri | 2.1.1 | Lightweight cross-platform desktop framework |
| Backend | Rust + Polars | 1.75+ | High-performance data processing engine |
| Frontend | Vue 3 + Vite + TypeScript | 3.5.13 | Modern frontend technology stack |
| UI Components | Naive UI | 2.40.1 | Enterprise Vue 3 component library |
| Charts | ECharts | 5.5.1 | Powerful visualization chart library |
| Database | SQL Server (tiberius) | 2019+ | Industrial data storage and query |
| Design System | CSS Tokens + Glassmorphism | 1.0.0 | Unified design token system with modern visual style |
