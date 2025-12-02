<script setup lang="ts">
import { ref, computed } from 'vue'
import { 
  NSpace, 
  NButton, 
  NButtonGroup,
  NDatePicker, 
  NTooltip,
  NIcon,
  NDivider,
  NBadge
} from 'naive-ui'
import { 
  RefreshOutline, 
  DownloadOutline,
  PlayOutline,
  TimeOutline
} from '@vicons/ionicons5'
import { useDataStore } from '@/stores/data'

const dataStore = useDataStore()

// 预设时间快捷按钮
type PresetKey = 'realtime' | '1h' | 'shift' | 'today' | 'yesterday' | 'week' | 'custom'

const presets: { key: PresetKey; label: string; tooltip: string }[] = [
  { key: 'realtime', label: '实时', tooltip: '最近5分钟，自动刷新' },
  { key: '1h', label: '1H', tooltip: '最近1小时' },
  { key: 'shift', label: '班次', tooltip: '当前班次（8小时）' },
  { key: 'today', label: '今日', tooltip: '今日 00:00 至今' },
  { key: 'yesterday', label: '昨日', tooltip: '昨日全天' },
  { key: 'week', label: '本周', tooltip: '本周一至今' },
]

const selectedPreset = ref<PresetKey>('today')
const customRange = ref<[number, number] | null>(null)
const showCustomPicker = ref(false)

const loading = computed(() => dataStore.loading)
const total = computed(() => dataStore.total)

const getPresetRange = (preset: PresetKey): [Date, Date] => {
  const now = new Date()
  const start = new Date()
  
  switch (preset) {
    case 'realtime':
      start.setMinutes(now.getMinutes() - 5)
      break
    case '1h':
      start.setHours(now.getHours() - 1)
      break
    case 'shift':
      // 假设8小时班次
      start.setHours(now.getHours() - 8)
      break
    case 'today':
      start.setHours(0, 0, 0, 0)
      break
    case 'yesterday':
      start.setDate(now.getDate() - 1)
      start.setHours(0, 0, 0, 0)
      const end = new Date(start)
      end.setHours(23, 59, 59, 999)
      return [start, end]
    case 'week':
      const dayOfWeek = now.getDay()
      const diff = dayOfWeek === 0 ? 6 : dayOfWeek - 1 // 调整为周一
      start.setDate(now.getDate() - diff)
      start.setHours(0, 0, 0, 0)
      break
    default:
      start.setHours(0, 0, 0, 0)
  }
  
  return [start, now]
}

const handlePresetClick = (preset: PresetKey) => {
  selectedPreset.value = preset
  showCustomPicker.value = false
  
  const [start, end] = getPresetRange(preset)
  dataStore.setTimeRange(start, end)
}

const handleCustomClick = () => {
  showCustomPicker.value = !showCustomPicker.value
  selectedPreset.value = 'custom'
}

const handleCustomRangeChange = (range: [number, number] | null) => {
  if (range) {
    customRange.value = range
    dataStore.setTimeRange(new Date(range[0]), new Date(range[1]))
  }
}

const handleQuery = () => {
  dataStore.fetchData()
}

const handleExport = () => {
  dataStore.exportToCsv()
}

// 初始化默认时间范围
const [defaultStart, defaultEnd] = getPresetRange('today')
dataStore.setTimeRange(defaultStart, defaultEnd)
</script>

<template>
  <div class="query-toolbar glass">
    <NSpace align="center" :size="12">
      <!-- 时间图标 -->
      <NIcon :component="TimeOutline" :size="18" class="toolbar-icon" />
      
      <!-- 时间快捷按钮 -->
      <NButtonGroup size="small">
        <NTooltip v-for="preset in presets" :key="preset.key">
          <template #trigger>
            <NButton
              :type="selectedPreset === preset.key ? 'primary' : 'default'"
              :tertiary="selectedPreset !== preset.key"
              @click="handlePresetClick(preset.key)"
            >
              {{ preset.label }}
            </NButton>
          </template>
          {{ preset.tooltip }}
        </NTooltip>
      </NButtonGroup>
      
      <!-- 自定义时间 -->
      <NTooltip>
        <template #trigger>
          <NButton 
            size="small"
            :type="selectedPreset === 'custom' ? 'primary' : 'default'"
            :tertiary="selectedPreset !== 'custom'"
            @click="handleCustomClick"
          >
            自定义
          </NButton>
        </template>
        选择自定义时间范围
      </NTooltip>
      
      <!-- 自定义时间选择器 -->
      <NDatePicker
        v-if="showCustomPicker"
        v-model:value="customRange"
        type="datetimerange"
        size="small"
        clearable
        :shortcuts="{
          '近3天': () => [Date.now() - 3 * 24 * 60 * 60 * 1000, Date.now()],
          '近30天': () => [Date.now() - 30 * 24 * 60 * 60 * 1000, Date.now()],
        }"
        @update:value="handleCustomRangeChange"
      />
    </NSpace>
    
    <NDivider vertical style="height: 24px; margin: 0 8px" />
    
    <NSpace align="center" :size="12">
      <!-- 查询按钮 -->
      <NButton 
        type="primary" 
        size="small"
        :loading="loading"
        @click="handleQuery"
      >
        <template #icon>
          <NIcon :component="PlayOutline" />
        </template>
        查询
      </NButton>
      
      <!-- 刷新按钮 -->
      <NTooltip>
        <template #trigger>
          <NButton 
            size="small" 
            tertiary
            :loading="loading"
            @click="handleQuery"
          >
            <template #icon>
              <NIcon :component="RefreshOutline" />
            </template>
          </NButton>
        </template>
        刷新数据
      </NTooltip>
      
      <!-- 导出按钮 -->
      <NTooltip>
        <template #trigger>
          <NButton 
            size="small" 
            tertiary
            :disabled="total === 0"
            @click="handleExport"
          >
            <template #icon>
              <NIcon :component="DownloadOutline" />
            </template>
          </NButton>
        </template>
        导出 CSV
      </NTooltip>
      
      <NDivider vertical style="height: 24px; margin: 0 4px" />
      
      <!-- 数据量提示 -->
      <NBadge :value="total" :max="99999" show-zero>
        <span class="data-count-label">数据量</span>
      </NBadge>
    </NSpace>
  </div>
</template>

<style scoped>
.query-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: var(--radius-lg);
  margin-bottom: 16px;
}

.toolbar-icon {
  color: var(--text-muted);
}

.data-count-label {
  font-size: 13px;
  color: var(--text-secondary);
  padding: 4px 8px;
}
</style>
