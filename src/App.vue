<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { 
  NConfigProvider, 
  NMessageProvider, 
  NDialogProvider, 
  NNotificationProvider, 
  NLoadingBarProvider,
  zhCN, 
  dateZhCN 
} from 'naive-ui'
import { useThemeStore } from '@/stores/theme'
import { useConfigStore } from '@/stores/config'

const themeStore = useThemeStore()
const configStore = useConfigStore()

onMounted(() => {
  themeStore.init()
  // 启动连接状态监控（自动验证 + 每10分钟刷新）
  configStore.startConnectionMonitor()
})

onUnmounted(() => {
  // 停止连接监控
  configStore.stopConnectionMonitor()
})
</script>

<template>
  <NConfigProvider 
    :locale="zhCN" 
    :date-locale="dateZhCN"
    :theme="themeStore.naiveTheme"
    :theme-overrides="themeStore.themeOverrides"
  >
    <NLoadingBarProvider>
      <NMessageProvider>
        <NDialogProvider>
          <NNotificationProvider>
            <!-- 渐变背景装饰 -->
            <div class="gradient-mesh"></div>
            <router-view />
          </NNotificationProvider>
        </NDialogProvider>
      </NMessageProvider>
    </NLoadingBarProvider>
  </NConfigProvider>
</template>
