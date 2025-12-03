import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, ConnectionTestResult, DatabaseConfig } from '@/types'

// 连接监控间隔（毫秒），默认10分钟
const CONNECTION_CHECK_INTERVAL = 10 * 60 * 1000

interface ConfigFormValue {
  server: string
  port: number
  database: string
  username: string
  password: string
  defaultTable: string
}

export const useConfigStore = defineStore('config', () => {
  // State
  const config = ref<AppConfig | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const isConnected = ref(false)
  const lastConnectionMessage = ref<string>('')
  const isChecking = ref(false) // 是否正在检查连接
  const lastCheckTime = ref<Date | null>(null) // 上次检查时间
  
  // 定时器引用
  let connectionCheckTimer: ReturnType<typeof setInterval> | null = null

  // Actions
  const loadConfig = async (): Promise<ConfigFormValue | null> => {
    loading.value = true
    error.value = null
    
    try {
      const result = await invoke<AppConfig>('load_config')
      config.value = result
      return {
        server: result.database.server,
        port: result.database.port,
        database: result.database.database,
        username: result.database.username,
        password: result.database.password,
        defaultTable: result.query.defaultTable,
      }
    } catch (e) {
      console.error('Failed to load config:', e)
      error.value = String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  const saveConfig = async (formValue: ConfigFormValue): Promise<boolean> => {
    loading.value = true
    error.value = null
    
    try {
      const appConfig: AppConfig = {
        database: {
          server: formValue.server,
          port: formValue.port,
          database: formValue.database,
          username: formValue.username,
          password: formValue.password,
        },
        query: {
          defaultTable: formValue.defaultTable,
        },
      }
      
      await invoke('save_config', { config: appConfig })
      config.value = appConfig
      return true
    } catch (e) {
      console.error('Failed to save config:', e)
      error.value = String(e)
      return false
    } finally {
      loading.value = false
    }
  }

  const testConnection = async (formValue: ConfigFormValue): Promise<ConnectionTestResult> => {
    try {
      const dbConfig: DatabaseConfig = {
        server: formValue.server,
        port: formValue.port,
        database: formValue.database,
        username: formValue.username,
        password: formValue.password,
      }
      
      const result = await invoke<ConnectionTestResult>('test_connection', { config: dbConfig })
      // 更新连接状态
      isConnected.value = result.success
      lastConnectionMessage.value = result.message
      return result
    } catch (e) {
      isConnected.value = false
      lastConnectionMessage.value = String(e)
      return { success: false, message: String(e) }
    }
  }

  // 重置连接状态（当配置改变时）
  const resetConnectionStatus = () => {
    isConnected.value = false
    lastConnectionMessage.value = ''
  }

  // 使用已保存配置自动测试连接（后台静默检测）
  const autoTestConnection = async (): Promise<void> => {
    // 如果正在检查或没有配置，直接返回
    if (isChecking.value || !config.value?.database) {
      return
    }
    
    isChecking.value = true
    
    try {
      const dbConfig = config.value.database
      const result = await invoke<ConnectionTestResult>('test_connection', { config: dbConfig })
      isConnected.value = result.success
      lastConnectionMessage.value = result.message
      lastCheckTime.value = new Date()
    } catch (e) {
      isConnected.value = false
      lastConnectionMessage.value = String(e)
    } finally {
      isChecking.value = false
    }
  }

  // 启动连接监控（应用启动时调用）
  const startConnectionMonitor = async (): Promise<void> => {
    // 先加载配置
    await loadConfig()
    
    // 立即进行一次连接检测
    await autoTestConnection()
    
    // 清除可能存在的旧定时器
    stopConnectionMonitor()
    
    // 设置定时检测
    connectionCheckTimer = setInterval(() => {
      autoTestConnection()
    }, CONNECTION_CHECK_INTERVAL)
  }

  // 停止连接监控
  const stopConnectionMonitor = (): void => {
    if (connectionCheckTimer) {
      clearInterval(connectionCheckTimer)
      connectionCheckTimer = null
    }
  }

  return {
    // State
    config,
    loading,
    error,
    isConnected,
    lastConnectionMessage,
    isChecking,
    lastCheckTime,
    // Actions
    loadConfig,
    saveConfig,
    testConnection,
    resetConnectionStatus,
    autoTestConnection,
    startConnectionMonitor,
    stopConnectionMonitor,
  }
})
