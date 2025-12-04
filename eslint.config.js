import pluginVue from 'eslint-plugin-vue'
import vueTsEslintConfig from '@vue/eslint-config-typescript'

export default [
  {
    name: 'app/files-to-lint',
    files: ['src/**/*.{ts,mts,tsx,vue}'],
  },
  {
    name: 'app/files-to-ignore',
    ignores: [
      '**/dist/**',
      '**/dist-ssr/**',
      '**/coverage/**',
      '**/node_modules/**',
      'src-tauri/**',
      'docs/**',
      '*.config.*',
    ],
  },
  ...pluginVue.configs['flat/essential'],
  ...vueTsEslintConfig(),
  {
    rules: {
      // 允许 any 类型（开发阶段放宽）
      '@typescript-eslint/no-explicit-any': 'warn',
      // 允许未使用变量以下划线开头
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      // Vue 组件名多词规则放宽
      'vue/multi-word-component-names': 'off',
      // 允许空对象类型
      '@typescript-eslint/no-empty-object-type': 'off',
    },
  },
]
