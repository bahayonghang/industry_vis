<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { 
  NButton, 
  NIcon, 
  NInput, 
  NEmpty, 
  NTag, 
  NSpace, 
  NSpin,
  NScrollbar,
  useMessage,
  useDialog
} from 'naive-ui'
import { 
  ArrowBackOutline, 
  SaveOutline,
  SearchOutline
} from '@vicons/ionicons5'
import { useTagGroupStore } from '@/stores/tagGroup'
import { useDebounceFn } from '@vueuse/core'
import type { TagGroup } from '@/types'

const props = defineProps<{
  groupId: string
}>()

const emit = defineEmits<{
  'back': []
  'saved': [group: TagGroup]
}>()

const tagGroupStore = useTagGroupStore()
const message = useMessage()
const dialog = useDialog()

// 编辑状态
const groupName = ref('')
const selectedTags = ref<string[]>([])
const searchKeyword = ref('')
const hasChanges = ref(false)
const saving = ref(false)

// 原始数据（用于检测变更）
const originalName = ref('')
const originalTags = ref<string[]>([])

// 是否达到标签上限
const isMaxReached = computed(() => selectedTags.value.length >= 20)

// 过滤已选标签后的搜索结果
const filteredResults = computed(() => 
  tagGroupStore.searchResults.filter(tag => !selectedTags.value.includes(tag))
)

// 检测是否有变更
watch([groupName, selectedTags], () => {
  hasChanges.value = 
    groupName.value !== originalName.value ||
    JSON.stringify(selectedTags.value.sort()) !== JSON.stringify(originalTags.value.sort())
}, { deep: true })

// 初始化加载分组数据
onMounted(() => {
  loadGroupData()
})

watch(() => props.groupId, () => {
  loadGroupData()
})

function loadGroupData() {
  const group = tagGroupStore.getGroup(props.groupId)
  if (group) {
    groupName.value = group.name
    selectedTags.value = [...group.tags]
    originalName.value = group.name
    originalTags.value = [...group.tags]
    hasChanges.value = false
  }
}

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
  if (isMaxReached.value) {
    message.warning('每个分组最多包含 20 个标签')
    return
  }
  if (selectedTags.value.includes(tag)) return
  
  selectedTags.value.push(tag)
}

// 移除标签
function removeTag(tag: string) {
  selectedTags.value = selectedTags.value.filter(t => t !== tag)
}

// 返回（检查未保存变更）
function handleBack() {
  if (hasChanges.value) {
    dialog.warning({
      title: '未保存的更改',
      content: '您有未保存的更改，确定要离开吗？',
      positiveText: '离开',
      negativeText: '继续编辑',
      onPositiveClick: () => {
        emit('back')
      }
    })
  } else {
    emit('back')
  }
}

// 保存分组
async function handleSave() {
  if (!groupName.value.trim()) {
    message.warning('请输入分组名称')
    return
  }
  
  saving.value = true
  
  try {
    const result = await tagGroupStore.updateGroup(
      props.groupId,
      groupName.value.trim(),
      selectedTags.value
    )
    
    if (result) {
      message.success('分组已保存')
      originalName.value = result.name
      originalTags.value = [...result.tags]
      hasChanges.value = false
      emit('saved', result)
      emit('back')
    } else if (tagGroupStore.error) {
      message.error(tagGroupStore.error)
    }
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="group-edit-view">
    <!-- 顶部栏 -->
    <div class="edit-header glass">
      <div class="header-left">
        <NButton quaternary circle @click="handleBack">
          <template #icon>
            <NIcon :component="ArrowBackOutline" />
          </template>
        </NButton>
        <span class="header-title">编辑分组</span>
      </div>
      
      <div class="header-right">
        <NButton 
          type="primary" 
          :loading="saving"
          :disabled="!hasChanges"
          @click="handleSave"
        >
          <template #icon>
            <NIcon :component="SaveOutline" />
          </template>
          保存
        </NButton>
      </div>
    </div>
    
    <!-- 编辑内容 -->
    <div class="edit-content">
      <!-- 分组名称 -->
      <div class="edit-section">
        <div class="section-label">分组名称</div>
        <NInput
          v-model:value="groupName"
          placeholder="输入分组名称"
          maxlength="50"
          show-count
          class="name-input"
        />
      </div>
      
      <!-- 搜索标签 -->
      <div class="edit-section">
        <div class="section-label">搜索标签添加</div>
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
        <div v-if="searchKeyword.trim()" class="search-results glass-card">
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
      </div>
      
      <!-- 已添加标签 -->
      <div class="edit-section tags-section">
        <div class="section-label">
          已添加标签
          <span class="tag-count" :class="{ warning: isMaxReached }">
            {{ selectedTags.length }}/20
          </span>
        </div>
        
        <div class="selected-tags glass-card">
          <NScrollbar style="max-height: 300px">
            <NEmpty 
              v-if="selectedTags.length === 0" 
              size="small"
              description="搜索并点击标签添加到分组"
            />
            
            <NSpace v-else :size="[8, 8]" style="padding: 12px">
              <NTag
                v-for="tag in selectedTags"
                :key="tag"
                closable
                round
                type="info"
                size="medium"
                @close="removeTag(tag)"
              >
                {{ tag }}
              </NTag>
            </NSpace>
          </NScrollbar>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.group-edit-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-base);
}

.edit-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
  background: var(--header-bg);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.edit-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  max-width: 800px;
}

.edit-section {
  margin-bottom: 24px;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
}

.tag-count {
  font-weight: normal;
  color: var(--text-muted);
}

.tag-count.warning {
  color: #f59e0b;
}

.name-input {
  max-width: 400px;
}

.search-results {
  margin-top: 12px;
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

.tags-section {
  flex: 1;
}

.selected-tags {
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  min-height: 150px;
}
</style>
