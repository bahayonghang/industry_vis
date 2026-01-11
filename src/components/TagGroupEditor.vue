<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { 
  NModal, 
  NForm, 
  NFormItem, 
  NInput, 
  NButton, 
  NSpace,
  useMessage 
} from 'naive-ui'
// TagSearchSelect 已移至 GroupEditView
import { useTagGroupStore } from '@/stores/tagGroup'
import type { ChartConfig, TagGroup } from '@/types'
import { createDefaultChartConfig } from '@/types'

const props = defineProps<{
  show: boolean
  editGroup?: TagGroup | null  // 编辑模式时传入
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  'saved': [group: TagGroup]
}>()

const tagGroupStore = useTagGroupStore()
const message = useMessage()

// 表单数据
const formValue = ref({
  name: '',
  charts: [] as ChartConfig[]
})

// 是否编辑模式
const isEditMode = computed(() => !!props.editGroup)

// 标题
const modalTitle = computed(() => isEditMode.value ? '编辑标签分组' : '创建标签分组')

// 监听编辑分组变化，填充表单
watch(() => props.editGroup, (group) => {
  if (group) {
    formValue.value = {
      name: group.name,
      charts: JSON.parse(JSON.stringify(group.charts || []))
    }
  } else {
    formValue.value = {
      name: '',
      charts: []
    }
  }
}, { immediate: true })

// 监听弹窗关闭时重置表单
watch(() => props.show, (show) => {
  if (!show && !props.editGroup) {
    formValue.value = {
      name: '',
      charts: []
    }
  }
})

// 关闭弹窗
function handleClose() {
  emit('update:show', false)
}

// 表单验证
function validate(): boolean {
  if (!formValue.value.name.trim()) {
    message.warning('请输入分组名称')
    return false
  }
  
  // 编辑模式下不强制要求有标签
  // 创建模式下也允许创建空分组
  return true
}

// 保存分组
async function handleSave() {
  if (!validate()) return
  
  const { name, charts } = formValue.value
  
  let result: TagGroup | null = null
  
  if (isEditMode.value && props.editGroup) {
    // 更新
    result = await tagGroupStore.updateGroup(props.editGroup.id, name, charts)
    if (result) {
      message.success('分组已更新')
    }
  } else {
    // 创建（自动创建默认图表）
    const initialCharts = charts.length > 0 ? charts : [createDefaultChartConfig('默认图表')]
    result = await tagGroupStore.createGroup(name, initialCharts)
    if (result) {
      message.success('分组已创建，点击编辑添加标签')
    }
  }
  
  if (result) {
    emit('saved', result)
    handleClose()
  } else if (tagGroupStore.error) {
    message.error(tagGroupStore.error)
  }
}
</script>

<template>
  <NModal
    :show="show"
    :mask-closable="false"
    preset="card"
    :title="modalTitle"
    style="width: 500px; max-width: 90vw"
    @update:show="emit('update:show', $event)"
  >
    <NForm :model="formValue" label-placement="top">
      <NFormItem label="分组名称" required>
        <NInput
          v-model:value="formValue.name"
          placeholder="例如：分解炉温度"
          maxlength="50"
          show-count
        />
      </NFormItem>
      
      <div class="create-hint">
        {{ isEditMode ? '请在分组编辑页面管理图表和标签' : '创建分组后，点击编辑按钮添加图表和标签' }}
      </div>
    </NForm>
    
    <template #footer>
      <NSpace justify="end">
        <NButton @click="handleClose">取消</NButton>
        <NButton 
          type="primary" 
          :loading="tagGroupStore.loading"
          @click="handleSave"
        >
          {{ isEditMode ? '保存修改' : '创建分组' }}
        </NButton>
      </NSpace>
    </template>
  </NModal>
</template>

<style scoped>
/* ===== 赛博朋克标签分组编辑器 ===== */
.create-hint {
  padding: 14px 18px;
  background: linear-gradient(
    90deg,
    rgba(0, 245, 255, 0.05) 0%,
    rgba(255, 0, 255, 0.03) 100%
  );
  border: 1px solid rgba(0, 245, 255, 0.15);
  border-radius: var(--radius-lg);
  color: var(--text-muted);
  font-family: var(--font-body);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
  text-align: center;
}

/* 弹窗霓虹边框 */
:deep(.n-modal) {
  border: 1px solid var(--border-default);
  overflow: hidden;
}

/* 弹窗顶部霓虹条 */
:deep(.n-card) {
  position: relative;
}

:deep(.n-card)::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--neon-cyan), var(--neon-magenta));
  opacity: 0.8;
  z-index: 1;
}

/* 弹窗标题样式 */
:deep(.n-card-header__main) {
  font-family: var(--font-display);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
}

/* 表单标签样式 */
:deep(.n-form-item-label__text) {
  font-family: var(--font-body);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
  color: var(--text-secondary);
}

/* 输入框霓虹聚焦效果 */
:deep(.n-input:focus-within) {
  border-color: var(--neon-cyan);
  box-shadow: 0 0 12px var(--neon-cyan-glow);
}

/* 底部按钮区域 */
:deep(.n-card__footer) {
  border-top: 1px solid var(--border-default);
  background: rgba(0, 0, 0, 0.1);
}
</style>
