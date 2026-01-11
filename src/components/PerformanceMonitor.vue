<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { NCard, NStatistic, NGrid, NGi, NProgress, NTag, NButton, NSpace, NTooltip, NIcon } from 'naive-ui'
import { Refresh } from '@vicons/ionicons5'
import { useDataStore } from '@/stores/data'
import type { CacheStats, PoolState } from '@/types'

const dataStore = useDataStore()

// 状态
const cacheStats = ref<CacheStats | null>(null)
const poolState = ref<PoolState | null>(null)
const loading = ref(false)
const lastUpdated = ref<Date | null>(null)
let refreshInterval: ReturnType<typeof setInterval> | null = null

// 计算属性
const hitRateStatus = computed(() => {
  if (!cacheStats.value) return 'default'
  const rate = cacheStats.value.hitRate
  if (rate >= 70) return 'success'
  if (rate >= 40) return 'warning'
  return 'error'
})

const hitRateColor = computed(() => {
  const status = hitRateStatus.value
  if (status === 'success') return '#00ff88'
  if (status === 'warning') return '#ffee00'
  return '#ff0055'
})

const memoryUsageFormatted = computed(() => {
  if (!cacheStats.value) return '0 B'
  const bytes = cacheStats.value.estimatedMemoryBytes
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
})

const cacheUsagePercent = computed(() => {
  if (!cacheStats.value) return 0
  return (cacheStats.value.entries / cacheStats.value.maxEntries) * 100
})

const poolUsagePercent = computed(() => {
  if (!poolState.value) return 0
  return (poolState.value.activeConnections / poolState.value.maxSize) * 100
})

const poolStatusColor = computed(() => {
  const percent = poolUsagePercent.value
  if (percent >= 80) return '#ff0055'
  if (percent >= 50) return '#ffee00'
  return '#00ff88'
})

const lastUpdatedFormatted = computed(() => {
  if (!lastUpdated.value) return '从未'
  return lastUpdated.value.toLocaleTimeString('zh-CN')
})

// 方法
const fetchStats = async () => {
  loading.value = true
  try {
    const [cache, pool] = await Promise.all([
      dataStore.getCacheStats(),
      dataStore.getPoolState()
    ])
    cacheStats.value = cache
    poolState.value = pool
    lastUpdated.value = new Date()
  } finally {
    loading.value = false
  }
}

const clearCache = async () => {
  await dataStore.clearCache()
  await fetchStats()
}

