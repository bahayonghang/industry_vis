# 数据模型

## HistoryRecord

历史数据记录。

```typescript
interface HistoryRecord {
  dateTime: string    // ISO 8601 时间戳
  tagName: string     // 标签名称
  tagVal: number      // 数值
  tagQuality: string  // 质量标志
}
```

## DatabaseConfig

数据库连接配置。

```typescript
interface DatabaseConfig {
  server: string      // 服务器地址
  port: number        // 端口号
  database: string    // 数据库名
  username: string    // 用户名
  password: string    // 密码
}
```

## QueryConfig

查询配置。

```typescript
interface QueryConfig {
  defaultTable: string  // 默认表名
}
```

## AppConfig

完整应用配置。

```typescript
interface AppConfig {
  database: DatabaseConfig
  query: QueryConfig
}
```

## QueryParams

查询参数。

```typescript
interface QueryParams {
  startTime: string       // 开始时间
  endTime: string         // 结束时间
  tags?: string[]         // 标签列表（可选）
  limit?: number          // 限制数量（可选）
  offset?: number         // 偏移量（可选）
}
```

## QueryResult

查询结果。

```typescript
interface QueryResult {
  records: HistoryRecord[]  // 记录数组
  total: number             // 总数
}
```

## ConnectionTestResult

连接测试结果。

```typescript
interface ConnectionTestResult {
  success: boolean    // 是否成功
  message: string     // 消息
}
```
