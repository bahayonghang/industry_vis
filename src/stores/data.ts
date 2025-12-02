import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DataProcessingConfig, HistoryRecord, QueryParams, QueryResult } from '@/types'

/**
 * 将 Date 对象格式化为本地时间字符串（用于数据库查询）
 * 格式：YYYY-MM-DDTHH:mm:ss.SSS（不带时区后缀）
 */
function formatLocalDateTime(date: Date): string {
  const pad = (n: number, len = 2) => n.toString().padStart(len, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}.${pad(date.getMilliseconds(), 3)}`
}

export const useDataStore = defineStore('data', () => {
  // State
  const records = ref<HistoryRecord[]>([])
  const availableTags = ref<string[]>([])
  const selectedTags = ref<string[]>([])
  const startTime = ref<Date>(new Date(Date.now() - 24 * 60 * 60 * 1000))
  const endTime = ref<Date>(new Date())
  const loading = ref(false)
  const error = ref<string | null>(null)
  const total = ref(0)

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

  const fetchData = async (processingConfig?: DataProcessingConfig) => {
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
    availableTags,
    selectedTags,
    startTime,
    endTime,
    loading,
    error,
    total,
    // Actions
    setTimeRange,
    setSelectedTags,
    fetchAvailableTags,
    fetchData,
    exportToCsv,
  }
})
