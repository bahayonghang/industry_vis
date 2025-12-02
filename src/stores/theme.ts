import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { darkTheme, type GlobalTheme } from 'naive-ui'

export type ThemeMode = 'light' | 'dark' | 'system'

export const useThemeStore = defineStore('theme', () => {
  // State
  const mode = ref<ThemeMode>('system')
  const systemPrefersDark = ref(
    window.matchMedia('(prefers-color-scheme: dark)').matches
  )

  // 监听系统主题变化
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  mediaQuery.addEventListener('change', (e) => {
    systemPrefersDark.value = e.matches
  })

  // 计算当前是否为暗色模式
  const isDark = computed(() => {
    if (mode.value === 'system') {
      return systemPrefersDark.value
    }
    return mode.value === 'dark'
  })

  // Naive UI 主题
  const naiveTheme = computed<GlobalTheme | null>(() => {
    return isDark.value ? darkTheme : null
  })

  // 主题覆盖配置（自定义颜色）
  const themeOverrides = computed(() => ({
    common: {
      // 主色调 - 科技蓝
      primaryColor: '#3b82f6',
      primaryColorHover: '#60a5fa',
      primaryColorPressed: '#2563eb',
      primaryColorSuppl: '#3b82f6',
      // 成功色
      successColor: '#22c55e',
      successColorHover: '#4ade80',
      successColorPressed: '#16a34a',
      // 警告色
      warningColor: '#f59e0b',
      warningColorHover: '#fbbf24',
      warningColorPressed: '#d97706',
      // 错误色
      errorColor: '#ef4444',
      errorColorHover: '#f87171',
      errorColorPressed: '#dc2626',
      // 信息色
      infoColor: '#8b5cf6',
      infoColorHover: '#a78bfa',
      infoColorPressed: '#7c3aed',
      // 圆角
      borderRadius: '8px',
      borderRadiusSmall: '6px',
      // 字体
      fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    },
    Card: {
      borderRadius: '12px',
    },
    Button: {
      borderRadiusMedium: '8px',
    },
    Input: {
      borderRadius: '8px',
    },
    Select: {
      borderRadius: '8px',
    },
  }))

  // Actions
  const setMode = (newMode: ThemeMode) => {
    mode.value = newMode
    localStorage.setItem('theme-mode', newMode)
    updateDocumentClass()
  }

  const toggleTheme = () => {
    if (mode.value === 'light') {
      setMode('dark')
    } else if (mode.value === 'dark') {
      setMode('system')
    } else {
      setMode('light')
    }
  }

  const updateDocumentClass = () => {
    const root = document.documentElement
    if (isDark.value) {
      root.classList.add('dark')
      root.classList.remove('light')
    } else {
      root.classList.add('light')
      root.classList.remove('dark')
    }
  }

  // 初始化
  const init = () => {
    const saved = localStorage.getItem('theme-mode') as ThemeMode | null
    if (saved && ['light', 'dark', 'system'].includes(saved)) {
      mode.value = saved
    }
    updateDocumentClass()
  }

  // 监听 isDark 变化，更新 DOM
  watch(isDark, () => {
    updateDocumentClass()
  })

  return {
    // State
    mode,
    isDark,
    naiveTheme,
    themeOverrides,
    // Actions
    setMode,
    toggleTheme,
    init,
  }
})
