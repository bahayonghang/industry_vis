<script setup lang="ts">
import MainLayout from '@/layouts/MainLayout.vue'
import ConnectionSettings from '@/components/ConnectionSettings.vue'
</script>

<template>
  <MainLayout>
    <div class="settings-view">
      <!-- 页面标题 -->
      <div class="page-header">
        <div class="page-header-left">
          <h1 class="page-title">系统设置</h1>
          <p class="page-subtitle">配置数据库连接和应用参数</p>
        </div>
      </div>

      <!-- 设置内容区 -->
      <div class="settings-content">
        <ConnectionSettings />
      </div>
    </div>
  </MainLayout>
</template>

<style scoped>
.settings-view {
  padding: var(--space-lg, 24px);
  height: 100%;
  overflow-y: auto;
  animation: fadeIn 0.3s ease-out;
}

.page-header {
  margin-bottom: var(--space-lg, 24px);
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.page-header-left {
  flex: 1;
}

.page-title {
  margin: 0 0 var(--space-sm, 8px) 0;
  font-size: var(--text-4xl, 28px);
  font-weight: var(--font-semibold, 600);
  color: var(--text-primary);
}

.page-subtitle {
  margin: 0;
  font-size: var(--text-base, 14px);
  color: var(--text-muted);
}

.settings-content {
  max-width: 800px;
}

.settings-content :deep(.connection-card) {
  background: var(--card-bg);
  border-radius: var(--radius-xl, 16px);
  box-shadow: var(--card-shadow);
}

.settings-content :deep(.n-card-header) {
  padding: var(--space-md, 16px) var(--space-lg, 24px);
  border-bottom: 1px solid var(--border-default);
}

.settings-content :deep(.n-card__content) {
  padding: var(--space-lg, 24px);
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
