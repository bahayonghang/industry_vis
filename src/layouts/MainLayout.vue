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
.main-layout {
  height: 100vh;
  background: var(--bg-base);
}

/* 侧边栏 */
.sidebar {
  background: var(--sidebar-bg) !important;
  border-right: 1px solid var(--sidebar-border) !important;
  display: flex;
  flex-direction: column;
}

.sidebar :deep(.n-layout-sider-scroll-container) {
  display: flex;
  flex-direction: column;
}

/* Logo 区域 */
.logo-area {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  min-height: 64px;
}

.theme-toggle-btn {
  margin-left: auto;
  flex-shrink: 0;
}

.logo-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(139, 92, 246, 0.1));
  flex-shrink: 0;
}

.logo-text {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
}

/* 菜单样式 */
.nav-menu {
  flex-shrink: 0;
}

.nav-menu :deep(.n-menu-item-content) {
  border-radius: 8px;
  margin: 4px 8px;
}

/* 标签分组区域 */
.tag-group-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 关于弹窗 */
.about-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 24px 0;
}

.about-logo {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 20px;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(139, 92, 246, 0.1));
  margin-bottom: 16px;
}

.about-title {
  margin: 0 0 8px 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}

.about-version {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--industrial-blue);
  font-weight: 500;
}

.about-desc {
  margin: 0 0 24px 0;
  font-size: 14px;
  color: var(--text-secondary);
}

.about-copyright {
  font-size: 12px;
  color: var(--text-tertiary);
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
  width: 100%;
}

.tag-group-section :deep(.tag-group-tree) {
  flex: 1;
  min-height: 0;
}

/* 顶部栏 */
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  height: 56px;
  background: var(--header-bg) !important;
  border-bottom: 1px solid var(--header-border) !important;
}

.page-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.status-text {
  font-size: 13px;
  color: var(--text-secondary);
}

/* 连接警告样式 */
.connection-alert {
  padding: 6px 12px;
  font-size: 13px;
  border-radius: 8px;
  animation: slideIn 0.3s ease-out;
}

.connection-alert :deep(.n-alert-body) {
  padding: 0;
}

.alert-link {
  color: var(--accent-primary);
  cursor: pointer;
  font-weight: 500;
  text-decoration: underline;
}

.alert-link:hover {
  color: var(--accent-primary-hover);
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

/* 内容区 */
.content {
  height: calc(100vh - 56px);
  background: var(--bg-base);
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
