# Tauri 命令

本页面列出所有可用的 Tauri IPC 命令，用于前端与 Rust 后端通信。

::: info 调用方式
所有命令通过 `@tauri-apps/api/core` 的 `invoke` 函数调用。
:::

## 配置管理

### load_config

加载应用配置。

```typescript
import { invoke } from '@tauri-apps/api/core'

const config = await invoke<AppConfig>('load_config')
```

| 项目 | 说明 |
|------|------|
| **参数** | 无 |
| **返回值** | `AppConfig` — 应用配置对象 |

---

### save_config

保存应用配置。

```typescript
await invoke('save_config', { config: appConfig })
```

| 项目 | 说明 |
|------|------|
| **参数** | `config: AppConfig` — 要保存的配置对象 |
| **返回值** | 无 |

---

## 数据库连接

### test_connection

测试数据库连接。

```typescript
const result = await invoke<ConnectionTestResult>('test_connection', { 
  config: databaseConfig 
})
```

| 项目 | 说明 |
|------|------|
| **参数** | `config: DatabaseConfig` — 数据库配置 |
| **返回值** | `ConnectionTestResult` |

**返回值结构**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `success` | `boolean` | 是否连接成功 |
| `message` | `string` | 结果消息 |

---

### get_available_tags

获取数据库中所有可用的标签。

```typescript
const tags = await invoke<string[]>('get_available_tags')
```

| 项目 | 说明 |
|------|------|
| **参数** | 无 |
| **返回值** | `string[]` — 标签名称数组 |

---

## 数据查询

### query_history

查询历史数据。

```typescript
const result = await invoke<QueryResult>('query_history', { params })
```

**参数 `QueryParams`**：

| 字段 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| `startTime` | `string` | ✅ | ISO 8601 格式的开始时间 |
| `endTime` | `string` | ✅ | ISO 8601 格式的结束时间 |
| `tags` | `string[]` | ❌ | 要查询的标签列表 |
| `limit` | `number` | ❌ | 返回记录数限制 |
| `offset` | `number` | ❌ | 分页偏移量 |

**返回值 `QueryResult`**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `records` | `HistoryRecord[]` | 记录数组 |
| `total` | `number` | 总记录数 |

---

## 数据导出

### export_to_csv

导出数据到 CSV 文件。

```typescript
await invoke('export_to_csv', { 
  records: historyRecords,
  filePath: '/path/to/file.csv'
})
```

| 项目 | 说明 |
|------|------|
| **参数** | `records: HistoryRecord[]` — 要导出的记录<br>`filePath: string` — 目标文件路径 |
| **返回值** | 无 |

::: warning 注意
导出前请确保目标路径有写入权限。
:::
