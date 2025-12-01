# Tauri Commands

## load_config

Load application configuration.

```typescript
const config = await invoke<AppConfig>('load_config')
```

**Returns**: `AppConfig`

## save_config

Save application configuration.

```typescript
await invoke('save_config', { config: appConfig })
```

**Parameters**:
- `config`: `AppConfig` - Configuration object to save

## test_connection

Test database connection.

```typescript
const result = await invoke<ConnectionTestResult>('test_connection', { 
  config: databaseConfig 
})
```

**Parameters**:
- `config`: `DatabaseConfig` - Database configuration

**Returns**: `ConnectionTestResult`
- `success`: `boolean` - Whether connection succeeded
- `message`: `string` - Result message

## get_available_tags

Get all available tags from the database.

```typescript
const tags = await invoke<string[]>('get_available_tags')
```

**Returns**: `string[]` - Array of tag names

## query_history

Query history data.

```typescript
const result = await invoke<QueryResult>('query_history', { params })
```

**Parameters**: `QueryParams`
- `startTime`: `string` - ISO 8601 format start time
- `endTime`: `string` - ISO 8601 format end time
- `tags`: `string[]` (optional) - Tags to query
- `limit`: `number` (optional) - Record count limit
- `offset`: `number` (optional) - Pagination offset

**Returns**: `QueryResult`
- `records`: `HistoryRecord[]` - Record array
- `total`: `number` - Total record count

## export_to_csv

Export data to CSV file.

```typescript
await invoke('export_to_csv', { 
  records: historyRecords,
  filePath: '/path/to/file.csv'
})
```

**Parameters**:
- `records`: `HistoryRecord[]` - Records to export
- `filePath`: `string` - Target file path
