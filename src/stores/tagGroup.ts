import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ChartConfig, DataProcessingConfig, TagGroup } from '@/types'
import { createDefaultChartConfig } from '@/types'

export const useTagGroupStore = defineStore('tagGroup', () => {
  // 状态
  const groups = ref<TagGroup[]>([])
  const loading = ref(false)
  const searchLoading = ref(false)
  const searchResults = ref<string[]>([])
  const selectedGroupId = ref<string | null>(null)
  const error = ref<string | null>(null)

  // 计算属性
  const selectedGroup = computed(() => 
    groups.value.find(g => g.id === selectedGroupId.value) || null
  )

  const groupCount = computed(() => groups.value.length)

  // 加载所有分组
  async function loadGroups() {
    loading.value = true
    error.value = null
    
    try {
      groups.value = await invoke<TagGroup[]>('list_tag_groups')
    } catch (e) {
      error.value = String(e)
      console.error('加载分组失败:', e)
    } finally {
      loading.value = false
    }
  }

  // 模糊搜索标签（防抖在组件中处理）
  async function searchTags(keyword: string, limit: number = 50) {
    if (!keyword.trim()) {
      searchResults.value = []
      return
    }
    
    searchLoading.value = true
    
    try {
      searchResults.value = await invoke<string[]>('search_tags', { 
        keyword: keyword.trim(),
        limit 
      })
    } catch (e) {
      console.error('搜索标签失败:', e)
      searchResults.value = []
    } finally {
      searchLoading.value = false
    }
  }

  // 清空搜索结果
  function clearSearchResults() {
    searchResults.value = []
  }

  // 创建分组
  async function createGroup(name: string, charts: ChartConfig[] = []): Promise<TagGroup | null> {
    loading.value = true
    error.value = null
    
    try {
      // 如果没有图表，创建一个默认图表
      const chartsToCreate = charts.length > 0 ? charts : [createDefaultChartConfig('默认图表')]
      const newGroup = await invoke<TagGroup>('create_tag_group', { name, charts: chartsToCreate })
      groups.value.push(newGroup)
      return newGroup
    } catch (e) {
      error.value = String(e)
      console.error('创建分组失败:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  // 更新分组
  async function updateGroup(
    id: string, 
    name: string, 
    charts: ChartConfig[],
    processingConfig?: DataProcessingConfig
  ): Promise<TagGroup | null> {
    loading.value = true
    error.value = null
    
    try {
      const updated = await invoke<TagGroup>('update_tag_group', { 
        id, 
        name, 
        charts,
        processingConfig: processingConfig || null,
      })
      const idx = groups.value.findIndex(g => g.id === id)
      if (idx !== -1) {
        groups.value[idx] = updated
      }
      return updated
    } catch (e) {
      error.value = String(e)
      console.error('更新分组失败:', e)
      return null
    } finally {
      loading.value = false
    }
  }

  // === 图表管理辅助方法 ===
  
  // 添加图表到分组
  function addChartToGroup(groupId: string, chartName: string = '新图表'): ChartConfig | null {
    const group = groups.value.find(g => g.id === groupId)
    if (!group) return null
    if (group.charts.length >= 10) return null
    
    const newChart = createDefaultChartConfig(chartName)
    group.charts.push(newChart)
    return newChart
  }
  
  // 从分组移除图表
  function removeChartFromGroup(groupId: string, chartId: string): boolean {
    const group = groups.value.find(g => g.id === groupId)
    if (!group) return false
    
    const idx = group.charts.findIndex(c => c.id === chartId)
    if (idx === -1) return false
    
    group.charts.splice(idx, 1)
    return true
  }
  
  // 更新图表
  function updateChart(groupId: string, chartId: string, updates: Partial<ChartConfig>): boolean {
    const group = groups.value.find(g => g.id === groupId)
    if (!group) return false
    
    const chart = group.charts.find(c => c.id === chartId)
    if (!chart) return false
    
    if (updates.name !== undefined) chart.name = updates.name
    if (updates.tags !== undefined) chart.tags = updates.tags
    return true
  }
  
  // 添加标签到图表
  function addTagToChart(groupId: string, chartId: string, tagName: string): boolean {
    const group = groups.value.find(g => g.id === groupId)
    if (!group) return false
    
    const chart = group.charts.find(c => c.id === chartId)
    if (!chart) return false
    if (chart.tags.length >= 5) return false
    if (chart.tags.includes(tagName)) return false
    
    chart.tags.push(tagName)
    return true
  }
  
  // 从图表移除标签
  function removeTagFromChart(groupId: string, chartId: string, tagName: string): boolean {
    const group = groups.value.find(g => g.id === groupId)
    if (!group) return false
    
    const chart = group.charts.find(c => c.id === chartId)
    if (!chart) return false
    
    const idx = chart.tags.indexOf(tagName)
    if (idx === -1) return false
    
    chart.tags.splice(idx, 1)
    return true
  }
  
  // 获取分组中所有标签（去重）
  function getAllTagsInGroup(groupId: string): string[] {
    const group = groups.value.find(g => g.id === groupId)
    if (!group) return []
    
    const allTags = group.charts.flatMap(c => c.tags)
    return [...new Set(allTags)].sort()
  }

  // 删除分组
  async function deleteGroup(id: string): Promise<boolean> {
    loading.value = true
    error.value = null
    
    try {
      await invoke('delete_tag_group', { id })
      groups.value = groups.value.filter(g => g.id !== id)
      
      // 如果删除的是当前选中的分组，清除选中状态
      if (selectedGroupId.value === id) {
        selectedGroupId.value = null
      }
      return true
    } catch (e) {
      error.value = String(e)
      console.error('删除分组失败:', e)
      return false
    } finally {
      loading.value = false
    }
  }

  // 选择分组
  function selectGroup(id: string | null) {
    selectedGroupId.value = id
  }

  // 获取分组
  function getGroup(id: string): TagGroup | undefined {
    return groups.value.find(g => g.id === id)
  }

  return {
    // 状态
    groups,
    loading,
    searchLoading,
    searchResults,
    selectedGroupId,
    error,
    // 计算属性
    selectedGroup,
    groupCount,
    // 分组 CRUD
    loadGroups,
    createGroup,
    updateGroup,
    deleteGroup,
    selectGroup,
    getGroup,
    // 搜索
    searchTags,
    clearSearchResults,
    // 图表管理
    addChartToGroup,
    removeChartFromGroup,
    updateChart,
    // 标签管理
    addTagToChart,
    removeTagFromChart,
    getAllTagsInGroup,
  }
})
