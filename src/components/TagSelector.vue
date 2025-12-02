<script setup lang="ts">
import { computed } from 'vue'
import { NSelect, NSpace, NIcon, NText } from 'naive-ui'
import { PricetagsOutline } from '@vicons/ionicons5'
import { useDataStore } from '@/stores/data'

const dataStore = useDataStore()

const selectedTags = computed({
  get: () => dataStore.selectedTags,
  set: (tags: string[]) => dataStore.setSelectedTags(tags),
})

const tagOptions = computed(() => 
  dataStore.availableTags.map(tag => ({ label: tag, value: tag }))
)

const handleTagChange = (tags: string[]) => {
  dataStore.setSelectedTags(tags)
}
</script>

<template>
  <NSpace align="center" :size="12">
    <NSpace align="center" :size="6">
      <NIcon :component="PricetagsOutline" :size="18" class="label-icon" />
      <NText class="label">标签选择</NText>
    </NSpace>
    <NSelect
      v-model:value="selectedTags"
      :options="tagOptions"
      multiple
      filterable
      clearable
      placeholder="选择要查询的标签..."
      style="min-width: 360px"
      max-tag-count="responsive"
      :loading="dataStore.availableTags.length === 0"
      @update:value="handleTagChange"
    />
    <NText v-if="selectedTags.length > 0" depth="3" class="tag-count">
      已选 {{ selectedTags.length }} 个
    </NText>
  </NSpace>
</template>

<style scoped>
.label-icon {
  color: var(--text-muted);
}

.label {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}

.tag-count {
  font-size: 13px;
}
</style>
