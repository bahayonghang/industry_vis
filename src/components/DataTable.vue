<script setup lang="ts">
import { computed, h } from 'vue'
import { NDataTable, NButton, NSpace, NTag } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useDataStore } from '@/stores/data'
import type { HistoryRecord } from '@/types'

const dataStore = useDataStore()

const columns: DataTableColumns<HistoryRecord> = [
  {
    title: '时间',
    key: 'dateTime',
    width: 180,
    sorter: (a, b) => new Date(a.dateTime).getTime() - new Date(b.dateTime).getTime(),
    render: (row) => new Date(row.dateTime).toLocaleString('zh-CN'),
  },
  {
    title: '标签名',
    key: 'tagName',
    width: 200,
    filterOptions: dataStore.availableTags.map(t => ({ label: t, value: t })),
    filter: (value, row) => row.tagName === value,
  },
  {
    title: '数值',
    key: 'tagVal',
    width: 120,
    sorter: (a, b) => a.tagVal - b.tagVal,
    render: (row) => row.tagVal.toFixed(4),
  },
  {
    title: '质量',
    key: 'tagQuality',
    width: 100,
    render: (row) => h(
      NTag,
      { type: row.tagQuality === '未连接' ? 'warning' : 'success', size: 'small' },
      { default: () => row.tagQuality }
    ),
  },
]

const tableData = computed(() => dataStore.records)
const loading = computed(() => dataStore.loading)

const handleExport = () => {
  dataStore.exportToCsv()
}

const handleRefresh = () => {
  dataStore.fetchData()
}
</script>

<template>
  <div class="data-table-container">
    <NSpace class="toolbar" justify="end">
      <NButton @click="handleRefresh" :loading="loading">
        刷新
      </NButton>
      <NButton type="primary" @click="handleExport" :disabled="tableData.length === 0">
        导出 CSV
      </NButton>
    </NSpace>
    
    <NDataTable
      :columns="columns"
      :data="tableData"
      :loading="loading"
      :pagination="{ pageSize: 50 }"
      :max-height="500"
      virtual-scroll
      striped
    />
  </div>
</template>

<style scoped>
.data-table-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.toolbar {
  padding: 8px 0;
}
</style>
