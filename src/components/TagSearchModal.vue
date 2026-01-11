<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { 
  NModal, 
  NInput, 
  NButton, 
  NIcon, 
  NEmpty, 
  NSpin,
  NScrollbar,
  NList,
  NListItem
} from 'naive-ui'
import { SearchOutline, AddOutline } from '@vicons/ionicons5'
import { useTagGroupStore } from '@/stores/tagGroup'
import { useDebounceFn } from '@vueuse/core'

const props = defineProps<{
  show: boolean
  selectedTags: string[]
  maxTags?: number
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  'add': [tag: string]
}>()

const tagGroupStore = useTagGroupStore()
const searchKeyword = ref('')

// 过滤已选标签
const filteredResults = computed(() => 
  tagGroupStore.searchResults.filter(tag => !props.selectedTags.includes(tag))
)

// 是否达到上限
const isMaxReached = computed(() => 
  props.maxTags ? props.selectedTags.length >= props.maxTags : false
)

// 防抖搜索
const debouncedSearch = useDebounceFn((keyword: string) => {
  tagGroupStore.searchTags(keyword, 100)
}, 300)

watch(searchKeyword, (keyword) => {
  if (keyword.trim()) {
    debouncedSearch(keyword)
  } else {
    tagGroupStore.clearSearchResults()
  }
})

// 关闭时清空搜索
watch(() => props.show, (show) => {
  if (!show) {
    searchKeyword.value = ''
    tagGroupStore.clearSearchResults()
  }
})

function handleAdd(tag: string) {
  if (isMaxReached.value) return
  emit('add', tag)
}

function handleClose() {
  emit('update:show', false)
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    title="添加标签"
    :style="{ width: '500px', maxHeight: '70vh' }"
    :mask-closable="true"
    :close-on-esc="true"
    @update:show="handleClose"
  >
    <div class="tag-search-modal">
      <!-- 搜索框 -->
      <NInput
        v-model:value="searchKeyword"
        placeholder="输入标签名模糊搜索（从 TagDatabase 表查询）..."
        clearable
        size="large"
        :disabled="isMaxReached"
      >
        <template #prefix>
          <NIcon :component="SearchOutline" />
        </template>
        <template #suffix>
          <NSpin v-if="tagGroupStore.searchLoading" :size="16" />
        </template>
      </NInput>
      
      <!-- 提示信息 -->
      <div v-if="isMaxReached" class="max-warning">
        已达到标签上限（{{ maxTags }}个）
      </div>
      
      <!-- 搜索结果 -->
      <div class="results-container">
        <NScrollbar style="max-height: 400px">
          <div v-if="!searchKeyword.trim()" class="empty-hint">
            请输入关键词搜索标签
          </div>
          
          <div v-else-if="tagGroupStore.searchLoading" class="loading-state">
            <NSpin size="medium" />
            <span>搜索中...</span>
          </div>
          
          <NEmpty 
            v-else-if="filteredResults.length === 0" 
            description="无匹配结果"
          />
          
          <NList v-else hoverable clickable>
            <NListItem
              v-for="tag in filteredResults"
              :key="tag"
              @click="handleAdd(tag)"
            >
              <div class="result-item">
                <span class="tag-name">{{ tag }}</span>
                <NButton 
                  size="tiny" 
                  type="primary" 
                  secondary
                  :disabled="isMaxReached"
                >
                  <template #icon>
                    <NIcon :component="AddOutline" />
                  </template>
                  添加
                </NButton>
              </div>
            </NListItem>
          </NList>
        </NScrollbar>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
/* ===== 赛博朋克标签搜索弹窗 ===== */
.tag-search-modal {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.max-warning {
  padding: 10px 14px;
  background: rgba(255, 136, 0, 0.1);
  border: 1px solid rgba(255, 136, 0, 0.3);
  border-radius: var(--radius-lg);
  color: var(--neon-orange);
  font-family: var(--font-body);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
  box-shadow: 0 0 10px rgba(255, 136, 0, 0.1);
}

.results-container {
  position: relative;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  min-height: 200px;
  background: var(--glass-bg);
  overflow: hidden;
}

/* 结果容器顶部霓虹条 */
.results-container::before {
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
  opacity: 0.5;
}

.empty-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-muted);
  font-family: var(--font-body);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  height: 200px;
  color: var(--neon-cyan);
  font-family: var(--font-body);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
}

.result-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 4px 0;
}

.tag-name {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--neon-cyan);
  letter-spacing: var(--tracking-wide);
}

/* 列表项悬停效果 */
.results-container :deep(.n-list-item) {
  transition: all var(--transition-fast);
  border-radius: var(--radius-md);
  margin: 2px 4px;
}

.results-container :deep(.n-list-item:hover) {
  background: var(--bg-hover);
  box-shadow: 0 0 10px var(--neon-cyan-glow);
}
</style>
