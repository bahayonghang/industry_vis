<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NTabs, NTabPane, NCard, NEmpty } from 'naive-ui'
import MainLayout from '@/layouts/MainLayout.vue'
import DataTable from '@/components/DataTable.vue'
import TagSelector from '@/components/TagSelector.vue'
import LineChart from '@/components/LineChart.vue'
import KpiCards from '@/components/KpiCards.vue'
import QueryToolbar from '@/components/QueryToolbar.vue'
import { useDataStore } from '@/stores/data'

const dataStore = useDataStore()
const activeTab = ref<'chart' | 'table'>('chart')

onMounted(() => {
  // 初始化加载标签
  dataStore.fetchAvailableTags()
})
</script>

<template>
  <MainLayout>
    <div class="home-view">
      <!-- 顶部工具栏 -->
      <div class="toolbar-section">
        <QueryToolbar />
        
        <!-- 标签选择器 -->
        <div class="tag-selector-wrapper glass">
          <TagSelector />
        </div>
      </div>
      
      <!-- KPI 概览卡片 -->
      <KpiCards />
      
      <!-- 主内容区：图表/表格 -->
      <NCard class="main-content glass-card" :bordered="false">
        <NTabs 
          v-model:value="activeTab" 
          type="line" 
          animated
          class="content-tabs"
        >
          <NTabPane name="chart" tab="趋势图表">
            <div class="tab-content">
              <LineChart v-if="dataStore.records.length > 0" />
              <NEmpty v-else description="暂无数据，请选择时间范围和标签后查询" />
            </div>
          </NTabPane>
          <NTabPane name="table" tab="数据表格">
            <div class="tab-content">
              <DataTable v-if="dataStore.records.length > 0" />
              <NEmpty v-else description="暂无数据，请选择时间范围和标签后查询" />
            </div>
          </NTabPane>
        </NTabs>
      </NCard>
    </div>
  </MainLayout>
</template>

<style scoped>
.home-view {
  padding: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: fadeIn var(--transition-normal) ease-out;
}

.toolbar-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tag-selector-wrapper {
  padding: 12px 16px;
  border-radius: var(--radius-lg);
}

.main-content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.main-content :deep(.n-card__content) {
  padding: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.content-tabs {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.content-tabs :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  min-height: 0;
}

.content-tabs :deep(.n-tab-pane) {
  height: 100%;
}

.tab-content {
  height: 100%;
  min-height: 400px;
  display: flex;
  flex-direction: column;
}

@keyframes fadeIn {
  from { 
    opacity: 0; 
    transform: translateY(8px); 
  }
  to { 
    opacity: 1; 
    transform: translateY(0); 
  }
}
</style>
