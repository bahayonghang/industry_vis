<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NSelect, NSpace } from 'naive-ui'
import { useDataStore } from '@/stores/data'

const dataStore = useDataStore()

const selectedTags = ref<string[]>([])

const tagOptions = computed(() => 
  dataStore.availableTags.map(tag => ({ label: tag, value: tag }))
)

const handleTagChange = (tags: string[]) => {
  selectedTags.value = tags
  dataStore.setSelectedTags(tags)
}

onMounted(async () => {
  await dataStore.fetchAvailableTags()
})
</script>

<template>
  <NSpace align="center">
    <span class="label">标签选择：</span>
    <NSelect
      v-model:value="selectedTags"
      :options="tagOptions"
      multiple
      filterable
      clearable
      placeholder="选择标签..."
      style="min-width: 300px"
      max-tag-count="responsive"
      @update:value="handleTagChange"
    />
  </NSpace>
</template>

<style scoped>
.label {
  font-size: 14px;
  color: #666;
}
</style>
