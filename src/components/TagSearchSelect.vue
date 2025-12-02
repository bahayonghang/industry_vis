<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { NInput, NIcon, NSpin, NEmpty, NTag, NSpace, NScrollbar } from 'naive-ui'
import { SearchOutline } from '@vicons/ionicons5'
import { useTagGroupStore } from '@/stores/tagGroup'
import { useDebounceFn } from '@vueuse/core'

const props = defineProps<{
  modelValue: string[]
  maxTags?: number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const tagGroupStore = useTagGroupStore()

const searchKeyword = ref('')
const maxTags = computed(() => props.maxTags ?? 20)

// 已选标签
const selectedTags = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

// 是否达到上限
const isMaxReached = computed(() => selectedTags.value.length >= maxTags.value)

// 防抖搜索（300ms）
const debouncedSearch = useDebounceFn((keyword: string) => {
  tagGroupStore.searchTags(keyword, 50)
}, 300)

// 监听搜索关键词变化
watch(searchKeyword, (keyword) => {
  if (keyword.trim()) {
    debouncedSearch(keyword)
  } else {
    tagGroupStore.clearSearchResults()
  }
})

// 添加标签
function addTag(tag: string) {
  if (isMaxReached.value) return
  if (selectedTags.value.includes(tag)) return
  
  selectedTags.value = [...selectedTags.value, tag]
}

// 移除标签
function removeTag(tag: string) {
  selectedTags.value = selectedTags.value.filter(t => t !== tag)
}

// 清空所有
function clearAll() {
  selectedTags.value = []
}

// 过滤已选标签后的搜索结果
const filteredResults = computed(() => 
  tagGroupStore.searchResults.filter(tag => !selectedTags.value.includes(tag))
)
</script>

<template>
  <div class="tag-search-select">
    <!-- 搜索框 -->
    <NInput
      v-model:value="searchKeyword"
      placeholder="输入标签名模糊搜索..."
      clearable
      :disabled="isMaxReached"
    >
      <template #prefix>
        <NIcon :component="SearchOutline" />
      </template>
      <template #suffix>
        <NSpin v-if="tagGroupStore.searchLoading" :size="16" />
      </template>
    </NInput>

    <!-- 搜索结果 -->
    <div v-if="searchKeyword.trim()" class="search-results">
      <div class="results-header">
        搜索结果
        <span v-if="filteredResults.length > 0" class="results-count">
          ({{ filteredResults.length }})
        </span>
      </div>
      
      <NScrollbar style="max-height: 200px">
        <div v-if="tagGroupStore.searchLoading" class="loading-state">
          <NSpin size="small" />
          <span>搜索中...</span>
        </div>
        
        <NEmpty 
          v-else-if="filteredResults.length === 0" 
          size="small"
          description="无匹配结果"
        />
        
        <div v-else class="results-list">
          <div
            v-for="tag in filteredResults"
            :key="tag"
            class="result-item"
            :class="{ disabled: isMaxReached }"
            @click="addTag(tag)"
          >
            {{ tag }}
          </div>
        </div>
      </NScrollbar>
    </div>

    <!-- 已选标签 -->
    <div class="selected-tags">
      <div class="selected-header">
        <span>已选标签</span>
        <span class="tag-count" :class="{ warning: isMaxReached }">
          {{ selectedTags.length }}/{{ maxTags }}
        </span>
        <span 
          v-if="selectedTags.length > 0" 
          class="clear-all"
          @click="clearAll"
        >
          清空
        </span>
      </div>
      
      <NScrollbar style="max-height: 150px">
        <NEmpty 
          v-if="selectedTags.length === 0" 
          size="small"
          description="点击搜索结果添加标签"
        />
        
        <NSpace v-else :size="[8, 8]" style="padding: 8px 0">
          <NTag
            v-for="tag in selectedTags"
            :key="tag"
            closable
            round
            type="info"
            @close="removeTag(tag)"
          >
            {{ tag }}
          </NTag>
        </NSpace>
      </NScrollbar>
    </div>
  </div>
</template>

<style scoped>
.tag-search-select {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.search-results {
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.results-header {
  padding: 8px 12px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.results-count {
  color: var(--text-muted);
}

.loading-state {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px;
  color: var(--text-muted);
  font-size: 13px;
}

.results-list {
  padding: 4px;
}

.result-item {
  padding: 8px 12px;
  font-size: 13px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: background-color 0.15s;
}

.result-item:hover:not(.disabled) {
  background: var(--bg-hover);
}

.result-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.selected-tags {
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 12px;
}

.selected-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
}

.tag-count {
  color: var(--text-muted);
  font-weight: normal;
}

.tag-count.warning {
  color: #f59e0b;
}

.clear-all {
  margin-left: auto;
  color: var(--accent-color);
  cursor: pointer;
  font-weight: normal;
}

.clear-all:hover {
  text-decoration: underline;
}
</style>
