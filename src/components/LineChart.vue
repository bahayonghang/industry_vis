<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { NSpin } from 'naive-ui'
import * as echarts from 'echarts'
import { useDataStore } from '@/stores/data'

const dataStore = useDataStore()
const chartRef = ref<HTMLDivElement | null>(null)
let chartInstance: echarts.ECharts | null = null

const loading = computed(() => dataStore.loading)

const chartOption = computed(() => {
  const records = dataStore.records
  
  // Group data by tag
  const seriesData: Record<string, { time: string; value: number }[]> = {}
  
  for (const record of records) {
    if (!seriesData[record.tagName]) {
      seriesData[record.tagName] = []
    }
    seriesData[record.tagName].push({
      time: record.dateTime,
      value: record.tagVal,
    })
  }
  
  // Create series for each tag
  const series = Object.entries(seriesData).map(([tagName, data]) => ({
    name: tagName,
    type: 'line',
    smooth: true,
    showSymbol: false,
    data: data.map(d => [d.time, d.value]),
  }))
  
  return {
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'cross' },
      formatter: (params: any[]) => {
        if (!params.length) return ''
        const time = new Date(params[0].axisValue).toLocaleString('zh-CN')
        let html = `<div style="font-weight:bold">${time}</div>`
        for (const p of params) {
          html += `<div>${p.marker} ${p.seriesName}: ${p.value[1].toFixed(4)}</div>`
        }
        return html
      },
    },
    legend: {
      type: 'scroll',
      bottom: 0,
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '15%',
      containLabel: true,
    },
    xAxis: {
      type: 'time',
      axisLabel: {
        formatter: (value: number) => {
          const date = new Date(value)
          return `${date.getHours()}:${date.getMinutes().toString().padStart(2, '0')}`
        },
      },
    },
    yAxis: {
      type: 'value',
      scale: true,
    },
    dataZoom: [
      { type: 'inside', start: 0, end: 100 },
      { type: 'slider', start: 0, end: 100 },
    ],
    series,
  }
})

const initChart = () => {
  if (chartRef.value) {
    chartInstance = echarts.init(chartRef.value)
    chartInstance.setOption(chartOption.value)
  }
}

const updateChart = () => {
  if (chartInstance) {
    chartInstance.setOption(chartOption.value)
  }
}

const resizeChart = () => {
  chartInstance?.resize()
}

watch(() => dataStore.records, updateChart, { deep: true })

onMounted(() => {
  initChart()
  window.addEventListener('resize', resizeChart)
})

onUnmounted(() => {
  window.removeEventListener('resize', resizeChart)
  chartInstance?.dispose()
})
</script>

<template>
  <div class="chart-container">
    <NSpin :show="loading">
      <div ref="chartRef" class="chart"></div>
    </NSpin>
  </div>
</template>

<style scoped>
.chart-container {
  width: 100%;
  height: 100%;
  min-height: 400px;
}

.chart {
  width: 100%;
  height: 500px;
}
</style>
