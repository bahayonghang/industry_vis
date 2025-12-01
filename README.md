# 工业数据查看系统 (Industry Vis)

基于 Tauri 的工业数据查看系统，用于读取、处理和展示工业运行数据。

## 功能特性

- 🔌 SQL Server 数据库连接（SQL 认证）
- 📊 时序数据折线图展示
- 📋 数据表格展示（虚拟滚动）
- 🕐 时间范围选择（预设 + 自定义）
- 🏷️ 标签多选过滤
- 💾 CSV 数据导出
- ⚙️ TOML 配置文件

## 技术栈

- **桌面框架**: Tauri 2.x
- **后端**: Rust + Polars
- **前端**: Vue 3 + Vite + Naive UI + ECharts
- **数据库**: SQL Server (tiberius)

## 快速开始

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- Windows 10/11 (WebView2)

### 安装依赖

```bash
# 安装前端依赖
npm install

# Rust 依赖会在首次构建时自动安装
```

### 配置数据库

1. 复制 `config.example.toml` 为 `config.toml`
2. 修改数据库连接信息：

```toml
[database]
server = "localhost"
port = 1433
database = "控制器数据库"
username = "sa"
password = "your_password"

[query]
default_table = "历史表"
```

### 开发模式

```bash
npm run tauri:dev
```

### 生产构建

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 数据库表结构

系统默认查询 `历史表`，表结构如下：

| 列名 | 类型 | 说明 |
|------|------|------|
| DateTime | datetime | 时间戳 |
| TagName | nchar(50) | 标签名称 |
| TagVal | real | 数值 |
| TagQuality | nchar(10) | 质量标志 |

## 项目结构

```
industry_vis/
├── src/                    # Vue 前端源码
│   ├── components/         # 组件
│   ├── views/              # 页面
│   ├── stores/             # Pinia 状态管理
│   └── types/              # TypeScript 类型
├── src-tauri/              # Rust 后端源码
│   ├── src/
│   │   ├── commands.rs     # Tauri 命令
│   │   ├── config.rs       # 配置管理
│   │   ├── datasource/     # 数据源抽象
│   │   └── models.rs       # 数据模型
│   └── Cargo.toml
├── config.example.toml     # 配置文件示例
└── package.json
```

## 许可证

MIT