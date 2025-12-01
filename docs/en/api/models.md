# Data Models

## HistoryRecord

History data record.

```typescript
interface HistoryRecord {
  dateTime: string    // ISO 8601 timestamp
  tagName: string     // Tag name
  tagVal: number      // Value
  tagQuality: string  // Quality flag
}
```

## DatabaseConfig

Database connection configuration.

```typescript
interface DatabaseConfig {
  server: string      // Server address
  port: number        // Port number
  database: string    // Database name
  username: string    // Username
  password: string    // Password
}
```

## QueryConfig

Query configuration.

```typescript
interface QueryConfig {
  defaultTable: string  // Default table name
}
```

## AppConfig

Complete application configuration.

```typescript
interface AppConfig {
  database: DatabaseConfig
  query: QueryConfig
}
```

## QueryParams

Query parameters.

```typescript
interface QueryParams {
  startTime: string       // Start time
  endTime: string         // End time
  tags?: string[]         // Tag list (optional)
  limit?: number          // Limit count (optional)
  offset?: number         // Offset (optional)
}
```

## QueryResult

Query result.

```typescript
interface QueryResult {
  records: HistoryRecord[]  // Record array
  total: number             // Total count
}
```

## ConnectionTestResult

Connection test result.

```typescript
interface ConnectionTestResult {
  success: boolean    // Success status
  message: string     // Message
}
```
