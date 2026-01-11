<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { 
  NButton, 
  NIcon, 
  NInput, 
  NSpace, 
  NTooltip,
  NButtonGroup,
  NDatePicker,
  NDivider,
  NPopconfirm,
  NCheckbox,
  NInputNumber,
  NCard,
  useMessage,
  useDialog
} from 'naive-ui'
import { 
  ArrowBackOutline, 
  SaveOutline,
  TrashOutline,
  AddOutline,
  PlayOutline,
  TimeOutline,
  FlashOutline,
  CheckmarkCircleOutline
} from '@vicons/ionicons5'
import { useTagGroupStore } from '@/stores/tagGroup'
import { useDataStore } from '@/stores/data'
import LineChart from '@/components/LineChart.vue'
import ChartCard from '@/components/ChartCard.vue'
import TagSearchModal from '@/components/TagSearchModal.vue'
import type { ChartConfig, DataProcessingConfig, TagGroup } from '@/types'
import { createDefaultProcessingConfig, createDefaultChartConfig } from '@/types'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  groupId: string
}>()

const emit = defineEmits<{
  'back': []
  'saved': [group: TagGroup]
  'deleted': []
}>()

const tagGroupStore = useTagGroupStore()
const dataStore = useDataStore()
const message = useMessage()
const dialog = useDialog()

// 编辑状态
const groupName = ref('')
const charts = ref<ChartConfig[]>([])
const hasChanges = ref(false)
const saving = ref(false)
const showTagModal = ref(false)
const activeChartId = ref<string | null>(null)  // 当前添加标签的图表

// 数据处理配置 - 使用 reactive 确保嵌套属性的响应式更新
const processingConfig = reactive<DataProcessingConfig>(createDefaultProcessingConfig())

// 原始数据（用于检测变更）
const originalName = ref('')
const originalCharts = ref<ChartConfig[]>([])
const originalProcessingConfig = ref<DataProcessingConfig>(createDefaultProcessingConfig())

// 图表数量限制
const canAddChart = computed(() => charts.value.length < 10)

// 时间选择
type PresetKey = 'realtime' | '1h' | 'shift' | 'today' | 'yesterday' | 'week' | 'custom'
const presets: { key: PresetKey; label: string; tooltip: string }[] = [
  { key: 'realtime', label: '实时', tooltip: '最近5分钟' },
  { key: '1h', label: '1H', tooltip: '最近1小时' },
  { key: 'shift', label: '班次', tooltip: '当前班次（8小时）' },
  { key: 'today', label: '今日', tooltip: '今日 00:00 至今' },
  { key: 'yesterday', label: '昨日', tooltip: '昨日全天' },
  { key: 'week', label: '本周', tooltip: '本周一至今' },
]
const selectedPreset = ref<PresetKey>('today')
const customRange = ref<[number, number] | null>(null)
const showCustomPicker = ref(false)

// 计算属性
const loading = computed(() => dataStore.loading)
const hasData = computed(() => dataStore.records.length > 0 || dataStore.chartSeries.length > 0)
const allTags = computed(() => {
  // 获取所有图表中的标签（去重）
  const tags = charts.value.flatMap(c => c.tags)
  return [...new Set(tags)]
})

// 缓存状态
const cacheHit = computed(() => dataStore.cacheHit)
const queryTimeMs = computed(() => dataStore.queryTimeMs)
const totalProcessed = computed(() => dataStore.totalProcessed)

// 检测是否有变更
watch([groupName, charts, () => processingConfig], () => {
  hasChanges.value = 
    groupName.value !== originalName.value ||
    JSON.stringify(charts.value) !== JSON.stringify(originalCharts.value) ||
    JSON.stringify(processingConfig) !== JSON.stringify(originalProcessingConfig.value)
}, { deep: true })

