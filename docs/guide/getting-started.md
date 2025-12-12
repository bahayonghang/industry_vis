# 快速开始

本指南将帮助你在本地运行 Industry Vis 项目。

## 环境要求

| 依赖 | 版本要求 | 说明 |
|------|----------|------|
| **Node.js** | >= 18 | 前端构建工具 |
| **Rust** | >= 1.70 | 后端编译 |
| **Windows** | 10/11 | 目前仅支持 Windows |
| **WebView2** | 最新版 | [下载地址](https://developer.microsoft.com/microsoft-edge/webview2/) |

::: warning WebView2 必需
如果启动时报错，请先安装 WebView2 Runtime。Windows 11 已内置，Windows 10 可能需要手动安装。
:::

## 第一步：克隆项目

```bash
git clone https://github.com/your-repo/industry-vis.git
cd industry-vis
```

## 第二步：安装依赖

```bash
# 安装前端依赖（推荐使用 bun）
bun install

# 或使用 npm
npm install
```

::: info Rust 依赖
Rust 依赖会在首次构建时自动安装，无需手动操作。
:::

## 第三步：配置数据库

1. 复制配置文件模板：

```bash
cp config.example.toml config.toml
```

2. 编辑 `config.toml`，填写数据库连接信息：

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

::: danger 安全提示
请勿将包含真实密码的 `config.toml` 提交到版本控制系统。
:::

## 第四步：启动开发模式

```bash
bun run tauri:dev
```

这将同时启动 Vite 开发服务器和 Tauri 应用。

::: tip 热重载
开发模式下，前端代码修改会自动热重载，Rust 代码修改需要重新编译。
:::

## 生产构建

```bash
bun run tauri:build
```

- **构建产物位置**：`src-tauri/target/release/bundle/nsis/`

## 使用 Just 命令（可选）

如果你安装了 [just](https://github.com/casey/just)，可以使用以下快捷命令：

| 命令 | 说明 |
|------|------|
| `just dev` | 启动开发模式 |
| `just build` | 生产构建 |
| `just test` | 运行测试 |
| `just docs` | 启动文档服务 |

## 下一步

- [配置](/guide/configuration) — 详细配置说明
- [数据查询](/guide/data-query) — 使用数据查询功能
