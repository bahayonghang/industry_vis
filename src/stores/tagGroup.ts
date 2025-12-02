import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { TagGroup } from '@/types'

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
  async function createGroup(name: string, tags: string[]): Promise<TagGroup | null> {
    loading.value = true
    error.value = null
    
    try {
      const newGroup = await invoke<TagGroup>('create_tag_group', { name, tags })
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
  async function updateGroup(id: string, name: string, tags: string[]): Promise<TagGroup | null> {
    loading.value = true
    error.value = null
    
    try {
      const updated = await invoke<TagGroup>('update_tag_group', { id, name, tags })
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
    // 方法
    loadGroups,
    searchTags,
    clearSearchResults,
    createGroup,
    updateGroup,
    deleteGroup,
    selectGroup,
    getGroup,
  }
})
