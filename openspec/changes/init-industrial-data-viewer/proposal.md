# Change: 初始化工业数据查看系统

## Why
工业现场需要一个轻量、高性能的本地桌面应用，用于查询 SQL Server 等数据源中的运行数据，并进行处理和可视化展示。现有方案（如 Electron）包体积大、性能差，无法满足工业场景需求。

## What Changes
- 创建基于 Tauri 的桌面应用脚手架
- 实现数据源抽象层，支持 SQL Server（SQL 认证）
- 支持 TOML 配置文件配置数据库连接
- 默认查询【控制器数据库】的【历史表】
- 集成 Polars 进行高性能数据处理
- 构建前端数据展示与折线图组件（支持时间范围选择）
- 实现 CSV 数据导出功能

## Impact
- **新增 specs**:
  - `desktop-app`: Tauri 桌面应用框架
  - `data-source`: 数据源管理与连接
  - `data-processing`: 数据处理引擎
  - `data-visualization`: 数据展示与图表
- **Affected code**: 全新项目，无现有代码影响
- **Dependencies**: Tauri 2.x, Polars, tiberius, Vue 3, ECharts

## Success Criteria
- [ ] 可通过 TOML 配置连接 SQL Server
- [ ] 默认查询【控制器数据库.历史表】的时序数据
- [ ] 查询结果可在表格中展示（DateTime, TagName, TagVal, TagQuality）
- [ ] 支持折线图展示，可选择时间范围和标签
- [ ] 数据可导出为 CSV
- [ ] 应用打包体积 < 30MB

## Domain Data Model
**历史表结构**:
| Column | Type | Description |
|--------|------|-------------|
| DateTime | datetime | 时间戳 |
| TagName | nchar(50) | 标签名称（如 C1A1出口温度） |
| TagVal | real | 数值 |
| TagQuality | nchar(10) | 质量标志（如 未连接） |
