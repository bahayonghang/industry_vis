<script setup lang="ts">
import { ref, computed, h, provide } from 'vue'
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
  NDropdown
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
  EllipsisVertical
} from '@vicons/ionicons5'
import type { MenuOption, DropdownOption } from 'naive-ui'
import { useThemeStore } from '@/stores/theme'
import TagGroupTree from '@/components/TagGroupTree.vue'
import GroupEditView from '@/components/GroupEditView.vue'

const router = useRouter()
const route = useRoute()
const themeStore = useThemeStore()

// 侧边栏折叠状态
const collapsed = ref(false)

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

const themeLabel = computed(() => {
  if (themeStore.mode === 'light') return '浅色'
  if (themeStore.mode === 'dark') return '深色'
  return '跟随系统'
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
          <span v-if="!collapsed" class="logo-text">工业数据监控</span>
        </Transition>
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
      
      <!-- 底部操作区 -->
      <div class="sidebar-footer">
        <NDivider style="margin: 8px 0" />
        
        <!-- 主题切换 -->
        <NDropdown 
          :options="themeOptions" 
          @select="handleThemeSelect"
          placement="right"
        >
          <NButton quaternary block class="theme-btn">
            <template #icon>
              <NIcon :component="themeIcon" />
            </template>
            <span v-if="!collapsed">{{ themeLabel }}</span>
          </NButton>
        </NDropdown>
      </div>
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
          <!-- 连接状态指示 -->
          <NTooltip>
            <template #trigger>
              <NSpace align="center" :size="6">
                <span class="status-dot online"></span>
                <span class="status-text">已连接</span>
              </NSpace>
            </template>
            数据库连接正常
          </NTooltip>
          
          <!-- 更多操作 -->
          <NButton quaternary circle>
            <template #icon>
              <NIcon :component="EllipsisVertical" />
            </template>
          </NButton>
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

.tag-group-section :deep(.tag-group-tree) {
  flex: 1;
  min-height: 0;
}

/* 侧边栏底部 */
.sidebar-footer {
  padding: 8px;
  margin-top: auto;
}

.theme-btn {
  justify-content: flex-start;
  padding-left: 12px;
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
