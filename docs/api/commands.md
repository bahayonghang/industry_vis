# Tauri 命令

## load_config

加载应用配置。

```typescript
const config = await invoke<AppConfig>('load_config')
```

**返回值**: `AppConfig`

## save_config

保存应用配置。

```typescript
await invoke('save_config', { config: appConfig })
```

**参数**:
- `config`: `AppConfig` - 要保存的配置对象

## test_connection

测试数据库连接。

```typescript
const result = await invoke<ConnectionTestResult>('test_connection', { 
  config: databaseConfig 
})
```

**参数**:
- `config`: `DatabaseConfig` - 数据库配置

**返回值**: `ConnectionTestResult`
- `success`: `boolean` - 是否连接成功
- `message`: `string` - 结果消息

## get_available_tags

获取数据库中所有可用的标签。

```typescript
const tags = await invoke<string[]>('get_available_tags')
```

**返回值**: `string[]` - 标签名称数组

## query_history

查询历史数据。

```typescript
const result = await invoke<QueryResult>('query_history', { params })
```

**参数**: `QueryParams`
- `startTime`: `string` - ISO 8601 格式的开始时间
- `endTime`: `string` - ISO 8601 格式的结束时间
- `tags`: `string[]` (可选) - 要查询的标签列表
- `limit`: `number` (可选) - 返回记录数限制
- `offset`: `number` (可选) - 分页偏移量

**返回值**: `QueryResult`
- `records`: `HistoryRecord[]` - 记录数组
- `total`: `number` - 总记录数

## export_to_csv

导出数据到 CSV 文件。

```typescript
await invoke('export_to_csv', { 
  records: historyRecords,
  filePath: '/path/to/file.csv'
})
```

**参数**:
- `records`: `HistoryRecord[]` - 要导出的记录
- `filePath`: `string` - 目标文件路径
