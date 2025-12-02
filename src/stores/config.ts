import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, ConnectionTestResult, DatabaseConfig } from '@/types'

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

  return {
    // State
    config,
    loading,
    error,
    isConnected,
    lastConnectionMessage,
    // Actions
    loadConfig,
    saveConfig,
    testConnection,
    resetConnectionStatus,
  }
})
