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
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useThemeStore } from '@/stores/theme'
import { useConfigStore } from '@/stores/config'
import { useTagGroupStore } from '@/stores/tagGroup'

// Config change event payload from backend
interface ConfigChangeEvent {
  config_type: string
  success: boolean
  error: string | null
}

const themeStore = useThemeStore()
const configStore = useConfigStore()
const tagGroupStore = useTagGroupStore()

// Event listener cleanup function
let unlistenConfigChange: UnlistenFn | null = null

// Setup config change listener
async function setupConfigChangeListener() {
  unlistenConfigChange = await listen<ConfigChangeEvent>('config-changed', (event) => {
    const { config_type, success, error } = event.payload

    console.log(`[ConfigWatcher] Config changed: ${config_type}, success: ${success}`)

    if (success) {
      // Reload corresponding store data
      if (config_type === 'app') {
        configStore.loadConfig()
        console.log('[ConfigWatcher] App config reloaded')
      } else if (config_type === 'tag_groups') {
        tagGroupStore.loadGroups()
        console.log('[ConfigWatcher] Tag groups reloaded')
      }
    } else {
      console.error(`[ConfigWatcher] Failed to reload ${config_type}:`, error)
    }
  })
}

onMounted(async () => {
  themeStore.init()

  // Setup config hot-reload listener
  try {
    await setupConfigChangeListener()
  } catch (e) {
    console.error('配置热更新监听启动失败:', e)
  }

  // 启动连接状态监控（自动验证 + 每10分钟刷新）
  // 使用 try-catch 确保连接失败不影响应用启动
  try {
    await configStore.startConnectionMonitor()
  } catch (e) {
    console.error('连接监控启动失败:', e)
  }
})

onUnmounted(() => {
  // Cleanup config change listener
  if (unlistenConfigChange) {
    unlistenConfigChange()
    unlistenConfigChange = null
  }
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
            <router-view v-slot="{ Component, route }">
              <transition name="page" mode="out-in">
                <component :is="Component" :key="route.path" />
              </transition>
            </router-view>
          </NNotificationProvider>
        </NDialogProvider>
      </NMessageProvider>
    </NLoadingBarProvider>
  </NConfigProvider>
</template>

<style>
/* Page transition animations */
.page-enter-active,
.page-leave-active {
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
}

.page-enter-from {
  opacity: 0;
  transform: translateX(20px);
  filter: blur(4px);
}

.page-leave-to {
  opacity: 0;
  transform: translateX(-20px);
  filter: blur(4px);
}
</style>
