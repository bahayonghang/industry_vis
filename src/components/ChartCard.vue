<script setup lang="ts">
import { ref, computed } from 'vue'
import { 
  NButton, 
  NInput, 
  NIcon, 
  NPopconfirm,
  NTooltip,
  NEmpty
} from 'naive-ui'
import { 
  TrashOutline, 
  CreateOutline,
  AddOutline,
  CloseOutline
} from '@vicons/ionicons5'
import type { ChartConfig } from '@/types'

const props = defineProps<{
  chart: ChartConfig
  index: number
  canDelete: boolean
}>()

const emit = defineEmits<{
  (e: 'update:name', value: string): void
  (e: 'delete'): void
  (e: 'add-tag'): void
  (e: 'remove-tag', tagName: string): void
}>()

// 编辑名称状态
const isEditing = ref(false)
const editName = ref('')

// 标签数量状态
const tagCount = computed(() => props.chart.tags.length)
const canAddTag = computed(() => tagCount.value < 5)

// 开始编辑名称
function startEditName() {
  editName.value = props.chart.name
  isEditing.value = true
}

// 保存名称
function saveName() {
  if (editName.value.trim()) {
    emit('update:name', editName.value.trim())
  }
  isEditing.value = false
}

// 取消编辑
function cancelEdit() {
  isEditing.value = false
}

// 键盘事件
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    saveName()
  } else if (e.key === 'Escape') {
    cancelEdit()
  }
}
</script>

<template>
  <div class="chart-card animate-scale-in" :style="{ animationDelay: `${index * 0.05}s` }">
    <!-- 卡片头部 -->
    <div class="chart-card-header">
      <div class="chart-card-title">
        <template v-if="!isEditing">
          <span class="title-text">{{ chart.name }}</span>
          <NTooltip>
            <template #trigger>
              <NButton quaternary circle size="tiny" @click="startEditName">
                <template #icon>
                  <NIcon :component="CreateOutline" size="14" />
                </template>
              </NButton>
            </template>
            编辑名称
          </NTooltip>
        </template>
        <template v-else>
          <NInput 
            v-model:value="editName" 
            size="small"
            placeholder="图表名称"
            @keydown="handleKeydown"
            @blur="saveName"
            autofocus
            style="width: 150px"
          />
        </template>
      </div>
      
      <div class="chart-card-actions">
        <span class="tag-count">{{ tagCount }}/5</span>
        <NPopconfirm 
          v-if="canDelete"
          @positive-click="emit('delete')"
        >
          <template #trigger>
            <NButton quaternary circle size="small" type="error">
              <template #icon>
                <NIcon :component="TrashOutline" size="16" />
              </template>
            </NButton>
          </template>
          确定删除该图表吗？
        </NPopconfirm>
      </div>
    </div>
    
    <!-- 卡片内容 - 标签列表 -->
    <div class="chart-card-body">
      <div v-if="chart.tags.length > 0" class="tag-list">
        <div 
          v-for="tag in chart.tags" 
          :key="tag" 
          class="tag-pill"
        >
          <span class="tag-name">{{ tag }}</span>
          <span class="tag-pill-remove" @click="emit('remove-tag', tag)">
            <NIcon :component="CloseOutline" size="12" />
          </span>
        </div>
      </div>
      
      <NEmpty v-else description="暂无标签" size="small" style="padding: 20px 0">
        <template #extra>
          <NButton size="small" @click="emit('add-tag')" :disabled="!canAddTag">
            添加标签
          </NButton>
        </template>
      </NEmpty>
    </div>
    
    <!-- 卡片底部 - 添加标签按钮 -->
    <div v-if="chart.tags.length > 0" class="chart-card-footer">
      <NButton 
        size="small" 
        quaternary 
        @click="emit('add-tag')"
        :disabled="!canAddTag"
      >
        <template #icon>
          <NIcon :component="AddOutline" />
        </template>
        添加标签
      </NButton>
    </div>
  </div>
</template>

<style scoped>
.chart-card {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--glass-border);
  border-radius: 16px;
  box-shadow: var(--glass-shadow);
  overflow: hidden;
  transition: 
    background var(--transition-normal),
    border-color var(--transition-normal),
    box-shadow var(--transition-normal),
    transform var(--transition-normal);
}

.chart-card:hover {
  background: var(--glass-bg-hover);
  border-color: var(--glass-border-hover);
  box-shadow: var(--glass-shadow-hover);
  transform: translateY(-3px);
}

.chart-card-header {
  background: var(--gradient-header);
  border-bottom: 1px solid var(--glass-border);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.chart-card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.chart-card-title::before {
  content: '';
  display: inline-block;
  width: 4px;
  height: 16px;
  border-radius: 2px;
  background: var(--industrial-blue);
  flex-shrink: 0;
}

.title-text {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chart-card-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tag-count {
  font-size: 12px;
  color: var(--text-tertiary);
  padding: 2px 8px;
  background: var(--glass-bg);
  border-radius: 10px;
}

.chart-card-body {
  padding: 16px;
  min-height: 100px;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.tag-pill:hover {
  background: var(--glass-bg-hover);
  border-color: var(--industrial-blue);
}

.tag-name {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-pill-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  cursor: pointer;
  transition: all var(--transition-fast);
  color: var(--text-tertiary);
}

.tag-pill-remove:hover {
  background: var(--industrial-red);
  color: white;
}

.chart-card-footer {
  padding: 8px 16px;
  border-top: 1px solid var(--glass-border);
  background: rgba(0, 0, 0, 0.02);
}
</style>
