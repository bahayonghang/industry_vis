# Design: 工业数据查看系统架构

## Context
工业数据查看系统是一个桌面应用，面向工业现场运维人员和数据分析师。需要在无网络环境下运行，处理大量时序数据，支持多种数据源。

### Stakeholders
- 工业现场运维人员（查询历史数据）
- 数据分析师（数据处理与可视化）
- 系统管理员（配置数据源连接）

### Constraints
- Windows 平台优先（工业现场主流）
- 本地运行，无云依赖
- 敏感数据不可外传

## Goals / Non-Goals

### Goals
- 提供高性能的本地数据查询与处理能力
- 支持多数据源扩展（SQL Server → PostgreSQL → CSV）
- 简洁直观的用户界面
- 打包体积小、启动快

### Non-Goals
- 不做实时数据流处理（第一期）
- 不做多用户协作功能
- 不做云端数据同步

## Decisions

### D1: 桌面框架选择 Tauri
**Decision**: 使用 Tauri 2.x 而非 Electron

**Rationale**:
- 包体积小（~10MB vs Electron ~150MB）
- 内存占用低
- Rust 后端与 Polars 无缝集成
- 安全沙箱机制

**Alternatives considered**:
- Electron: 生态成熟但体积大、性能差
- Flutter Desktop: Dart 生态与数据处理库不匹配
- Native (WPF/Qt): 开发成本高，跨平台困难

### D2: 数据处理引擎选择 Polars
**Decision**: 使用 Polars 而非 Pandas/DataFusion

**Rationale**:
- 纯 Rust 实现，无 Python 依赖
- 多线程并行处理
- 惰性求值优化查询
- 内存效率高（Arrow 格式）

**Alternatives considered**:
- Pandas (via PyO3): 引入 Python 运行时，打包复杂
- DataFusion: 更偏向 SQL 引擎，DataFrame API 不如 Polars 友好

### D3: 数据源抽象设计
**Decision**: 使用 Rust trait 定义数据源接口

```rust
#[async_trait]
pub trait DataSource: Send + Sync {
    /// 执行查询并返回 Polars DataFrame
    async fn query(&self, sql: &str) -> Result<DataFrame, DataSourceError>;
    
    /// 测试连接
    async fn test_connection(&self) -> Result<(), DataSourceError>;
    
    /// 获取数据源元数据
    fn metadata(&self) -> &SourceMetadata;
    
    /// 列出可用表
    async fn list_tables(&self) -> Result<Vec<TableInfo>, DataSourceError>;
}
```

**Rationale**:
- 接口简洁（ISP 原则）
- 易于扩展新数据源
- 类型安全

### D4: 前后端通信策略
**Decision**: 使用 Tauri IPC + JSON 序列化，大数据分页传输

**Rationale**:
- Tauri 原生 `invoke` 机制成熟稳定
- JSON 兼容性好，调试方便
- 分页避免前端内存溢出

**Data transfer strategy**:
- 表格数据：分页加载（默认 1000 行/页）
- 图表数据：后端预聚合后传输
- 导出数据：后端直接写文件，前端只传参数

### D5: 前端技术栈
**Decision**: Vue 3 + Vite + Naive UI + ECharts

**Rationale**:
- Vue 3 Composition API 适合复杂状态管理
- Vite 开发体验好，HMR 快
- Naive UI 组件丰富，TypeScript 支持好
- ECharts 图表能力全面，工业图表支持好

### D6: 配置文件格式选择 TOML
**Decision**: 使用 TOML 格式配置数据库连接

**Rationale**:
- 人类可读性好，适合手动编辑
- Rust 生态支持好（serde + toml crate）
- 结构清晰，适合配置场景

**Config structure**:
```toml
[database]
server = "localhost"
port = 1433
database = "控制器数据库"
username = "sa"
password = "password"

[query]
default_table = "历史表"
```

### D7: 默认查询策略
**Decision**: 默认查询【控制器数据库】的【历史表】

**Rationale**:
- 减少用户配置步骤
- 符合工业现场典型使用场景
- 可通过配置文件覆盖默认值

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| tiberius (SQL Server) 稳定性 | 连接失败导致功能不可用 | 早期 PoC 验证；准备 ODBC 备选方案 |
| Tauri WebView2 兼容性 | Windows 旧系统无法运行 | 打包时嵌入 WebView2 运行时 |
| 大数据前端渲染性能 | UI 卡顿 | 虚拟滚动表格；Canvas 图表 |
| Rust 开发效率 | 开发周期长 | 核心逻辑单元测试；前端 mock 并行开发 |

## Migration Plan
N/A（新项目，无迁移需求）

## Resolved Questions
1. ~~是否需要支持 Windows 认证连接 SQL Server？~~ → **否，仅支持 SQL 认证**
2. ~~图表需要支持哪些具体类型？~~ → **仅折线图，支持时间范围选择**
3. ~~数据导出是否需要支持 Excel 格式？~~ → **否，仅 CSV**

## Open Questions
1. 配置文件默认路径？（建议：应用目录下 `config.toml`）
2. 时间范围选择器的预设选项？（如：最近1小时、最近24小时、自定义）
