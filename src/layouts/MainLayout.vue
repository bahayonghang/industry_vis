<script setup lang="ts">
import { ref, computed, h, provide, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { 
  NLayout, 
  NLayoutHeader, 
  NLayoutContent,
  NLayoutSider,
  NMenu,
  NIcon,
  NSpace,
  NButton,
  NTooltip,
  NDivider,
  NDropdown,
  NModal,
  NAlert
} from 'naive-ui'
import { 
  HomeOutline, 
  SettingsOutline,
  ServerOutline,
  SunnyOutline,
  MoonOutline,
  DesktopOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
  EllipsisVertical,
  InformationCircleOutline,
  WarningOutline
} from '@vicons/ionicons5'
import type { MenuOption, DropdownOption } from 'naive-ui'
import { useThemeStore } from '@/stores/theme'
import { useConfigStore } from '@/stores/config'
import TagGroupTree from '@/components/TagGroupTree.vue'
import GroupEditView from '@/components/GroupEditView.vue'
import { getVersion } from '@tauri-apps/api/app'

const router = useRouter()
const route = useRoute()
const themeStore = useThemeStore()
const configStore = useConfigStore()

// 侧边栏折叠状态
const collapsed = ref(false)

// 应用版本号
const appVersion = ref('')
onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '0.0.0'
  }
})

// 编辑分组状态
const editingGroupId = ref<string | null>(null)

// 进入编辑模式
function handleEditGroup(groupId: string) {
  editingGroupId.value = groupId
}

// 退出编辑模式
function handleBackFromEdit() {
  editingGroupId.value = null
}

// 分组保存后
function handleGroupSaved() {
  // 保存后自动退出编辑模式（GroupEditView 会触发 back）
}

// 提供给子组件
provide('editingGroupId', editingGroupId)

// 主题图标
const themeIcon = computed(() => {
  if (themeStore.mode === 'light') return SunnyOutline
  if (themeStore.mode === 'dark') return MoonOutline
  return DesktopOutline
})

// 菜单选项
const menuOptions: MenuOption[] = [
  {
    label: '数据监控',
    key: '/',
    icon: () => h(NIcon, null, { default: () => h(HomeOutline) }),
  },
  {
    label: '系统设置',
    key: '/settings',
    icon: () => h(NIcon, null, { default: () => h(SettingsOutline) }),
  },
]

const activeKey = computed(() => route.path)

const handleMenuSelect = (key: string) => {
  router.push(key)
}

// 主题切换下拉菜单
const themeOptions: DropdownOption[] = [
  { label: '浅色模式', key: 'light', icon: () => h(NIcon, null, { default: () => h(SunnyOutline) }) },
  { label: '深色模式', key: 'dark', icon: () => h(NIcon, null, { default: () => h(MoonOutline) }) },
  { label: '跟随系统', key: 'system', icon: () => h(NIcon, null, { default: () => h(DesktopOutline) }) },
]

const handleThemeSelect = (key: string) => {
  themeStore.setMode(key as 'light' | 'dark' | 'system')
}

// 关于弹窗
const showAboutModal = ref(false)

// 更多操作下拉菜单
const moreOptions: DropdownOption[] = [
  { label: '关于', key: 'about', icon: () => h(NIcon, null, { default: () => h(InformationCircleOutline) }) },
]

const handleMoreSelect = (key: string) => {
  if (key === 'about') {
    showAboutModal.value = true
  }
}
</script>

