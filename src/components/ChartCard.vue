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
/* ===== 赛博朋克图表卡片 ===== */
.chart-card {
  position: relative;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  box-shadow: var(--glass-shadow);
  overflow: hidden;
  transition:
    background var(--transition-normal),
    border-color var(--transition-normal),
    box-shadow var(--transition-normal),
    transform var(--transition-normal);
}

/* 顶部霓虹渐变条 */
.chart-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(
    90deg,
    var(--neon-cyan) 0%,
    var(--neon-magenta) 50%,
    var(--neon-orange) 100%
  );
  opacity: 0.8;
  transition: opacity var(--transition-normal);
}

/* 角落装饰 */
.chart-card::after {
  content: '';
  position: absolute;
  top: 10px;
  right: 10px;
  width: 14px;
  height: 14px;
  border-top: 2px solid var(--neon-cyan);
  border-right: 2px solid var(--neon-cyan);
  opacity: 0.4;
  transition: opacity var(--transition-fast);
}

.chart-card:hover {
  background: var(--glass-bg-hover);
  border-color: var(--border-glow);
  box-shadow: var(--glass-shadow-hover);
  transform: translateY(-4px);
}

.chart-card:hover::before {
  opacity: 1;
}

.chart-card:hover::after {
  opacity: 0.8;
}

.chart-card-header {
  position: relative;
  background: linear-gradient(
    90deg,
    rgba(0, 245, 255, 0.05) 0%,
    rgba(255, 0, 255, 0.03) 100%
  );
  border-bottom: 1px solid var(--border-default);
  padding: 14px 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.chart-card-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
}

/* 标题前的霓虹指示条 */
.chart-card-title::before {
  content: '';
  display: inline-block;
  width: 3px;
  height: 16px;
  border-radius: 2px;
  background: linear-gradient(180deg, var(--neon-cyan), var(--neon-magenta));
  box-shadow: 0 0 8px var(--neon-cyan-glow);
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
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
  color: var(--text-muted);
  padding: 3px 10px;
  background: rgba(0, 245, 255, 0.08);
  border: 1px solid rgba(0, 245, 255, 0.15);
  border-radius: var(--radius-sm);
}

.chart-card-body {
  padding: 18px;
  min-height: 100px;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

/* 霓虹标签胶囊 */
.tag-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  border-radius: var(--radius-full);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  background: rgba(0, 245, 255, 0.08);
  border: 1px solid rgba(0, 245, 255, 0.2);
  color: var(--neon-cyan);
  transition: all var(--transition-fast);
}

.tag-pill:hover {
  background: rgba(0, 245, 255, 0.15);
  border-color: var(--neon-cyan);
  box-shadow: 0 0 12px var(--neon-cyan-glow);
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
  color: var(--text-muted);
}

.tag-pill-remove:hover {
  background: var(--neon-red);
  color: white;
  box-shadow: 0 0 8px var(--neon-red-glow);
}

.chart-card-footer {
  padding: 10px 18px;
  border-top: 1px solid var(--border-default);
  background: rgba(0, 0, 0, 0.15);
}

/* 入场动画 */
@keyframes scale-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.animate-scale-in {
  animation: scale-in 0.4s var(--ease-cyber) forwards;
  opacity: 0;
}
</style>
