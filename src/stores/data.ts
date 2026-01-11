import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { CacheStats, ChartSeriesData, DataProcessingConfig, HistoryRecord, PoolState, QueryParams, QueryResult, QueryResultV2 } from '@/types'

/**
 * 将 Date 对象格式化为本地时间字符串（用于数据库查询）
 * 格式：YYYY-MM-DDTHH:mm:ss.SSS（不带时区后缀）
 */
function formatLocalDateTime(date: Date): string {
  const pad = (n: number, len = 2) => n.toString().padStart(len, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}.${pad(date.getMilliseconds(), 3)}`
}

export const useDataStore = defineStore('data', () => {
  // State - 使用 shallowRef 避免大数据深度响应式追踪
  const records = shallowRef<HistoryRecord[]>([])
  const chartSeries = shallowRef<ChartSeriesData[]>([])  // V2 预分组数据
  const availableTags = ref<string[]>([])
  const selectedTags = ref<string[]>([])
  const startTime = ref<Date>(new Date(Date.now() - 24 * 60 * 60 * 1000))
  const endTime = ref<Date>(new Date())
  const loading = ref(false)
  const error = ref<string | null>(null)
  const total = ref(0)
  const totalProcessed = ref(0)
  const cacheHit = ref(false)
  const queryTimeMs = ref(0)

  // Actions
  const setTimeRange = (start: Date, end: Date) => {
    startTime.value = start
    endTime.value = end
  }

  const setSelectedTags = (tags: string[]) => {
    selectedTags.value = tags
  }

  const fetchAvailableTags = async () => {
    try {
      const tags = await invoke<string[]>('get_available_tags')
      availableTags.value = tags
    } catch (e) {
      console.error('Failed to fetch tags:', e)
      error.value = String(e)
    }
  }

  /**
   * 使用 V1 接口获取数据（保持向后兼容）
   */
  const fetchData = async (processingConfig?: DataProcessingConfig, forceRefresh = false) => {
    loading.value = true
    error.value = null
    
    try {
      const params: QueryParams = {
        startTime: formatLocalDateTime(startTime.value),
        endTime: formatLocalDateTime(endTime.value),
        tags: selectedTags.value.length > 0 ? selectedTags.value : undefined,
      }
      
      const result = await invoke<QueryResult>('query_history', { 
        params,
        processingConfig: processingConfig || null,
        forceRefresh,
      })
      records.value = result.records
      total.value = result.total
    } catch (e) {
      console.error('Failed to fetch data:', e)
      error.value = String(e)
      records.value = []
    } finally {
      loading.value = false
    }
  }

  /**
   * 使用 V2 接口获取数据（预分组格式，优化渲染）
   */
  const fetchDataV2 = async (processingConfig?: DataProcessingConfig, forceRefresh = false) => {
    loading.value = true
    error.value = null
    
    try {
      const params: QueryParams = {
        startTime: formatLocalDateTime(startTime.value),
        endTime: formatLocalDateTime(endTime.value),
        tags: selectedTags.value.length > 0 ? selectedTags.value : undefined,
      }
      
      const result = await invoke<QueryResultV2>('query_history_v2', { 
        params,
        processingConfig: processingConfig || null,
        forceRefresh,
      })
      
      chartSeries.value = result.series
      total.value = result.totalRaw
      totalProcessed.value = result.totalProcessed
      cacheHit.value = result.cacheHit
      queryTimeMs.value = result.queryTimeMs
      
      // 同时更新 records 以保持兼容（如果需要）
      // records.value = seriesDataToRecords(result.series)
    } catch (e) {
      console.error('Failed to fetch data V2:', e)
      error.value = String(e)
      chartSeries.value = []
    } finally {
      loading.value = false
    }
  }

  /**
   * 清空缓存
   */
  const clearCache = async () => {
    try {
      await invoke('clear_cache')
    } catch (e) {
      console.error('Failed to clear cache:', e)
      error.value = String(e)
    }
  }

  /**
   * 获取缓存统计
   */
  const getCacheStats = async (): Promise<CacheStats | null> => {
    try {
      return await invoke<CacheStats>('get_cache_stats')
    } catch (e) {
      console.error('Failed to get cache stats:', e)
      return null
    }
  }

  /**
   * 获取连接池状态
   */
  const getPoolState = async (): Promise<PoolState | null> => {
    try {
      return await invoke<PoolState | null>('get_pool_state')
    } catch (e) {
      console.error('Failed to get pool state:', e)
      return null
    }
  }

  const exportToCsv = async () => {
    try {
      // Use Tauri dialog plugin via invoke
      const filePath = await invoke<string | null>('plugin:dialog|save', {
        defaultPath: `history_${new Date().toISOString().slice(0, 10)}.csv`,
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      })
      
      if (filePath) {
        await invoke('export_to_csv', {
          records: records.value,
          filePath,
        })
      }
    } catch (e) {
      console.error('Failed to export:', e)
      error.value = String(e)
    }
  }

  return {
    // State
    records,
    chartSeries,
    availableTags,
    selectedTags,
    startTime,
    endTime,
    loading,
    error,
    total,
    totalProcessed,
    cacheHit,
    queryTimeMs,
    // Actions
    setTimeRange,
    setSelectedTags,
    fetchAvailableTags,
    fetchData,
    fetchDataV2,
    clearCache,
    getCacheStats,
    getPoolState,
    exportToCsv,
  }
})
