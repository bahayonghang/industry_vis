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
import TagSearchSelect from './TagSearchSelect.vue'
import { useTagGroupStore } from '@/stores/tagGroup'
import type { TagGroup } from '@/types'

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
  tags: [] as string[]
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
      tags: [...group.tags]
    }
  } else {
    formValue.value = {
      name: '',
      tags: []
    }
  }
}, { immediate: true })

// 监听弹窗关闭时重置表单
watch(() => props.show, (show) => {
  if (!show && !props.editGroup) {
    formValue.value = {
      name: '',
      tags: []
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
  
  const { name, tags } = formValue.value
  
  let result: TagGroup | null = null
  
  if (isEditMode.value && props.editGroup) {
    // 更新
    result = await tagGroupStore.updateGroup(props.editGroup.id, name, tags)
    if (result) {
      message.success('分组已更新')
    }
  } else {
    // 创建
    result = await tagGroupStore.createGroup(name, tags)
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
      
      <!-- 编辑模式才显示标签选择 -->
      <NFormItem v-if="isEditMode" label="选择标签">
        <TagSearchSelect
          v-model="formValue.tags"
          :max-tags="20"
        />
      </NFormItem>
      
      <div v-else class="create-hint">
        创建分组后，点击编辑按钮添加标签
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
.create-hint {
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  color: var(--text-muted);
  font-size: 13px;
  text-align: center;
}
</style>
