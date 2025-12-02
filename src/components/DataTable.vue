<script setup lang="ts">
import { computed, h } from 'vue'
import { NDataTable, NTag, NText } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useDataStore } from '@/stores/data'
import type { HistoryRecord } from '@/types'

const dataStore = useDataStore()

const columns = computed<DataTableColumns<HistoryRecord>>(() => [
  {
    title: '时间',
    key: 'dateTime',
    width: 180,
    fixed: 'left',
    sorter: (a, b) => new Date(a.dateTime).getTime() - new Date(b.dateTime).getTime(),
    render: (row) => h(
      NText,
      { depth: 2, style: 'font-variant-numeric: tabular-nums' },
      { default: () => new Date(row.dateTime).toLocaleString('zh-CN') }
    ),
  },
  {
    title: '标签名',
    key: 'tagName',
    width: 220,
    ellipsis: {
      tooltip: true,
    },
    filterOptions: dataStore.availableTags.map(t => ({ label: t, value: t })),
    filter: (value, row) => row.tagName === value,
    render: (row) => h(
      NText,
      { strong: true },
      { default: () => row.tagName }
    ),
  },
  {
    title: '数值',
    key: 'tagVal',
    width: 140,
    align: 'right',
    sorter: (a, b) => a.tagVal - b.tagVal,
    render: (row) => h(
      NText,
      { 
        code: true,
        style: 'font-variant-numeric: tabular-nums; font-size: 13px'
      },
      { default: () => row.tagVal.toFixed(4) }
    ),
  },
  {
    title: '质量',
    key: 'tagQuality',
    width: 100,
    align: 'center',
    render: (row) => {
      const isGood = row.tagQuality !== '未连接'
      return h(
        NTag,
        { 
          type: isGood ? 'success' : 'warning', 
          size: 'small',
          round: true,
          bordered: false,
        },
        { default: () => row.tagQuality || '正常' }
      )
    },
  },
])

const tableData = computed(() => dataStore.records)
const loading = computed(() => dataStore.loading)

const rowKey = (row: HistoryRecord) => `${row.dateTime}-${row.tagName}`
</script>

<template>
  <div class="data-table-container">
    <NDataTable
      :columns="columns"
      :data="tableData"
      :loading="loading"
      :row-key="rowKey"
      :pagination="{
        pageSize: 100,
        showSizePicker: true,
        pageSizes: [50, 100, 200, 500],
        showQuickJumper: true,
        prefix: ({ itemCount }) => `共 ${itemCount} 条`
      }"
      :max-height="'calc(100vh - 420px)'"
      :min-height="300"
      virtual-scroll
      striped
      flex-height
      class="data-table"
    />
  </div>
</template>

<style scoped>
.data-table-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  flex: 1;
}

.data-table {
  flex: 1;
}

.data-table :deep(.n-data-table-wrapper) {
  border-radius: var(--radius-md);
}

.data-table :deep(.n-data-table-th) {
  font-weight: 600;
}

.data-table :deep(.n-data-table-td) {
  padding: 10px 12px;
}
</style>
