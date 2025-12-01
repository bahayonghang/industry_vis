# Tasks: 初始化工业数据查看系统

## Phase 1: 项目脚手架与技术验证
- [x] 1.1 创建 Tauri + Vue 3 项目脚手架
- [x] 1.2 配置 Rust 依赖（polars, tiberius, tokio, serde, toml）
- [x] 1.3 配置前端依赖（naive-ui, echarts, pinia）
- [x] 1.4 验证 tiberius 连接 SQL Server（PoC）
- [x] 1.5 验证 Polars DataFrame 序列化到前端（PoC）

## Phase 2: 配置与数据源
- [x] 2.1 定义 TOML 配置结构（AppConfig, DatabaseConfig, QueryConfig）
- [x] 2.2 实现配置文件加载与默认值
- [x] 2.3 定义 DataSource trait 接口
- [x] 2.4 实现 SQL Server 连接器（SqlServerSource，仅 SQL 认证）
- [x] 2.5 实现连接测试功能
- [x] 2.6 实现默认查询：控制器数据库.历史表

## Phase 3: 查询执行与数据展示
- [x] 3.1 实现历史表查询 Tauri 命令（支持时间范围和标签过滤）
- [x] 3.2 实现查询结果分页
- [x] 3.3 前端：时间范围选择器（预设 + 自定义）
- [x] 3.4 前端：标签选择器（多选 + 搜索）
- [x] 3.5 前端：数据表格组件（虚拟滚动，显示 DateTime/TagName/TagVal/TagQuality）
- [ ] 3.6 前端：查询历史记录

## Phase 4: 数据处理
- [x] 4.1 实现 Polars 数据处理管道（filter by time, filter by tags, sort）
- [ ] 4.2 实现数据聚合（按时间窗口）
- [ ] 4.3 前端：数据处理配置 UI（可选）

## Phase 5: 数据可视化（折线图）
- [x] 5.1 集成 ECharts
- [x] 5.2 实现折线图组件（X: DateTime, Y: TagVal）
- [x] 5.3 实现多标签多系列展示
- [x] 5.4 实现缩放平移交互
- [x] 5.5 实现数据点 tooltip（含 TagQuality）
- [x] 5.6 前端：图例开关

## Phase 6: 数据导出
- [x] 6.1 实现 CSV 导出（支持筛选后数据）
- [x] 6.2 前端：导出按钮与文件保存对话框

## Phase 7: 测试与打包
- [x] 7.1 Rust 单元测试（配置加载、数据源、查询）
- [ ] 7.2 前端组件测试
- [ ] 7.3 E2E 测试（Playwright）
- [ ] 7.4 Windows 打包与测试
- [x] 7.5 编写 README 与配置说明

## Dependencies
- Phase 2 依赖 Phase 1 完成
- Phase 3-5 可部分并行（前端 mock 数据）
- Phase 6-7 依赖主要功能完成