// 初始化
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
    // 深拷贝 charts
    charts.value = JSON.parse(JSON.stringify(group.charts || []))
    originalName.value = group.name
    originalCharts.value = JSON.parse(JSON.stringify(group.charts || []))

    // 加载处理配置
    if (group.processingConfig) {
      Object.assign(processingConfig, JSON.parse(JSON.stringify(group.processingConfig)))
      originalProcessingConfig.value = JSON.parse(JSON.stringify(group.processingConfig))
    } else {
      Object.assign(processingConfig, createDefaultProcessingConfig())
      originalProcessingConfig.value = createDefaultProcessingConfig()
    }

    hasChanges.value = false

    // 异步预热分组缓存（1天数据），不阻塞用户操作
    invoke('warmup_group', { groupId: props.groupId }).catch((e) => {
      console.debug('[GroupEditView] 分组预热失败:', e)
    })

    // 设置标签并查询数据
    if (allTags.value.length > 0) {
      dataStore.setSelectedTags(allTags.value)
      const [start, end] = getPresetRange('today')
      dataStore.setTimeRange(start, end)
      // 使用 V2 接口获取预分组数据
      dataStore.fetchDataV2(processingConfig, false)
    }
  }
}

// 时间范围计算
function getPresetRange(preset: PresetKey): [Date, Date] {
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
      const diff = dayOfWeek === 0 ? 6 : dayOfWeek - 1
      start.setDate(now.getDate() - diff)
      start.setHours(0, 0, 0, 0)
      break
    default:
      start.setHours(0, 0, 0, 0)
  }
  
  return [start, now]
}

function handlePresetClick(preset: PresetKey) {
  selectedPreset.value = preset
  showCustomPicker.value = false
  const [start, end] = getPresetRange(preset)
  dataStore.setTimeRange(start, end)
}

function handleCustomClick() {
  showCustomPicker.value = !showCustomPicker.value
  selectedPreset.value = 'custom'
}

function handleCustomRangeChange(range: [number, number] | null) {
  if (range) {
    customRange.value = range
    dataStore.setTimeRange(new Date(range[0]), new Date(range[1]))
  }
}

function handleQuery(forceRefresh = false) {
  if (allTags.value.length === 0) {
    message.warning('请先添加标签')
    return
  }
  dataStore.setSelectedTags(allTags.value)
  // 使用 V2 接口获取预分组数据
  dataStore.fetchDataV2(processingConfig, forceRefresh)
}

function handleForceRefresh() {
  handleQuery(true)
}

// === 图表操作 ===

// 添加新图表
function handleAddChart() {
  if (!canAddChart.value) {
    message.warning('每个分组最多包含 10 个图表')
    return
  }
  const newChart = createDefaultChartConfig(`图表 ${charts.value.length + 1}`)
  charts.value.push(newChart)
}

// 删除图表
function handleDeleteChart(chartId: string) {
  charts.value = charts.value.filter(c => c.id !== chartId)
}

// 更新图表名称
function handleUpdateChartName(chartId: string, name: string) {
  const chart = charts.value.find(c => c.id === chartId)
  if (chart) {
    chart.name = name
  }
}

// 打开标签选择弹窗
function handleOpenTagModal(chartId: string) {
  activeChartId.value = chartId
  showTagModal.value = true
}

// 添加标签到指定图表
function handleAddTag(tag: string) {
  if (!activeChartId.value) return
  
  const chart = charts.value.find(c => c.id === activeChartId.value)
  if (!chart) return
  
  if (chart.tags.length >= 5) {
    message.warning('每个图表最多包含 5 个标签')
    return
  }
  
  if (!chart.tags.includes(tag)) {
    chart.tags.push(tag)
  }
}

// 从图表移除标签
function handleRemoveTag(chartId: string, tagName: string) {
  const chart = charts.value.find(c => c.id === chartId)
  if (chart) {
    chart.tags = chart.tags.filter(t => t !== tagName)
  }
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
      charts.value,
      processingConfig
    )
    
    if (result) {
      message.success('分组已保存')
      originalName.value = result.name
      originalCharts.value = JSON.parse(JSON.stringify(result.charts))
      originalProcessingConfig.value = JSON.parse(JSON.stringify(processingConfig))
      hasChanges.value = false
      emit('saved', result)
    } else if (tagGroupStore.error) {
      message.error(tagGroupStore.error)
    }
  } finally {
    saving.value = false
  }
}

