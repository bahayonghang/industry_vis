# 快速开始

## 环境要求

- **Node.js** >= 18
- **Rust** >= 1.70
- **Windows 10/11**
- [**WebView2 Runtime**](https://developer.microsoft.com/microsoft-edge/webview2/) (必需，若启动报错请先安装)

## 安装依赖

```bash
# 克隆项目
git clone https://github.com/your-repo/industry-vis.git
cd industry-vis

# 安装前端依赖
npm install

# Rust 依赖会在首次构建时自动安装
```

## 配置数据库

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

## 开发模式

```bash
npm run tauri:dev
```

这将同时启动 Vite 开发服务器和 Tauri 应用。

## 生产构建

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 使用 Just 命令

如果你安装了 [just](https://github.com/casey/just)，可以使用以下命令：

```bash
# 开发模式
just dev

# 构建
just build

# 运行测试
just test

# 启动文档
just docs
```

## 下一步

- [配置](/guide/configuration) - 详细配置说明
- [数据查询](/guide/data-query) - 使用数据查询功能
