<script setup lang="ts">
import { onMounted } from 'vue'
import { 
  NButton, 
  NIcon, 
  NEmpty, 
  NSpin,
  NTooltip,
  NPopconfirm,
  NInput,
  useMessage,
  useDialog
} from 'naive-ui'
import { 
  AddOutline, 
  FolderOpenOutline,
  TrashOutline,
  CreateOutline,
  AnalyticsOutline
} from '@vicons/ionicons5'
import { useTagGroupStore } from '@/stores/tagGroup'
import { useDataStore } from '@/stores/data'
import type { TagGroup } from '@/types'

const tagGroupStore = useTagGroupStore()
const dataStore = useDataStore()
const message = useMessage()
const dialog = useDialog()

const emit = defineEmits<{
  'edit': [groupId: string]
}>()

// 加载分组列表
onMounted(() => {
  tagGroupStore.loadGroups()
})

// 创建新分组
function handleCreate() {
  dialog.create({
    title: '新建分组',
    content: () => {
      const inputRef = { value: '' }
      return h('div', { style: 'padding: 8px 0' }, [
        h(NInput, {
          placeholder: '输入分组名称',
          maxlength: 50,
          onUpdateValue: (v: string) => { inputRef.value = v },
          ref: 'nameInput'
        })
      ])
    },
    positiveText: '创建',
    negativeText: '取消',
    onPositiveClick: async () => {
      const name = (document.querySelector('.n-dialog .n-input__input-el') as HTMLInputElement)?.value
      if (!name?.trim()) {
        message.warning('请输入分组名称')
        return false
      }
      const result = await tagGroupStore.createGroup(name.trim(), [])
      if (result) {
        message.success('分组已创建')
        // 自动进入编辑页面
        emit('edit', result.id)
      } else if (tagGroupStore.error) {
        message.error(tagGroupStore.error)
        return false
      }
    }
  })
}

// 编辑分组（进入编辑页面）
function handleEdit(group: TagGroup, event: Event) {
  event.stopPropagation()
  emit('edit', group.id)
}

// 删除分组
async function handleDelete(group: TagGroup, event: Event) {
  event.stopPropagation()
  
  const success = await tagGroupStore.deleteGroup(group.id)
  if (success) {
    message.success(`已删除分组 "${group.name}"`)
  } else if (tagGroupStore.error) {
    message.error(tagGroupStore.error)
  }
}

// 获取分组中所有标签
function getAllTags(group: TagGroup): string[] {
  return (group.charts || []).flatMap(c => c.tags || [])
}

// 选择分组（应用标签查询）
function handleSelect(group: TagGroup) {
  const allTags = getAllTags(group)
  
  // 如果分组为空，直接进入编辑
  if (allTags.length === 0) {
    emit('edit', group.id)
    return
  }
  
  // 如果当前有数据且选择了不同的分组，提示确认
  if (
    dataStore.records.length > 0 && 
    tagGroupStore.selectedGroupId !== group.id
  ) {
    dialog.warning({
      title: '切换分组',
      content: `切换到分组 "${group.name}" 将替换当前查询的标签，是否继续？`,
      positiveText: '确认切换',
      negativeText: '取消',
      onPositiveClick: () => {
        applyGroup(group)
      }
    })
  } else {
    applyGroup(group)
  }
}

// 应用分组
function applyGroup(group: TagGroup) {
  const allTags = getAllTags(group)
  tagGroupStore.selectGroup(group.id)
  dataStore.setSelectedTags(allTags)
  message.info(`已应用分组 "${group.name}"，包含 ${allTags.length} 个标签`)
}

// 需要导入 h 函数
import { h } from 'vue'
</script>