<template>
  <NLayout class="main-layout" has-sider>
    <!-- 侧边栏 -->
    <NLayoutSider
      bordered
      collapse-mode="width"
      :collapsed-width="64"
      :width="240"
      :collapsed="collapsed"
      show-trigger
      @collapse="collapsed = true"
      @expand="collapsed = false"
      class="sidebar glass-subtle"
      :native-scrollbar="false"
    >
      <!-- Logo 区域 -->
      <div class="logo-area">
        <div class="logo-icon">
          <NIcon size="28" :color="themeStore.isDark ? '#60a5fa' : '#3b82f6'">
            <ServerOutline />
          </NIcon>
        </div>
        <Transition name="fade">
          <span v-if="!collapsed" class="logo-text">工业数据可视化</span>
        </Transition>
        
        <!-- 主题切换按钮 -->
        <NDropdown 
          :options="themeOptions" 
          @select="handleThemeSelect"
          placement="bottom-start"
        >
          <NButton quaternary circle size="small" class="theme-toggle-btn">
            <template #icon>
              <NIcon :component="themeIcon" size="18" />
            </template>
          </NButton>
        </NDropdown>
      </div>
      
      <NDivider style="margin: 8px 0" />
      
      <!-- 导航菜单 -->
      <NMenu
        :collapsed="collapsed"
        :collapsed-width="64"
        :collapsed-icon-size="22"
        :options="menuOptions"
        :value="activeKey"
        @update:value="handleMenuSelect"
        class="nav-menu"
      />
      
      <!-- 标签分组（仅在首页显示且未折叠时） -->
      <Transition name="fade">
        <div v-if="!collapsed && route.path === '/'" class="tag-group-section">
          <NDivider style="margin: 8px 0" />
          <TagGroupTree @edit="handleEditGroup" />
        </div>
      </Transition>
      
    </NLayoutSider>
    
    <!-- 主内容区 -->
    <NLayout>
      <!-- 顶部栏 -->
      <NLayoutHeader class="header glass" bordered>
        <NSpace align="center">
          <NButton quaternary circle @click="collapsed = !collapsed">
            <template #icon>
              <NIcon :component="collapsed ? ChevronForwardOutline : ChevronBackOutline" />
            </template>
          </NButton>
          <span class="page-title">{{ route.meta.title || '数据监控' }}</span>
        </NSpace>
        
        <NSpace align="center" :size="12">
          <!-- 数据库未连接警告 -->
          <NAlert 
            v-if="!configStore.isConnected" 
            type="warning" 
            :bordered="false"
            class="connection-alert"
            closable
            style="display: inline-flex; align-items: center;"
          >
            <template #icon>
              <NIcon :component="WarningOutline" />
            </template>
            数据库未连接，请前往
            <span class="alert-link" @click="router.push('/settings')">设置</span>
            配置数据库
          </NAlert>
          
          <!-- 连接状态指示 -->
          <NTooltip>
            <template #trigger>
              <NSpace align="center" :size="6">
                <span class="status-dot" :class="configStore.isConnected ? 'online' : 'offline'"></span>
                <span class="status-text">{{ configStore.isConnected ? '已连接' : '未连接' }}</span>
              </NSpace>
            </template>
            {{ configStore.isConnected ? '数据库连接正常' : '请在设置中配置并测试数据库连接' }}
          </NTooltip>
          
          <!-- 更多操作 -->
          <NDropdown 
            :options="moreOptions" 
            @select="handleMoreSelect"
            placement="bottom-end"
          >
            <NButton quaternary circle>
              <template #icon>
                <NIcon :component="EllipsisVertical" />
              </template>
            </NButton>
          </NDropdown>
        </NSpace>
      </NLayoutHeader>
      
      <!-- 内容区 -->
      <NLayoutContent class="content" :native-scrollbar="false">
        <!-- 编辑分组模式 -->
        <GroupEditView 
          v-if="editingGroupId && route.path === '/'"
          :group-id="editingGroupId"
          @back="handleBackFromEdit"
          @saved="handleGroupSaved"
        />
        <!-- 正常内容 -->
        <slot v-else />
      </NLayoutContent>
    </NLayout>
    
    <!-- 关于弹窗 -->
    <NModal v-model:show="showAboutModal" preset="card" title="关于" style="width: 400px; max-width: 90vw">
      <div class="about-content">
        <div class="about-logo">
          <NIcon size="48" :color="themeStore.isDark ? '#60a5fa' : '#3b82f6'">
            <ServerOutline />
          </NIcon>
        </div>
        <h2 class="about-title">工业数据可视化</h2>
        <p class="about-version">v{{ appVersion }}</p>
        <p class="about-desc">工业时序数据查询与可视化分析工具</p>
        <div class="about-copyright">© 2025 Bahayonghang. All rights reserved.</div>
      </div>
    </NModal>
  </NLayout>
</template>

<style scoped>
/* ===== 赛博朋克主布局 ===== */
.main-layout {
  height: 100vh;
  background: var(--bg-base);
}

/* 侧边栏 - 霓虹玻璃效果 */
.sidebar {
  background: var(--sidebar-bg) !important;
  border-right: 1px solid var(--sidebar-border) !important;
  display: flex;
  flex-direction: column;
  position: relative;
}

/* 侧边栏右侧霓虹边线 */
.sidebar::after {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  width: 1px;
  height: 100%;
  background: linear-gradient(
    180deg,
    transparent 0%,
    var(--neon-cyan) 20%,
    var(--neon-magenta) 50%,
    var(--neon-cyan) 80%,
    transparent 100%
  );
  opacity: 0.4;
}

.sidebar :deep(.n-layout-sider-scroll-container) {
  display: flex;
  flex-direction: column;
}

/* Logo 区域 */
.logo-area {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px;
  min-height: 68px;
}

.theme-toggle-btn {
  margin-left: auto;
  flex-shrink: 0;
  color: var(--neon-cyan);
}

.theme-toggle-btn:hover {
  color: var(--neon-magenta);
}

.logo-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-lg);
  background: linear-gradient(135deg, rgba(0, 245, 255, 0.15), rgba(255, 0, 255, 0.1));
  border: 1px solid rgba(0, 245, 255, 0.2);
  flex-shrink: 0;
  box-shadow: 0 0 15px var(--neon-cyan-glow);
}

.logo-icon :deep(.n-icon) {
  color: var(--neon-cyan) !important;
  filter: drop-shadow(0 0 6px var(--neon-cyan-glow));
}

.logo-text {
  font-family: var(--font-display);
  font-size: var(--text-base);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
  white-space: nowrap;
  text-shadow: 0 0 10px var(--neon-cyan-glow);
}