// 删除分组
async function handleDelete() {
  const success = await tagGroupStore.deleteGroup(props.groupId)
  if (success) {
    message.success('分组已删除')
    emit('deleted')
    emit('back')
  } else if (tagGroupStore.error) {
    message.error(tagGroupStore.error)
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
        
        <NInput
          v-model:value="groupName"
          placeholder="输入分组名称"
          :maxlength="50"
          class="name-input"
        />
      </div>
      
      <div class="header-right">
        <NPopconfirm @positive-click="handleDelete">
          <template #trigger>
            <NButton tertiary type="error">
              <template #icon>
                <NIcon :component="TrashOutline" />
              </template>
              删除
            </NButton>
          </template>
          确定删除此分组吗？
        </NPopconfirm>
        
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
    
    <!-- 时间选择工具栏 -->
    <div class="time-toolbar glass">
      <NSpace align="center" :size="12">
        <NIcon :component="TimeOutline" :size="18" class="toolbar-icon" />
        
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
        
        <NDatePicker
          v-if="showCustomPicker"
          v-model:value="customRange"
          type="datetimerange"
          size="small"
          clearable
          @update:value="handleCustomRangeChange"
        />
      </NSpace>
      
      <NDivider vertical style="height: 24px; margin: 0 8px" />
      
      <NSpace align="center" :size="8">
        <NButton 
          type="primary" 
          size="small"
          :loading="loading"
          :disabled="allTags.length === 0"
          @click="() => handleQuery(false)"
        >
          <template #icon>
            <NIcon :component="PlayOutline" />
          </template>
          查询
        </NButton>
        
        <NTooltip>
          <template #trigger>
            <NButton 
              size="small" 
              tertiary
              :loading="loading"
              :disabled="allTags.length === 0"
              @click="handleForceRefresh"
            >
              <template #icon>
                <NIcon :component="FlashOutline" />
              </template>
            </NButton>
          </template>
          强制刷新（绕过缓存）
        </NTooltip>
      </NSpace>
      
      <!-- 查询状态指示器 -->
      <div v-if="hasData && !loading" class="query-status">
        <NTooltip>
          <template #trigger>
            <NTag :type="cacheHit ? 'success' : 'info'" size="small" round>
              <template #icon>
                <NIcon :component="cacheHit ? CheckmarkCircleOutline : FlashOutline" :size="14" />
              </template>
              {{ cacheHit ? '缓存' : '实时' }}
            </NTag>
          </template>
          {{ cacheHit ? '数据来自缓存' : '数据来自数据库' }}
        </NTooltip>
        
        <span class="query-time">
          {{ totalProcessed.toLocaleString() }} 条 · {{ queryTimeMs }}ms
        </span>
      </div>
    </div>
    
    <!-- 图表管理区域 -->
    <div class="charts-section">
      <div class="charts-header">
        <div class="charts-title">
          <span>图表配置</span>
          <span class="chart-count" :class="{ warning: !canAddChart }">
            {{ charts.length }}/10
          </span>
        </div>
      </div>
      
      <div class="charts-grid">
        <ChartCard
          v-for="(chart, index) in charts"
          :key="chart.id"
          :chart="chart"
          :index="index"
          :can-delete="charts.length > 1"
          @update:name="(name) => handleUpdateChartName(chart.id, name)"
          @delete="handleDeleteChart(chart.id)"
          @add-tag="handleOpenTagModal(chart.id)"
          @remove-tag="(tag) => handleRemoveTag(chart.id, tag)"
        />
        
        <!-- 添加图表卡片 -->
        <div 
          v-if="canAddChart" 
          class="add-chart-btn"
          @click="handleAddChart"
        >
          <div class="add-chart-btn-icon">
            <NIcon :component="AddOutline" size="24" />
          </div>
          <span class="add-chart-btn-text">添加图表</span>
        </div>
      </div>
    </div>
    
    <!-- 数据处理配置面板 -->
    <NCard class="processing-panel glass" :bordered="false" size="small">
      <template #header>
        <span class="panel-title">📊 数据处理</span>
      </template>
      
      <div class="processing-options">
        <!-- 异常值剔除 -->
        <div class="option-item">
          <NCheckbox v-model:checked="processingConfig.outlierRemoval.enabled">
            异常值剔除
          </NCheckbox>
          <span class="option-hint">(3σ法则)</span>
        </div>
        
        <!-- 重采样 -->
        <div class="option-item">
          <NCheckbox v-model:checked="processingConfig.resample.enabled">
            重采样
          </NCheckbox>
          <NInputNumber
            v-model:value="processingConfig.resample.interval"
            :disabled="!processingConfig.resample.enabled"
            :min="1"
            :max="3600"
            size="small"
            style="width: 100px"
          />
          <span class="option-unit">秒</span>
          <span class="option-hint">(均值聚合)</span>
        </div>
        
        <!-- 平滑滤波 -->
        <div class="option-item">
          <NCheckbox v-model:checked="processingConfig.smoothing.enabled">
            平滑滤波
          </NCheckbox>
          <span class="option-label">窗口:</span>
          <NInputNumber
            v-model:value="processingConfig.smoothing.window"
            :disabled="!processingConfig.smoothing.enabled"
            :min="2"
            :max="50"
            size="small"
            style="width: 80px"
          />
          <span class="option-hint">(移动平均)</span>
        </div>
      </div>
    </NCard>
    
    <!-- 图表区域 -->
    <div class="chart-container glass-card">
      <LineChart v-if="hasData" :use-v2="true" />
      <div v-else class="empty-chart">
        <div class="empty-content">
          <NIcon :component="TimeOutline" :size="48" class="empty-icon" />
          <p v-if="allTags.length === 0">请添加标签后查询数据</p>
          <p v-else>选择时间范围并点击查询</p>
        </div>
      </div>
    </div>
    
    <!-- 标签搜索弹窗 -->
    <TagSearchModal
      v-model:show="showTagModal"
      :selected-tags="activeChartId ? (charts.find(c => c.id === activeChartId)?.tags || []) : []"
      :max-tags="5"
      @add="handleAddTag"
    />
  </div>
</template>

<style scoped>
/* ===== 赛博朋克分组编辑视图 ===== */
.group-edit-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 20px;
  gap: 16px;
  background: var(--bg-base);
  position: relative;
  animation: cyber-fade-in 0.5s var(--ease-cyber) forwards;
}

/* 网格背景 */
.group-edit-view::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image:
    linear-gradient(rgba(0, 245, 255, 0.02) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 245, 255, 0.02) 1px, transparent 1px);
  background-size: 40px 40px;
  pointer-events: none;
  z-index: 0;
}