// 生命周期
onMounted(() => {
  fetchStats()
  // 每 30 秒自动刷新
  refreshInterval = setInterval(fetchStats, 30000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>

<template>
  <NCard title="性能监控" size="small" class="performance-monitor">
    <template #header-extra>
      <NSpace>
        <NTag size="small" :bordered="false">
          更新: {{ lastUpdatedFormatted }}
        </NTag>
        <NTooltip>
          <template #trigger>
            <NButton
              size="small"
              quaternary
              circle
              :loading="loading"
              @click="fetchStats"
            >
              <template #icon>
                <NIcon><Refresh /></NIcon>
              </template>
            </NButton>
          </template>
          刷新统计
        </NTooltip>
      </NSpace>
    </template>

    <NGrid :cols="3" :x-gap="12" :y-gap="12">
      <!-- 缓存命中率 -->
      <NGi>
        <div class="stat-card">
          <div class="stat-label">缓存命中率</div>
          <div class="stat-value">
            <NProgress
              type="circle"
              :percentage="cacheStats?.hitRate ?? 0"
              :color="hitRateColor"
              :rail-color="'rgba(255,255,255,0.1)'"
              :stroke-width="8"
              :show-indicator="true"
            >
              <span class="progress-text">{{ (cacheStats?.hitRate ?? 0).toFixed(1) }}%</span>
            </NProgress>
          </div>
          <div class="stat-detail">
            命中 {{ cacheStats?.hits ?? 0 }} / 未命中 {{ cacheStats?.misses ?? 0 }}
          </div>
        </div>
      </NGi>

      <!-- 缓存使用量 -->
      <NGi>
        <div class="stat-card">
          <div class="stat-label">缓存使用量</div>
          <div class="stat-value">
            <NStatistic :value="cacheStats?.entries ?? 0">
              <template #suffix>
                <span class="stat-suffix">/ {{ cacheStats?.maxEntries ?? 200 }}</span>
              </template>
            </NStatistic>
          </div>
          <NProgress
            type="line"
            :percentage="cacheUsagePercent"
            :color="'#00f5ff'"
            :rail-color="'rgba(255,255,255,0.1)'"
            :height="6"
            :show-indicator="false"
          />
          <div class="stat-detail">
            内存占用: {{ memoryUsageFormatted }}
          </div>
        </div>
      </NGi>

      <!-- 连接池状态 -->
      <NGi>
        <div class="stat-card">
          <div class="stat-label">连接池状态</div>
          <div class="stat-value">
            <NStatistic :value="poolState?.activeConnections ?? 0">
              <template #suffix>
                <span class="stat-suffix">/ {{ poolState?.maxSize ?? 10 }}</span>
              </template>
            </NStatistic>
          </div>
          <NProgress
            type="line"
            :percentage="poolUsagePercent"
            :color="poolStatusColor"
            :rail-color="'rgba(255,255,255,0.1)'"
            :height="6"
            :show-indicator="false"
          />
          <div class="stat-detail">
            空闲: {{ poolState?.idleConnections ?? 0 }} / 总计: {{ poolState?.connections ?? 0 }}
          </div>
        </div>
      </NGi>

      <!-- 查询性能 -->
      <NGi>
        <div class="stat-card">
          <div class="stat-label">最近查询</div>
          <div class="stat-value">
            <NStatistic :value="dataStore.queryTimeMs">
              <template #suffix>
                <span class="stat-suffix">ms</span>
              </template>
            </NStatistic>
          </div>
          <div class="stat-detail">
            <NTag
              size="small"
              :type="dataStore.cacheHit ? 'success' : 'warning'"
              :bordered="false"
            >
              {{ dataStore.cacheHit ? '缓存命中' : '数据库查询' }}
            </NTag>
          </div>
        </div>
      </NGi>

      <!-- 数据处理 -->
      <NGi>
        <div class="stat-card">
          <div class="stat-label">数据处理</div>
          <div class="stat-value">
            <NStatistic :value="dataStore.totalProcessed">
              <template #suffix>
                <span class="stat-suffix">条</span>
              </template>
            </NStatistic>
          </div>
          <div class="stat-detail">
            原始数据: {{ dataStore.total }} 条
          </div>
        </div>
      </NGi>
    </NGrid>

    <!-- 操作按钮 -->
    <div class="actions">
      <NButton
        size="small"
        type="warning"
        ghost
        @click="clearCache"
      >
        清空缓存
      </NButton>
    </div>
  </NCard>
</template>

<style scoped>
.performance-monitor {
  background: rgba(30, 41, 59, 0.6);
  border: 1px solid rgba(0, 245, 255, 0.15);
}

.stat-card {
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
  border: 1px solid rgba(0, 245, 255, 0.1);
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
  animation: stat-card-in 0.4s cubic-bezier(0.23, 1, 0.32, 1) backwards;
}

.stat-card:hover {
  transform: translateY(-2px);
  border-color: rgba(0, 245, 255, 0.3);
  box-shadow: 0 4px 20px rgba(0, 245, 255, 0.15);
}

/* Staggered animation delays for each card */
.n-gi:nth-child(1) .stat-card { animation-delay: 0.05s; }
.n-gi:nth-child(2) .stat-card { animation-delay: 0.1s; }
.n-gi:nth-child(3) .stat-card { animation-delay: 0.15s; }
.n-gi:nth-child(4) .stat-card { animation-delay: 0.2s; }
.n-gi:nth-child(5) .stat-card { animation-delay: 0.25s; }

@keyframes stat-card-in {
  from {
    opacity: 0;
    transform: translateY(10px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.stat-label {
  font-size: 12px;
  color: #8ba4b4;
  margin-bottom: 8px;
}

.stat-value {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 60px;
}

.stat-suffix {
  font-size: 14px;
  color: #8ba4b4;
  margin-left: 4px;
}

.stat-detail {
  font-size: 11px;
  color: #64748b;
  text-align: center;
  margin-top: 8px;
}

.progress-text {
  font-size: 16px;
  font-weight: 600;
  color: #e2e8f0;
}

.actions {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

:deep(.n-statistic-value) {
  font-size: 24px !important;
  color: #00f5ff !important;
}

:deep(.n-card-header) {
  padding: 12px 16px;
}

:deep(.n-card__content) {
  padding: 12px 16px;
}
</style>
