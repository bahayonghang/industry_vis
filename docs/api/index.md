# API 概览

本节介绍 Industry Vis 的 API 接口。

## Tauri 命令

应用通过 Tauri IPC 机制进行前后端通信。详见 [Tauri 命令](/api/commands)。

## 数据模型

系统使用的数据结构定义。详见 [数据模型](/api/models)。

## 命令列表

| 命令 | 说明 |
|------|------|
| `load_config` | 加载配置文件 |
| `save_config` | 保存配置文件 |
| `test_connection` | 测试数据库连接 |
| `get_available_tags` | 获取可用标签列表 |
| `query_history` | 查询历史数据 |
| `export_to_csv` | 导出数据到 CSV |
