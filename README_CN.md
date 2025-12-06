# 工业数据可视化系统 (Industry Vis)

基于 Tauri 的工业数据可视化系统，用于读取、处理和展示工业运行数据。

[English](./README.md) | 简体中文

## 功能特性

### 数据连接
- 🔌 SQL Server 数据库连接（SQL 认证）
- 🔍 标签模糊搜索（支持中文）

### 数据展示
- 📊 时序数据折线图（ECharts）
- 🕐 时间范围选择（预设 + 自定义）
- 🏷️ 标签分组管理（最多20个标签/组）

### 数据处理
- 🧹 异常值剔除（3σ法则）
- 📉 时间序列重采样（均值聚合）
- 📈 平滑滤波（移动平均）
- ⚡ 数据降采样（自动优化渲染）

### 其他
- 💾 CSV 数据导出
- ⚙️ 配置持久化（便携模式/安装模式）
- 🌓 明暗主题切换

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x |
| 后端 | Rust + Polars |
| 前端 | Vue 3 + Vite + TypeScript |
| UI 组件 | Naive UI |
| 图表 | ECharts |
| 数据库 | SQL Server (tiberius) |

## 快速开始

### 环境要求

- [Bun](https://bun.sh) >= 1.0 (或 Node.js >= 18)
- Rust >= 1.70
- Windows 10/11
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (必需，若启动报错请先安装)
- [Just](https://github.com/casey/just) (可选，命令简化)

### 安装依赖

```bash
bun install
```

### 开发模式

```bash
# 使用 just
just dev

# 或直接使用 bun
bun run tauri:dev
```

### 构建

```bash
# 快速构建便携版（日常开发）
just build
# 输出: src-tauri/target/release/industry-vis.exe

# 构建安装包（正式发布）
just release
# 输出: src-tauri/target/release/bundle/nsis/Industry Vis_x.x.x_x64-setup.exe
```

## 配置

应用首次启动后，在**系统设置**页面配置数据库连接：

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| 服务器 | SQL Server 地址 | localhost |
| 端口 | 数据库端口 | 1433 |
| 数据库 | 数据库名称 | 控制器数据库 |
| 用户名 | SQL 认证用户名 | sa |
| 默认表 | 历史数据表名 | 历史表 |

配置保存位置：
- **便携模式**: exe 同目录 `config.toml`
- **安装模式**: `%APPDATA%\IndustryVis\config.toml`

## 数据库表结构

### 历史表

| 列名 | 类型 | 说明 |
|------|------|------|
| DateTime | datetime | 时间戳 |
| TagName | nvarchar(50) | 标签名称 |
| TagVal | real | 数值 |
| TagQuality | nchar(10) | 质量标志 |

### TagDataBase（标签搜索）

| 列名 | 类型 | 说明 |
|------|------|------|
| TagName | nvarchar(50) | 标签名称 |

## 自定义 Schema Profile

系统通过 **Schema Profile** 支持不同厂商的数据库结构。这允许您适配不同的表结构和字段名称，同时保持数据处理和可视化逻辑一致。

### 配置方式

在 `config.toml` 中添加 `schema` 配置节：

```toml
[schema]
profile = "default"  # Profile 名称，默认为 "default"
```

### 可用的 Profile

| Profile | 说明 |
|---------|------|
| `default` | 默认 Profile，适配当前厂商（TagDataBase + 历史表） |

### 添加新的 Profile

如需支持新厂商的数据库结构，按以下步骤操作：

1. **创建 Profile 文件**：`src-tauri/src/datasource/profiles/vendor_x.rs`

```rust
use crate::datasource::SchemaProfile;
use crate::error::AppResult;
use crate::models::HistoryRecord;

pub struct VendorXProfile;

impl SchemaProfile for VendorXProfile {
    fn name(&self) -> &str { "vendor_x" }
    
    fn tag_search_sql(&self, limit: usize) -> String {
        // 返回标签搜索 SQL
        format!("SELECT TOP {} TagName FROM YourTagTable WHERE TagName LIKE @P1", limit)
    }
    
    fn history_query_sql(&self, table: &str, start: &str, end: &str, filter: &str) -> String {
        // 返回历史查询 SQL
        format!("SELECT time_col, tag_col, value_col, quality_col FROM [{}] WHERE ...", table)
    }
    
    fn map_history_row(&self, row: &tiberius::Row) -> AppResult<HistoryRecord> {
        // 将数据库行映射为 HistoryRecord
        Ok(HistoryRecord::new(/* ... */))
    }
}
```

2. **在 ProfileRegistry 中注册**：`src-tauri/src/datasource/profiles/registry.rs`

```rust
pub fn get(name: &str) -> AppResult<Arc<dyn SchemaProfile>> {
    match name {
        "default" => Ok(Arc::new(DefaultProfile::new())),
        "vendor_x" => Ok(Arc::new(VendorXProfile::new())),  // 添加此行
        _ => Err(AppError::Config(format!("未知的 Profile: {}", name))),
    }
}
```

3. **更新配置**并重启应用。

详细文档请参阅 [Schema Profile 指南](./docs/guide/schema-profile.md)。

## 项目结构

```
industry_vis/
├── src/                      # Vue 前端源码
│   ├── components/           # 组件
│   │   ├── GroupEditView.vue # 分组编辑（含数据处理）
│   │   ├── LineChart.vue     # 折线图
│   │   └── TagSearchModal.vue# 标签搜索弹窗
│   ├── views/                # 页面
│   ├── stores/               # Pinia 状态管理
│   └── types/                # TypeScript 类型
├── src-tauri/                # Rust 后端源码
│   ├── src/
│   │   ├── commands.rs       # Tauri 命令
│   │   ├── config.rs         # 配置管理
│   │   ├── data_processing.rs# 数据处理模块
│   │   ├── datasource/       # 数据源抽象
│   │   ├── models.rs         # 数据模型
│   │   └── tag_group.rs      # 标签分组管理
│   └── Cargo.toml
├── docs/                     # VitePress 文档
├── justfile                  # Just 命令配置
└── package.json
```

## 文档

完整文档请访问 [docs/](./docs/) 目录或运行：

```bash
just docs
```

## 许可证

MIT