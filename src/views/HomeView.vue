<script setup lang="ts">
import { ref } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import DataTable from '@/components/DataTable.vue'
import TimeRangeSelector from '@/components/TimeRangeSelector.vue'
import TagSelector from '@/components/TagSelector.vue'
import LineChart from '@/components/LineChart.vue'

const activeTab = ref<'table' | 'chart'>('table')
</script>

<template>
  <MainLayout>
    <div class="home-view">
      <!-- 查询控制区 -->
      <div class="query-controls">
        <TimeRangeSelector />
        <TagSelector />
      </div>
      
      <!-- 视图切换 -->
      <n-tabs v-model:value="activeTab" type="line" animated>
        <n-tab-pane name="table" tab="数据表格">
          <DataTable />
        </n-tab-pane>
        <n-tab-pane name="chart" tab="趋势图表">
          <LineChart />
        </n-tab-pane>
      </n-tabs>
    </div>
  </MainLayout>
</template>

<style scoped>
.home-view {
  padding: 16px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.query-controls {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
</style>
