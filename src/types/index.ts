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

// 查询结果 (V1 兼容格式)
export interface QueryResult {
  records: HistoryRecord[]
  total: number
}

// 图表系列数据 (V2 格式，按标签预分组)
export interface ChartSeriesData {
  tagName: string
  data: [number, number][]  // [[timestamp_ms, value], ...]
}

// 查询结果 V2 (预分组格式，优化前端渲染)
export interface QueryResultV2 {
  series: ChartSeriesData[]
  totalRaw: number
  totalProcessed: number
  cacheHit: boolean
  queryTimeMs: number
}

// 缓存统计信息
export interface CacheStats {
  hits: number
  misses: number
  hitRate: number
  entries: number
  maxEntries: number
  estimatedMemoryBytes: number
}

// 连接测试结果
export interface ConnectionTestResult {
  success: boolean
  message: string
}

// 连接池状态
export interface PoolState {
  connections: number
  idleConnections: number
  activeConnections: number
  maxSize: number
}

// 异常值剔除配置
export interface OutlierRemovalConfig {
  enabled: boolean
  method: string  // "3sigma"
}

// 重采样配置
export interface ResampleConfig {
  enabled: boolean
  interval: number  // 秒
  method: string    // "mean"
}

// 平滑滤波配置
export interface SmoothingConfig {
  enabled: boolean
  method: string   // "moving_avg"
  window: number   // 窗口大小
}

// 数据处理配置
export interface DataProcessingConfig {
  outlierRemoval: OutlierRemovalConfig
  resample: ResampleConfig
  smoothing: SmoothingConfig
}

// 创建默认数据处理配置
export function createDefaultProcessingConfig(): DataProcessingConfig {
  return {
    outlierRemoval: {
      enabled: false,
      method: '3sigma'
    },
    resample: {
      enabled: false,
      interval: 60,
      method: 'mean'
    },
    smoothing: {
      enabled: false,
      method: 'moving_avg',
      window: 5
    }
  }
}

// 图表配置（分组内的单个图表）
export interface ChartConfig {
  id: string
  name: string
  tags: string[]  // 最多 5 个标签
}

// 标签分组（包含多个图表）
export interface TagGroup {
  id: string
  name: string
  charts: ChartConfig[]  // 最多 10 个图表
  processingConfig: DataProcessingConfig
  createdAt: string
  updatedAt: string
}

// 创建默认图表配置
export function createDefaultChartConfig(name: string = '新图表'): ChartConfig {
  return {
    id: `c${Date.now()}`,
    name,
    tags: []
  }
}
