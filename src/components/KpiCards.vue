<script setup lang="ts">
import { computed } from 'vue'
import { NCard, NStatistic, NIcon, NSkeleton } from 'naive-ui'
import { 
  BarChartOutline,
  TimeOutline, 
  TrendingUpOutline, 
  TrendingDownOutline 
} from '@vicons/ionicons5'
import { useDataStore } from '@/stores/data'

const dataStore = useDataStore()

// 计算统计数据
const stats = computed(() => {
  const records = dataStore.records
  
  if (records.length === 0) {
    return {
      count: 0,
      timeSpan: '-',
      maxVal: null as number | null,
      minVal: null as number | null,
      maxTag: '-',
      minTag: '-',
    }
  }
  
  // 计算极值
  let max = -Infinity
  let min = Infinity
  let maxTag = ''
  let minTag = ''
  
  for (const record of records) {
    if (record.tagVal > max) {
      max = record.tagVal
      maxTag = record.tagName
    }
    if (record.tagVal < min) {
      min = record.tagVal
      minTag = record.tagName
    }
  }
  
  // 计算时间跨度
  const start = dataStore.startTime
  const end = dataStore.endTime
  const diffMs = end.getTime() - start.getTime()
  const diffHours = Math.round(diffMs / (1000 * 60 * 60))
  
  let timeSpan = ''
  if (diffHours < 24) {
    timeSpan = `${diffHours} 小时`
  } else {
    const days = Math.round(diffHours / 24)
    timeSpan = `${days} 天`
  }
  
  return {
    count: records.length,
    timeSpan,
    maxVal: max === -Infinity ? null : max,
    minVal: min === Infinity ? null : min,
    maxTag: maxTag || '-',
    minTag: minTag || '-',
  }
})

const loading = computed(() => dataStore.loading)
</script>

<template>
  <div class="kpi-container">
    <!-- 数据量 -->
    <NCard class="kpi-card glass-card" :bordered="false">
      <div class="kpi-content">
        <div class="kpi-icon data-icon">
          <NIcon :component="BarChartOutline" :size="24" />
        </div>
        <div class="kpi-info">
          <span class="kpi-label">数据量</span>
          <NSkeleton v-if="loading" text style="width: 80px; height: 28px" />
          <NStatistic v-else :value="stats.count" class="kpi-value">
            <template #suffix>
              <span class="kpi-unit">条</span>
            </template>
          </NStatistic>
        </div>
      </div>
    </NCard>
    
    <!-- 时间跨度 -->
    <NCard class="kpi-card glass-card" :bordered="false">
      <div class="kpi-content">
        <div class="kpi-icon time-icon">
          <NIcon :component="TimeOutline" :size="24" />
        </div>
        <div class="kpi-info">
          <span class="kpi-label">时间跨度</span>
          <NSkeleton v-if="loading" text style="width: 80px; height: 28px" />
          <div v-else class="kpi-text">{{ stats.timeSpan }}</div>
        </div>
      </div>
    </NCard>
    
    <!-- 最大值 -->
    <NCard class="kpi-card glass-card" :bordered="false">
      <div class="kpi-content">
        <div class="kpi-icon max-icon">
          <NIcon :component="TrendingUpOutline" :size="24" />
        </div>
        <div class="kpi-info">
          <span class="kpi-label">最大值</span>
          <NSkeleton v-if="loading" text style="width: 80px; height: 28px" />
          <template v-else>
            <NStatistic v-if="stats.maxVal !== null" :value="stats.maxVal" class="kpi-value">
              <template #suffix>
                <span class="kpi-unit"></span>
              </template>
            </NStatistic>
            <div v-else class="kpi-text">-</div>
          </template>
          <span class="kpi-tag" :title="stats.maxTag">{{ stats.maxTag }}</span>
        </div>
      </div>
    </NCard>
    
    <!-- 最小值 -->
    <NCard class="kpi-card glass-card" :bordered="false">
      <div class="kpi-content">
        <div class="kpi-icon min-icon">
          <NIcon :component="TrendingDownOutline" :size="24" />
        </div>
        <div class="kpi-info">
          <span class="kpi-label">最小值</span>
          <NSkeleton v-if="loading" text style="width: 80px; height: 28px" />
          <template v-else>
            <NStatistic v-if="stats.minVal !== null" :value="stats.minVal" class="kpi-value">
              <template #suffix>
                <span class="kpi-unit"></span>
              </template>
            </NStatistic>
            <div v-else class="kpi-text">-</div>
          </template>
          <span class="kpi-tag" :title="stats.minTag">{{ stats.minTag }}</span>
        </div>
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.kpi-container {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

@media (max-width: 1200px) {
  .kpi-container {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 600px) {
  .kpi-container {
    grid-template-columns: 1fr;
  }
}

.kpi-card {
  padding: 4px;
}

.kpi-card :deep(.n-card__content) {
  padding: 16px;
}

.kpi-content {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.kpi-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: 12px;
  flex-shrink: 0;
}

.data-icon {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.15), rgba(59, 130, 246, 0.05));
  color: #3b82f6;
}

.time-icon {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.15), rgba(139, 92, 246, 0.05));
  color: #8b5cf6;
}

.max-icon {
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.15), rgba(34, 197, 94, 0.05));
  color: #22c55e;
}

.min-icon {
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.15), rgba(245, 158, 11, 0.05));
  color: #f59e0b;
}

.kpi-info {
  flex: 1;
  min-width: 0;
}

.kpi-label {
  font-size: 13px;
  color: var(--text-secondary);
  display: block;
  margin-bottom: 4px;
}

.kpi-value {
  font-size: 24px;
  font-weight: 600;
}

.kpi-value :deep(.n-statistic-value__content) {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
}

.kpi-text {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.2;
}

.kpi-unit {
  font-size: 14px;
  color: var(--text-muted);
  margin-left: 4px;
}

.kpi-tag {
  font-size: 12px;
  color: var(--text-muted);
  display: block;
  margin-top: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
