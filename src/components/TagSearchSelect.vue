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
            v-for="(tag, index) in filteredResults"
            :key="tag"
            class="result-item"
            :class="{ disabled: isMaxReached }"
            :style="{ animationDelay: `${index * 0.03}s` }"
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
/* ===== 赛博朋克标签搜索选择器 ===== */
.tag-search-select {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* 输入框霓虹聚焦效果 */
.tag-search-select :deep(.n-input:focus-within) {
  border-color: var(--neon-cyan);
  box-shadow: 0 0 12px var(--neon-cyan-glow);
}

.search-results {
  position: relative;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--glass-bg);
}

/* 搜索结果顶部霓虹条 */
.search-results::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent 0%,
    var(--neon-cyan) 30%,
    var(--neon-magenta) 70%,
    transparent 100%
  );
  opacity: 0.6;
}

.results-header {
  padding: 10px 14px;
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wide);
  color: var(--text-secondary);
  background: linear-gradient(
    90deg,
    rgba(0, 245, 255, 0.05) 0%,
    transparent 100%
  );
  border-bottom: 1px solid var(--border-default);
}

.results-count {
  font-family: var(--font-mono);
  color: var(--neon-cyan);
}

.loading-state {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 18px;
  color: var(--neon-cyan);
  font-family: var(--font-body);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
}

.results-list {
  padding: 6px;
}

.result-item {
  padding: 10px 14px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
  color: var(--neon-cyan);
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
  animation: result-item-in 0.3s cubic-bezier(0.23, 1, 0.32, 1) backwards;
}

@keyframes result-item-in {
  from {
    opacity: 0;
    transform: translateX(-10px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.result-item:hover:not(.disabled) {
  background: var(--bg-hover);
  box-shadow: 0 0 10px var(--neon-cyan-glow);
}

.result-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
  color: var(--text-muted);
}

.selected-tags {
  position: relative;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  padding: 14px;
  background: var(--glass-bg);
}

/* 已选标签顶部霓虹条 */
.selected-tags::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent 0%,
    var(--neon-magenta) 30%,
    var(--neon-cyan) 70%,
    transparent 100%
  );
  opacity: 0.5;
}

.selected-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wide);
  color: var(--text-secondary);
}

.tag-count {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
  color: var(--text-muted);
  font-weight: normal;
  padding: 2px 8px;
  background: rgba(0, 245, 255, 0.08);
  border-radius: var(--radius-sm);
}

.tag-count.warning {
  color: var(--neon-orange);
  background: rgba(255, 136, 0, 0.1);
}

.clear-all {
  margin-left: auto;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
  color: var(--neon-red);
  cursor: pointer;
  font-weight: normal;
  transition: all var(--transition-fast);
}

.clear-all:hover {
  text-shadow: 0 0 8px var(--neon-red-glow);
}

/* 标签霓虹样式 */
.selected-tags :deep(.n-tag) {
  background: rgba(0, 245, 255, 0.08);
  border: 1px solid rgba(0, 245, 255, 0.2);
  color: var(--neon-cyan);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
}

.selected-tags :deep(.n-tag:hover) {
  background: rgba(0, 245, 255, 0.15);
  border-color: var(--neon-cyan);
  box-shadow: 0 0 10px var(--neon-cyan-glow);
}
</style>