/* 编辑头部 */
.edit-header {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-radius: var(--radius-xl);
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-default);
}

/* 头部顶部霓虹条 */
.edit-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--neon-cyan), var(--neon-magenta));
  opacity: 0.7;
  border-radius: var(--radius-xl) var(--radius-xl) 0 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
}

.name-input {
  max-width: 320px;
  font-family: var(--font-display);
  font-size: var(--text-lg);
  font-weight: var(--font-semibold);
}

.name-input :deep(.n-input__input-el) {
  font-family: var(--font-display);
  font-size: var(--text-lg);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 14px;
}

/* 时间工具栏 */
.time-toolbar {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-radius: var(--radius-xl);
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  border: 1px solid var(--border-default);
}

.toolbar-icon {
  color: var(--neon-cyan);
  filter: drop-shadow(0 0 4px var(--neon-cyan-glow));
}

/* 图表管理区域 */
.charts-section {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.charts-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 6px;
}

.charts-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-family: var(--font-display);
  font-size: var(--text-base);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
}

/* 标题前的霓虹装饰 */
.charts-title::before {
  content: '//';
  color: var(--neon-cyan);
  font-family: var(--font-mono);
  opacity: 0.6;
}

.chart-count {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
  color: var(--text-muted);
  padding: 4px 12px;
  background: rgba(0, 245, 255, 0.08);
  border: 1px solid rgba(0, 245, 255, 0.15);
  border-radius: var(--radius-sm);
}

