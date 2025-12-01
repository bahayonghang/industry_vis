<script setup lang="ts">
import { ref, computed } from 'vue'
import { NSelect, NDatePicker, NSpace } from 'naive-ui'
import { useDataStore } from '@/stores/data'

const dataStore = useDataStore()

type PresetKey = '1h' | '6h' | '24h' | '7d' | 'custom'

const presetOptions = [
  { label: '最近1小时', value: '1h' },
  { label: '最近6小时', value: '6h' },
  { label: '最近24小时', value: '24h' },
  { label: '最近7天', value: '7d' },
  { label: '自定义', value: 'custom' },
]

const selectedPreset = ref<PresetKey>('24h')
const customRange = ref<[number, number] | null>(null)

const isCustom = computed(() => selectedPreset.value === 'custom')

const getPresetRange = (preset: PresetKey): [Date, Date] => {
  const now = new Date()
  const start = new Date()
  
  switch (preset) {
    case '1h':
      start.setHours(now.getHours() - 1)
      break
    case '6h':
      start.setHours(now.getHours() - 6)
      break
    case '24h':
      start.setHours(now.getHours() - 24)
      break
    case '7d':
      start.setDate(now.getDate() - 7)
      break
    default:
      start.setHours(now.getHours() - 24)
  }
  
  return [start, now]
}

const handlePresetChange = (value: PresetKey) => {
  selectedPreset.value = value
  if (value !== 'custom') {
    const [start, end] = getPresetRange(value)
    dataStore.setTimeRange(start, end)
  }
}

const handleCustomRangeChange = (range: [number, number] | null) => {
  if (range) {
    customRange.value = range
    dataStore.setTimeRange(new Date(range[0]), new Date(range[1]))
  }
}

// Initialize with default range
const [defaultStart, defaultEnd] = getPresetRange('24h')
dataStore.setTimeRange(defaultStart, defaultEnd)
</script>

<template>
  <NSpace align="center">
    <span class="label">时间范围：</span>
    <NSelect
      v-model:value="selectedPreset"
      :options="presetOptions"
      style="width: 140px"
      @update:value="handlePresetChange"
    />
    <NDatePicker
      v-if="isCustom"
      v-model:value="customRange"
      type="datetimerange"
      clearable
      @update:value="handleCustomRangeChange"
    />
  </NSpace>
</template>

<style scoped>
.label {
  font-size: 14px;
  color: #666;
}
</style>
