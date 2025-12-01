---
layout: home

hero:
  name: Industry Vis
  text: 工业数据查看系统
  tagline: 基于 Tauri 的高性能工业数据可视化桌面应用
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 查看 GitHub
      link: https://github.com/your-repo/industry-vis

features:
  - icon: 🔌
    title: SQL Server 连接
    details: 支持 SQL Server 数据库连接，使用 SQL 认证方式安全访问工业数据
  - icon: 📊
    title: 时序数据可视化
    details: 基于 ECharts 的折线图展示，支持缩放、平移和多标签对比
  - icon: 📋
    title: 高性能数据表格
    details: 虚拟滚动技术，轻松处理大量数据，支持排序和筛选
  - icon: ⚙️
    title: 灵活配置
    details: TOML 配置文件，支持默认数据库和表的配置
---

## 技术栈

- **桌面框架**: Tauri 2.x
- **后端**: Rust + Polars
- **前端**: Vue 3 + Vite + Naive UI + ECharts
- **数据库**: SQL Server (tiberius)
