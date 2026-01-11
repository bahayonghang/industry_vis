<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { NSpin } from 'naive-ui'
// ECharts 按需导入，减少约 700KB 体积
import * as echarts from 'echarts/core'
import { LineChart as EChartsLineChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  ToolboxComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import { useDataStore } from '@/stores/data'
import { useThemeStore } from '@/stores/theme'
import type { ChartSeriesData } from '@/types'

// 注册必要的组件
echarts.use([
  EChartsLineChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  ToolboxComponent,
  CanvasRenderer,
])

const props = defineProps<{
  useV2?: boolean  // 是否使用 V2 接口数据
}>()

const dataStore = useDataStore()
const themeStore = useThemeStore()
const chartRef = ref<HTMLDivElement | null>(null)
let chartInstance: echarts.ECharts | null = null

const loading = computed(() => dataStore.loading)
// V2 状态（由父组件通过 dataStore 直接访问）
// const cacheHit = computed(() => dataStore.cacheHit)
// const queryTimeMs = computed(() => dataStore.queryTimeMs)
// const totalProcessed = computed(() => dataStore.totalProcessed)

// 赛博朋克霓虹配色方案
const colorPalette = [
  '#00f5ff', // 霓虹青
  '#ff00ff', // 霓虹品红
  '#ff8800', // 霓虹橙
  '#00ff88', // 霓虹绿
  '#ffee00', // 霓虹黄
  '#0088ff', // 霓虹蓝
  '#ff0055', // 霓虹红
  '#aa00ff', // 霓虹紫
]

const chartOption = computed(() => {
  const isDark = themeStore.isDark

  // 赛博朋克主题颜色
  const textColor = isDark ? '#8ba4b4' : '#475569'
  const borderColor = isDark ? 'rgba(0, 245, 255, 0.15)' : 'rgba(0, 180, 196, 0.2)'
  const bgColor = 'transparent'
  
  // 根据接口版本选择数据源
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let series: any[]
  
  if (props.useV2 && dataStore.chartSeries.length > 0) {
    // V2 接口：直接使用预分组数据
    series = dataStore.chartSeries.map((s: ChartSeriesData) => ({
      name: s.tagName,
      type: 'line',
      smooth: 0.3,  // 较小的平滑系数，更贴合真实数据
      showSymbol: false,
      symbolSize: 4,
      // 大数据优化配置
      sampling: 'lttb',           // Largest-Triangle-Three-Buckets 降采样
      large: true,                // 启用大数据优化
      largeThreshold: 2000,       // 超过 2000 点启用
      progressive: 5000,          // 渐进渲染
      progressiveThreshold: 3000, // 超过 3000 点启用渐进渲染
      lineStyle: {
        width: 1.5,
      },
      areaStyle: {
        opacity: 0.05,
      },
      emphasis: {
        focus: 'series',
        lineStyle: {
          width: 2.5,
        },
      },
      data: s.data,  // [[timestamp_ms, value], ...]
    }))
  } else {
    // V1 接口：需要分组处理
    const records = dataStore.records
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
    
    series = Object.entries(seriesData).map(([tagName, data]) => ({
      name: tagName,
      type: 'line',
      smooth: 0.3,
      showSymbol: false,
      symbolSize: 4,
      sampling: 'lttb',
      large: true,
      largeThreshold: 2000,
      lineStyle: {
        width: 1.5,
      },
      areaStyle: {
        opacity: 0.05,
      },
      emphasis: {
        focus: 'series',
        lineStyle: {
          width: 2.5,
        },
      },
      data: data.map(d => [d.time, d.value]),
    }))
  }
  
  return {
    color: colorPalette,
    backgroundColor: bgColor,
    tooltip: {
      trigger: 'axis',
      axisPointer: { 
        type: 'cross',
        crossStyle: {
          color: textColor,
        },
      },
      backgroundColor: isDark ? 'rgba(30, 41, 59, 0.9)' : 'rgba(255, 255, 255, 0.95)',
      borderColor: borderColor,
      borderWidth: 1,
      textStyle: {
        color: isDark ? '#e2e8f0' : '#1e293b',
      },
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      formatter: (params: any[]) => {
        if (!params.length) return ''
        const time = new Date(params[0].axisValue).toLocaleString('zh-CN')
        let html = `<div style="font-weight:600;margin-bottom:8px;padding-bottom:6px;border-bottom:1px solid ${borderColor}">${time}</div>`
        for (const p of params) {
          html += `<div style="display:flex;justify-content:space-between;gap:24px;margin:4px 0">
            <span>${p.marker} ${p.seriesName}</span>
            <span style="font-weight:500">${p.value[1].toFixed(4)}</span>
          </div>`
        }
        return html
      },
    },
    legend: {
      type: 'scroll',
      bottom: 8,
      textStyle: {
        color: textColor,
      },
      pageTextStyle: {
        color: textColor,
      },
    },
    grid: {
      left: '3%',
      right: '4%',
      top: '8%',
      bottom: '12%',
      containLabel: true,
    },
    xAxis: {
      type: 'time',
      axisLine: {
        lineStyle: {
          color: borderColor,
        },
      },
      axisLabel: {
        color: textColor,
        formatter: (value: number) => {
          const date = new Date(value)
          const month = (date.getMonth() + 1).toString().padStart(2, '0')
          const day = date.getDate().toString().padStart(2, '0')
          const hours = date.getHours().toString().padStart(2, '0')
          const minutes = date.getMinutes().toString().padStart(2, '0')
          return `${month}-${day}\n${hours}:${minutes}`
        },
        lineHeight: 16,
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: borderColor,
          type: 'dashed',
          opacity: 0.5,
        },
      },
    },
    yAxis: {
      type: 'value',
      scale: true,
      axisLine: {
        show: false,
      },
      axisLabel: {
        color: textColor,
      },
      splitLine: {
        lineStyle: {
          color: borderColor,
          type: 'dashed',
        },
      },
    },
    dataZoom: [
      { 
        type: 'inside', 
        start: 0, 
        end: 100,
        zoomOnMouseWheel: true,
        moveOnMouseMove: true,
      },
    ],
    toolbox: {
      right: 16,
      top: 0,
      feature: {
        dataZoom: {
          yAxisIndex: 'none',
          title: {
            zoom: '区域缩放',
            back: '还原缩放',
          },
        },
        restore: {
          title: '还原',
        },
        saveAsImage: {
          title: '保存图片',
          backgroundColor: isDark ? '#1e293b' : '#ffffff',
        },
      },
      iconStyle: {
        borderColor: textColor,
      },
      emphasis: {
        iconStyle: {
          borderColor: '#3b82f6',
        },
      },
    },
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
    chartInstance.setOption(chartOption.value, { notMerge: false })
  }
}

const resizeChart = () => {
  chartInstance?.resize()
}

// 监听数据变化 - 使用引用监听而非深度监听
watch(
  () => props.useV2 ? dataStore.chartSeries : dataStore.records,
  updateChart,
  { flush: 'post' }  // 使用 post 确保 DOM 更新后再渲染
)

// 监听主题变化，重新渲染图表
watch(() => themeStore.isDark, () => {
  if (chartInstance) {
    chartInstance.dispose()
    initChart()
  }
})

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
    <NSpin :show="loading" description="加载中...">
      <div ref="chartRef" class="chart"></div>
    </NSpin>
  </div>
</template>

<style scoped>
.chart-container {
  width: 100%;
  height: 100%;
  min-height: 400px;
  flex: 1;
}

.chart {
  width: 100%;
  height: 100%;
  min-height: 450px;
}

.chart-container :deep(.n-spin-container) {
  height: 100%;
}

.chart-container :deep(.n-spin-content) {
  height: 100%;
}
</style>
