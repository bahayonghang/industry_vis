// 历史表记录类型
export interface HistoryRecord {
  dateTime: string
  tagName: string
  tagVal: number
  tagQuality: string
}

// 数据库配置
export interface DatabaseConfig {
  server: string
  port: number
  database: string
  username: string
  password: string
}

// 查询配置
export interface QueryConfig {
  defaultTable: string
}

// 完整应用配置
export interface AppConfig {
  database: DatabaseConfig
  query: QueryConfig
}

// 查询参数
export interface QueryParams {
  startTime: string
  endTime: string
  tags?: string[]
  limit?: number
  offset?: number
}

// 查询结果
export interface QueryResult {
  records: HistoryRecord[]
  total: number
}

// 连接测试结果
export interface ConnectionTestResult {
  success: boolean
  message: string
}

// 标签分组
export interface TagGroup {
  id: string
  name: string
  tags: string[]  // 最多 20 个
  createdAt: string
  updatedAt: string
}