/* 菜单样式 */
.nav-menu {
  flex-shrink: 0;
}

.nav-menu :deep(.n-menu-item-content) {
  border-radius: var(--radius-lg);
  margin: 4px 10px;
  font-family: var(--font-body);
  letter-spacing: var(--tracking-wide);
  transition: all var(--transition-fast);
}

.nav-menu :deep(.n-menu-item-content:hover) {
  background: var(--bg-hover);
}

.nav-menu :deep(.n-menu-item-content--selected) {
  background: rgba(0, 245, 255, 0.1) !important;
  border: 1px solid rgba(0, 245, 255, 0.2);
  box-shadow: 0 0 10px var(--neon-cyan-glow);
}

/* 标签分组区域 */
.tag-group-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.tag-group-section :deep(.tag-group-tree) {
  flex: 1;
  min-height: 0;
}

/* 关于弹窗 - 赛博朋克风格 */
.about-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 28px 0;
}

.about-logo {
  width: 88px;
  height: 88px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-xl);
  background: linear-gradient(135deg, rgba(0, 245, 255, 0.15), rgba(255, 0, 255, 0.1));
  border: 1px solid rgba(0, 245, 255, 0.25);
  margin-bottom: 20px;
  box-shadow: 0 0 25px var(--neon-cyan-glow);
  animation: pulse-glow 3s ease-in-out infinite;
}

.about-logo :deep(.n-icon) {
  color: var(--neon-cyan) !important;
  filter: drop-shadow(0 0 8px var(--neon-cyan-glow));
}

.about-title {
  margin: 0 0 10px 0;
  font-family: var(--font-display);
  font-size: var(--text-xl);
  font-weight: var(--font-bold);
  letter-spacing: var(--tracking-wider);
  color: var(--text-primary);
  text-shadow: 0 0 15px var(--neon-cyan-glow);
}

.about-version {
  margin: 0 0 14px 0;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--neon-cyan);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wide);
  text-shadow: 0 0 8px var(--neon-cyan-glow);
}

.about-desc {
  margin: 0 0 28px 0;
  font-family: var(--font-body);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  letter-spacing: var(--tracking-wide);
}

.about-copyright {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-muted);
  padding-top: 18px;
  border-top: 1px solid var(--border-default);
  width: 100%;
  letter-spacing: var(--tracking-wide);
}

/* 顶部栏 - 霓虹玻璃效果 */
.header {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  height: 60px;
  /* Fallback: 不透明背景，确保内容可见 */
  background: #0a0e1a !important;
  border-bottom: 1px solid var(--header-border, rgba(0, 245, 255, 0.1)) !important;
}

@supports (backdrop-filter: blur(1px)) {
  .header {
    background: var(--header-bg, rgba(10, 14, 26, 0.9)) !important;
    backdrop-filter: blur(var(--glass-blur, 16px));
    -webkit-backdrop-filter: blur(var(--glass-blur, 16px));
  }
}

/* 顶部栏底部霓虹线 */
.header::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent 0%,
    var(--neon-cyan) 20%,
    var(--neon-magenta) 50%,
    var(--neon-cyan) 80%,
    transparent 100%
  );
  opacity: 0.5;
}

.page-title {
  font-family: var(--font-display);
  font-size: var(--text-base);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
}

/* 状态指示器 */
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  transition: all var(--transition-fast);
}

.status-dot.online {
  background: var(--neon-green);
  box-shadow: 0 0 10px var(--neon-green-glow);
  animation: pulse-glow 2s ease-in-out infinite;
}

.status-dot.offline {
  background: var(--text-muted);
}

.status-text {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wide);
  color: var(--text-secondary);
}

/* 连接警告样式 */
.connection-alert {
  padding: 6px 14px;
  font-family: var(--font-body);
  font-size: var(--text-xs);
  border-radius: var(--radius-lg);
  animation: slideIn 0.4s var(--ease-cyber);
  line-height: 1.5;
  height: 34px;
  border: 1px solid rgba(255, 136, 0, 0.3);
}

.connection-alert :deep(.n-alert-body) {
  padding: 0;
  display: flex;
  align-items: center;
}

.connection-alert :deep(.n-alert__icon) {
  margin-right: 10px;
}

.connection-alert :deep(.n-alert__close) {
  margin-left: 10px;
}

.alert-link {
  color: var(--neon-cyan);
  cursor: pointer;
  font-weight: var(--font-medium);
  text-decoration: underline;
  margin: 0 4px;
}

.alert-link:hover {
  color: var(--neon-magenta);
  text-shadow: 0 0 8px var(--neon-magenta-glow);
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(20px);
    filter: blur(4px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
    filter: blur(0);
  }
}

@keyframes pulse-glow {
  0%, 100% {
    box-shadow: 0 0 10px var(--neon-cyan-glow);
  }
  50% {
    box-shadow: 0 0 20px var(--neon-cyan-glow), 0 0 30px var(--neon-cyan-glow);
  }
}

/* 内容区 */
.content {
  height: calc(100vh - 60px);
  background: var(--bg-base);
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s var(--ease-cyber);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