.chart-count.warning {
  color: var(--neon-orange);
  background: rgba(255, 136, 0, 0.1);
  border-color: rgba(255, 136, 0, 0.3);
}

.charts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 18px;
}

@media (max-width: 1200px) {
  .charts-grid {
    grid-template-columns: 1fr;
  }
}

/* 添加图表按钮 - 赛博朋克风格 */
.add-chart-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  min-height: 180px;
  background: transparent;
  border: 2px dashed var(--border-default);
  border-radius: var(--radius-xl);
  cursor: pointer;
  transition: all var(--transition-normal);
  color: var(--text-muted);
}

.add-chart-btn:hover {
  background: var(--bg-hover);
  border-color: var(--neon-cyan);
  color: var(--neon-cyan);
  transform: translateY(-3px);
  box-shadow: 0 0 25px var(--neon-cyan-glow);
}

.add-chart-btn-icon {
  width: 52px;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--glass-bg);
  border: 1px solid var(--border-default);
  transition: all var(--transition-normal);
}

.add-chart-btn:hover .add-chart-btn-icon {
  background: var(--neon-cyan);
  border-color: var(--neon-cyan);
  color: var(--bg-base);
  transform: scale(1.1);
  box-shadow: var(--glow-cyan);
}

.add-chart-btn-text {
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wide);
}

/* 数据处理面板 */
.processing-panel {
  position: relative;
  z-index: 1;
  border-radius: var(--radius-xl);
  background: var(--glass-bg);
  border: 1px solid var(--border-default);
}

/* 面板顶部霓虹条 */
.processing-panel::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--neon-magenta), var(--neon-cyan));
  opacity: 0.6;
  border-radius: var(--radius-xl) var(--radius-xl) 0 0;
}

.processing-panel :deep(.n-card-header) {
  padding: 12px 18px;
  background: linear-gradient(
    90deg,
    rgba(255, 0, 255, 0.03) 0%,
    rgba(0, 245, 255, 0.03) 100%
  );
}

.panel-title {
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
}

.processing-options {
  display: flex;
  flex-wrap: wrap;
  gap: 28px;
  align-items: center;
  padding: 4px 0;
}

.option-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.option-label {
  color: var(--text-secondary);
  font-family: var(--font-body);
  font-size: var(--text-sm);
}

.option-unit {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.option-hint {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  opacity: 0.7;
}

/* 图表容器 */
.chart-container {
  position: relative;
  z-index: 1;
  flex: 1;
  min-height: 400px;
  border-radius: var(--radius-xl);
  overflow: hidden;
  background: var(--glass-bg);
  border: 1px solid var(--border-default);
}

/* 图表容器顶部霓虹条 */
.chart-container::before {
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
  z-index: 1;
}

.empty-chart {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 400px;
}

.empty-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
  color: var(--text-muted);
}

.empty-icon {
  opacity: 0.3;
  color: var(--neon-cyan);
}

.empty-content p {
  margin: 0;
  font-family: var(--font-body);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
}

/* 查询状态指示器 */
.query-status {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-left: auto;
  padding-left: 14px;
}

.query-time {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
  color: var(--text-muted);
}

/* 赛博朋克入场动画 */
@keyframes cyber-fade-in {
  from {
    opacity: 0;
    transform: translateY(20px);
    filter: blur(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
    filter: blur(0);
  }
}
</style>