<template>
  <div class="tag-group-tree">
    <!-- 标题栏 -->
    <div class="tree-header">
      <div class="header-title">
        <NIcon :component="FolderOpenOutline" :size="16" />
        <span>标签分组</span>
      </div>
      <NTooltip trigger="hover">
        <template #trigger>
          <NButton 
            text 
            size="small"
            @click="handleCreate"
          >
            <template #icon>
              <NIcon :component="AddOutline" />
            </template>
          </NButton>
        </template>
        新建分组
      </NTooltip>
    </div>

    <!-- 分组列表 -->
    <div class="tree-content">
      <NSpin v-if="tagGroupStore.loading" size="small" />
      
      <NEmpty 
        v-else-if="tagGroupStore.groups.length === 0"
        size="small"
        description="暂无分组"
      >
        <template #extra>
          <NButton size="small" @click="handleCreate">
            创建第一个分组
          </NButton>
        </template>
      </NEmpty>
      
      <div v-else class="group-list">
        <div
          v-for="group in tagGroupStore.groups"
          :key="group.id"
          class="group-item"
          :class="{ 
            active: tagGroupStore.selectedGroupId === group.id,
            empty: getAllTags(group).length === 0
          }"
          @click="handleSelect(group)"
        >
          <div class="group-info">
            <NIcon :component="AnalyticsOutline" :size="16" class="group-icon" />
            <span class="group-name">{{ group.name }}</span>
            <span v-if="getAllTags(group).length > 0" class="group-count">({{ getAllTags(group).length }})</span>
            <span v-else class="group-empty-hint">点击添加标签</span>
          </div>
          
          <div class="group-actions">
            <NTooltip trigger="hover">
              <template #trigger>
                <NButton 
                  text 
                  size="tiny"
                  @click="handleEdit(group, $event)"
                >
                  <template #icon>
                    <NIcon :component="CreateOutline" :size="14" />
                  </template>
                </NButton>
              </template>
              编辑
            </NTooltip>
            
            <NPopconfirm
              :positive-text="'删除'"
              :negative-text="'取消'"
              @positive-click="handleDelete(group, $event)"
            >
              <template #trigger>
                <NButton 
                  text 
                  size="tiny"
                  type="error"
                  @click.stop
                >
                  <template #icon>
                    <NIcon :component="TrashOutline" :size="14" />
                  </template>
                </NButton>
              </template>
              确定删除分组 "{{ group.name }}" 吗？
            </NPopconfirm>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ===== 赛博朋克标签分组树 ===== */
.tag-group-tree {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.tree-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-default);
  background: linear-gradient(
    90deg,
    rgba(0, 245, 255, 0.03) 0%,
    transparent 100%
  );
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-secondary);
}

/* 标题图标霓虹效果 */
.header-title :deep(.n-icon) {
  color: var(--neon-cyan);
  filter: drop-shadow(0 0 4px var(--neon-cyan-glow));
}

.tree-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.group-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.group-item {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--transition-fast);
  background: transparent;
  border: 1px solid transparent;
}

.group-item:hover {
  background: var(--bg-hover);
  border-color: rgba(0, 245, 255, 0.15);
}

.group-item.active {
  background: rgba(0, 245, 255, 0.08);
  border-color: rgba(0, 245, 255, 0.25);
  border-left: 3px solid var(--neon-cyan);
  padding-left: 11px;
  box-shadow: 0 0 15px var(--neon-cyan-glow);
}

.group-info {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.group-icon {
  color: var(--text-muted);
  flex-shrink: 0;
  transition: all var(--transition-fast);
}

.group-item:hover .group-icon {
  color: var(--neon-cyan);
}

.group-item.active .group-icon {
  color: var(--neon-cyan);
  filter: drop-shadow(0 0 4px var(--neon-cyan-glow));
}

.group-name {
  font-family: var(--font-body);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-count {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
  color: var(--text-muted);
  flex-shrink: 0;
  padding: 2px 8px;
  background: rgba(0, 245, 255, 0.08);
  border-radius: var(--radius-sm);
}

.group-empty-hint {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
  color: var(--neon-orange);
  flex-shrink: 0;
  opacity: 0.9;
}

.group-item.empty {
  border: 1px dashed var(--border-default);
  background: transparent;
}

.group-item.empty:hover {
  border-color: var(--neon-cyan);
  background: rgba(0, 245, 255, 0.05);
  box-shadow: 0 0 12px var(--neon-cyan-glow);
}

.group-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.group-item:hover .group-actions {
  opacity: 1;
}

/* 操作按钮霓虹效果 */
.group-actions :deep(.n-button:hover) {
  color: var(--neon-cyan);
}

.group-actions :deep(.n-button--error-type:hover) {
  color: var(--neon-red);
}
</style>
