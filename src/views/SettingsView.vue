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
/* ===== 赛博朋克设置页面 ===== */
.settings-view {
  position: relative;
  padding: 28px;
  height: 100%;
  overflow-y: auto;
  animation: cyber-fade-in 0.5s var(--ease-cyber) forwards;
  background: var(--bg-base);
}

/* 网格背景 */
.settings-view::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image:
    linear-gradient(rgba(0, 245, 255, 0.02) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 245, 255, 0.02) 1px, transparent 1px);
  background-size: 40px 40px;
  pointer-events: none;
  z-index: 0;
}

.page-header {
  position: relative;
  z-index: 1;
  margin-bottom: 28px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.page-header-left {
  flex: 1;
}

.page-title {
  margin: 0 0 10px 0;
  font-family: var(--font-display);
  font-size: var(--text-3xl);
  font-weight: var(--font-bold);
  letter-spacing: var(--tracking-wider);
  color: var(--text-primary);
  text-shadow: 0 0 20px var(--neon-cyan-glow);
}

/* 标题前的霓虹装饰 */
.page-title::before {
  content: '// ';
  color: var(--neon-cyan);
  font-family: var(--font-mono);
  opacity: 0.6;
}

.page-subtitle {
  margin: 0;
  font-family: var(--font-body);
  font-size: var(--text-sm);
  letter-spacing: var(--tracking-wide);
  color: var(--text-muted);
}

.settings-content {
  position: relative;
  z-index: 1;
  max-width: 800px;
}

.settings-content :deep(.connection-card) {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur));
  border-radius: var(--radius-xl);
  border: 1px solid var(--border-default);
  box-shadow: var(--glass-shadow);
  overflow: hidden;
}

/* 卡片顶部霓虹条 */
.settings-content :deep(.connection-card)::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--neon-cyan), var(--neon-magenta));
  opacity: 0.8;
}

.settings-content :deep(.n-card-header) {
  padding: 18px 24px;
  border-bottom: 1px solid var(--border-default);
  background: linear-gradient(
    90deg,
    rgba(0, 245, 255, 0.03) 0%,
    rgba(255, 0, 255, 0.02) 100%
  );
}

.settings-content :deep(.n-card-header__main) {
  font-family: var(--font-display);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
}

.settings-content :deep(.n-card__content) {
  padding: 24px;
}

/* 赛博朋克入场动画 */
@keyframes cyber-fade-in {
  from {
    opacity: 0;
    transform: translateY(20px);
    filter: blur(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
    filter: blur(0);
  }
}
</style>
