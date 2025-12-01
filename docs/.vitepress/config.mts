import { defineConfig } from 'vitepress'

export default defineConfig({
  vite: {
    server: {
      port: 5174,
    },
  },
  title: 'Industry Vis',
  description: '工业数据查看系统文档 | Industrial Data Viewer Documentation',
  
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      themeConfig: {
        nav: [
          { text: '首页', link: '/' },
          { text: '指南', link: '/guide/' },
          { text: 'API', link: '/api/' },
        ],
        sidebar: {
          '/guide/': [
            {
              text: '入门',
              items: [
                { text: '简介', link: '/guide/' },
                { text: '快速开始', link: '/guide/getting-started' },
                { text: '配置', link: '/guide/configuration' },
              ],
            },
            {
              text: '功能',
              items: [
                { text: '数据查询', link: '/guide/data-query' },
                { text: '数据可视化', link: '/guide/visualization' },
                { text: '数据导出', link: '/guide/export' },
              ],
            },
          ],
          '/api/': [
            {
              text: 'API 参考',
              items: [
                { text: '概览', link: '/api/' },
                { text: 'Tauri 命令', link: '/api/commands' },
                { text: '数据模型', link: '/api/models' },
              ],
            },
          ],
        },
        outline: {
          label: '页面导航',
        },
        docFooter: {
          prev: '上一页',
          next: '下一页',
        },
        lastUpdated: {
          text: '最后更新于',
        },
        footer: {
          message: '基于 MIT 许可发布',
        },
      },
    },
    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/en/' },
          { text: 'Guide', link: '/en/guide/' },
          { text: 'API', link: '/en/api/' },
        ],
        sidebar: {
          '/en/guide/': [
            {
              text: 'Getting Started',
              items: [
                { text: 'Introduction', link: '/en/guide/' },
                { text: 'Quick Start', link: '/en/guide/getting-started' },
                { text: 'Configuration', link: '/en/guide/configuration' },
              ],
            },
            {
              text: 'Features',
              items: [
                { text: 'Data Query', link: '/en/guide/data-query' },
                { text: 'Visualization', link: '/en/guide/visualization' },
                { text: 'Export', link: '/en/guide/export' },
              ],
            },
          ],
          '/en/api/': [
            {
              text: 'API Reference',
              items: [
                { text: 'Overview', link: '/en/api/' },
                { text: 'Tauri Commands', link: '/en/api/commands' },
                { text: 'Data Models', link: '/en/api/models' },
              ],
            },
          ],
        },
        outline: {
          label: 'On this page',
        },
        footer: {
          message: 'Released under the MIT License',
        },
      },
    },
  },

  themeConfig: {
    logo: '/logo.svg',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/your-repo/industry-vis' },
    ],
    search: {
      provider: 'local',
    },
  },
})
