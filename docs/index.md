---
layout: home

hero:
  name: Industry Vis
  text: 工业数据可视化系统
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
    details: 支持 SQL Server 数据库连接，使用 SQL 认证方式安全访问工业数据，支持中文标签模糊搜索
  - icon: 📊
    title: 时序数据可视化
    details: 基于 ECharts 的折线图展示，支持缩放、平移、多标签对比和标签分组管理
  - icon: 🧹
    title: 数据处理
    details: 内置异常值剔除（3σ法则）、时间序列重采样（均值聚合）、平滑滤波（移动平均）
  - icon: ⚙️
    title: 灵活配置
    details: 支持便携模式和安装模式，配置自动持久化，明暗主题切换
---

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x |
| 后端 | Rust + Polars |
| 前端 | Vue 3 + Vite + TypeScript |
| UI 组件 | Naive UI |
| 图表 | ECharts |
| 数据库 | SQL Server (tiberius) |
