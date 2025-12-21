---
layout: home

hero:
  name: Industry Vis
  text: 工业数据可视化系统
  tagline: 基于 Tauri 的高性能工业数据可视化桌面应用
  image:
    src: /logo.svg
    alt: Industry Vis
  actions:
    - theme: brand
      text: 快速开始 →
      link: /guide/getting-started
    - theme: alt
      text: 查看 GitHub
      link: https://github.com/your-repo/industry-vis

features:
  - icon: 🔌
    title: SQL Server 连接
    details: 支持 SQL Server 数据库连接，使用 SQL 认证方式安全访问工业数据，支持中文标签模糊搜索。
    link: /guide/data-query
    linkText: 了解更多
  - icon: 📊
    title: 时序数据可视化
    details: 基于 ECharts 的折线图展示，支持缩放、平移、多标签对比和标签分组管理。
    link: /guide/visualization
    linkText: 了解更多
  - icon: 🧹
    title: 数据处理
    details: 内置异常值剔除（3σ法则）、时间序列重采样（均值聚合）、平滑滤波（移动平均）。
    link: /guide/data-processing
    linkText: 了解更多
  - icon: ⚙️
    title: 灵活配置
    details: 支持便携模式和安装模式，配置自动持久化，明暗主题切换。
    link: /guide/configuration
    linkText: 了解更多
  - icon: 🎨
    title: 现代化设计系统
    details: 统一设计令牌系统，玻璃拟物风格，支持明暗主题切换，性能优化。
    link: /guide/styling
    linkText: 了解更多
---

<div class="vp-doc" style="padding: 2rem;">

## 技术栈

| 层级 | 技术 | 版本 | 说明 |
|------|------|------|------|
| **桌面框架** | Tauri | 2.1.1 | 轻量级跨平台桌面应用框架 |
| **后端** | Rust + Polars | 1.75+ | 高性能数据处理引擎 |
| **前端** | Vue 3 + Vite + TypeScript | 3.5.13 | 现代化前端技术栈 |
| **UI 组件** | Naive UI | 2.40.1 | 企业级 Vue 3 组件库 |
| **图表** | ECharts | 5.5.1 | 功能强大的可视化图表库 |
| **数据库** | SQL Server (tiberius) | 2019+ | 工业数据存储与查询 |
| **设计系统** | CSS Tokens + Glassmorphism | 1.0.0 | 统一设计令牌系统，现代化视觉风格 |

</div>
