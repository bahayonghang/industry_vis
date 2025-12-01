# Project Context

## Purpose
工业数据查看系统（Industry Vis）：一个基于 Tauri 的桌面应用，用于读取、处理和展示工业运行数据。

## Tech Stack
- **桌面框架**: Tauri 2.x
- **后端语言**: Rust
- **数据处理**: Polars
- **前端框架**: Vue 3 + Vite
- **UI 组件库**: Naive UI
- **图表库**: ECharts（折线图）
- **数据库连接**: tiberius (SQL Server)
- **配置格式**: TOML

## Project Conventions

### Code Style
- Rust: 遵循 `rustfmt` 默认配置，使用 `clippy` 进行静态检查
- TypeScript/Vue: 使用 ESLint + Prettier，组合式 API（Composition API）
- 命名：Rust 用 snake_case，TypeScript 用 camelCase，组件用 PascalCase

### Architecture Patterns
- **前后端分离**: Tauri IPC 通信，后端处理数据密集型任务
- **数据源抽象**: trait-based 设计，支持多数据源扩展
- **惰性求值**: Polars lazy evaluation 优化大数据处理

### Testing Strategy
- Rust: 单元测试 + 集成测试（cargo test）
- 前端: Vitest 单元测试，Playwright E2E 测试
- 覆盖主流程、异常路径与边界值

### Git Workflow
- 主分支: main
- 功能分支: feature/<name>
- 提交规范: Conventional Commits（feat/fix/docs/refactor/test）

## Domain Context
- **工业数据特点**: 时序数据为主，数据量大，查询模式相对固定
- **典型数据源**: SQL Server（工业现场常见）
- **默认数据库**: 控制器数据库
- **默认表**: 历史表（DateTime, TagName, TagVal, TagQuality）
- **用户场景**: 历史数据查询、趋势分析（折线图）、CSV 导出

## Important Constraints
- 桌面应用需支持 Windows 7+ 和 Windows Server
- 数据处理需在本地完成，不依赖云服务
- 敏感数据不外传，连接信息本地加密存储

## External Dependencies
- SQL Server 数据库（客户现有系统，含控制器数据库.历史表）
