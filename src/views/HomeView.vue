<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { 
  NCard, 
  NGrid, 
  NGi, 
  NButton, 
  NIcon, 
  NEmpty,
  NStatistic,
  NTag,
  NSpin,
  NModal,
  NInput,
  useMessage
} from 'naive-ui'
import { 
  AddOutline, 
  LayersOutline,
  ServerOutline,
  PricetagsOutline,
  TimeOutline,
  ChevronForwardOutline,
  AnalyticsOutline
} from '@vicons/ionicons5'
import MainLayout from '@/layouts/MainLayout.vue'
import GroupEditView from '@/components/GroupEditView.vue'
import { useTagGroupStore } from '@/stores/tagGroup'
import { useConfigStore } from '@/stores/config'
import { APP_VERSION } from '@/version'

const tagGroupStore = useTagGroupStore()
const configStore = useConfigStore()
const message = useMessage()

// 当前视图状态
const currentView = ref<'list' | 'edit'>('list')
const editingGroupId = ref<string | null>(null)

// 新建分组弹窗
const showCreateModal = ref(false)
const newGroupName = ref('')
const creating = ref(false)

// 计算属性
const groups = computed(() => tagGroupStore.groups)
const loading = computed(() => tagGroupStore.loading)
const isConnected = computed(() => configStore.isConnected)
const dbConfig = computed(() => configStore.config?.database)


onMounted(async () => {
  await tagGroupStore.loadGroups()
})

// 打开分组编辑
function openGroup(groupId: string) {
  editingGroupId.value = groupId
  currentView.value = 'edit'
}

// 返回列表
function handleBack() {
  currentView.value = 'list'
  editingGroupId.value = null
}

// 创建新分组
async function handleCreate() {
  if (!newGroupName.value.trim()) {
    message.warning('请输入分组名称')
    return
  }
  
  creating.value = true
  try {
    const group = await tagGroupStore.createGroup(newGroupName.value.trim(), [])
    if (group) {
      message.success('分组创建成功')
      showCreateModal.value = false
      newGroupName.value = ''
      // 打开新创建的分组
      openGroup(group.id)
    }
  } finally {
    creating.value = false
  }
}

function openCreateModal() {
  newGroupName.value = ''
  showCreateModal.value = true
}

// 格式化时间
function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleDateString('zh-CN', { 
    month: 'short', 
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<template>
  <MainLayout>
    <!-- 分组编辑视图 -->
    <GroupEditView 
      v-if="currentView === 'edit' && editingGroupId"
      :group-id="editingGroupId"
      @back="handleBack"
      @saved="handleBack"
      @deleted="handleBack"
    />
    
    <!-- 主页列表视图 -->
    <div v-else class="home-view">
      <!-- 页面标题 -->
      <div class="page-header">
        <div class="page-header-left">
          <h1 class="page-title">数据监控中心</h1>
          <p class="page-subtitle">管理标签分组，查看历史数据趋势</p>
        </div>
        <span class="page-version">v{{ APP_VERSION }}</span>
      </div>
      
      <!-- 系统状态卡片 -->
      <div class="status-section">
        <NGrid :cols="3" :x-gap="16" :y-gap="16" responsive="screen" item-responsive>
          <NGi span="0:24 600:12 900:8">
            <NCard class="stat-card glass-card" :bordered="false">
              <NStatistic label="数据库状态">
                <template #prefix>
                  <NIcon :component="ServerOutline" :size="20" />
                </template>
                <NTag :type="isConnected ? 'success' : 'error'" size="small">
                  {{ isConnected ? '已连接' : '未连接' }}
                </NTag>
              </NStatistic>
              <div v-if="isConnected && dbConfig" class="stat-detail">
                {{ dbConfig.server }}:{{ dbConfig.port }}
              </div>
            </NCard>
          </NGi>
          
          <NGi span="0:24 600:12 900:8">
            <NCard class="stat-card glass-card" :bordered="false">
              <NStatistic label="分组数量" :value="groups.length">
                <template #prefix>
                  <NIcon :component="LayersOutline" :size="20" />
                </template>
              </NStatistic>
            </NCard>
          </NGi>
          
          <NGi span="0:24 600:12 900:8">
            <NCard class="stat-card glass-card" :bordered="false">
              <NStatistic label="总图表数" :value="groups.reduce((sum, g) => sum + (g.charts?.length || 0), 0)">
                <template #prefix>
                  <NIcon :component="AnalyticsOutline" :size="20" />
                </template>
              </NStatistic>
            </NCard>
          </NGi>
          
        </NGrid>
      </div>
      
      <!-- 分组列表 -->
      <div class="groups-section">
        <div class="section-header">
          <h2 class="section-title">我的分组</h2>
          <NButton type="primary" @click="openCreateModal">
            <template #icon>
              <NIcon :component="AddOutline" />
            </template>
            新建分组
          </NButton>
        </div>
        
        <NSpin :show="loading">
          <NGrid 
            v-if="groups.length > 0" 
            :cols="4" 
            :x-gap="16" 
            :y-gap="16" 
            responsive="screen"
            item-responsive
          >
            <NGi v-for="(group, index) in groups" :key="group.id" span="0:24 600:12 900:8 1200:6">
              <NCard
                class="group-card glass-card hoverable"
                :bordered="false"
                :style="{ animationDelay: `${index * 0.08}s` }"
                @click="openGroup(group.id)"
              >
                <div class="group-content">
                  <div class="group-header">
                    <NIcon :component="LayersOutline" :size="24" class="group-icon" />
                    <h3 class="group-name">{{ group.name }}</h3>
                  </div>
                  
                  <div class="group-info">
                    <div class="info-item">
                      <NIcon :component="AnalyticsOutline" :size="14" />
                      <span>{{ group.charts?.length || 0 }} 个图表</span>
                    </div>
                    <div class="info-item">
                      <NIcon :component="PricetagsOutline" :size="14" />
                      <span>{{ group.charts?.reduce((sum, c) => sum + c.tags.length, 0) || 0 }} 个标签</span>
                    </div>
                    <div class="info-item">
                      <NIcon :component="TimeOutline" :size="14" />
                      <span>{{ formatTime(group.updatedAt) }}</span>
                    </div>
                  </div>
                  
                  <div class="group-charts">
                    <NTag 
                      v-for="chart in (group.charts || []).slice(0, 3)" 
                      :key="chart.id" 
                      size="small"
                      round
                      type="info"
                    >
                      {{ chart.name }}
                    </NTag>
                    <NTag v-if="(group.charts?.length || 0) > 3" size="small" round>
                      +{{ (group.charts?.length || 0) - 3 }}
                    </NTag>
                  </div>
                </div>
                
                <div class="group-arrow">
                  <NIcon :component="ChevronForwardOutline" :size="20" />
                </div>
              </NCard>
            </NGi>
            
            <!-- 新建分组卡片 -->
            <NGi span="0:24 600:12 900:8 1200:6">
              <NCard 
                class="group-card group-card-new glass-card hoverable" 
                :bordered="false"
                @click="openCreateModal"
              >
                <div class="new-group-content">
                  <NIcon :component="AddOutline" :size="32" class="add-icon" />
                  <span>新建分组</span>
                </div>
              </NCard>
            </NGi>
          </NGrid>
          
          <NEmpty v-else description="暂无分组，点击新建开始使用" size="large">
            <template #extra>
              <NButton type="primary" @click="openCreateModal">
                <template #icon>
                  <NIcon :component="AddOutline" />
                </template>
                新建分组
              </NButton>
            </template>
          </NEmpty>
        </NSpin>
      </div>
      
      <!-- 新建分组弹窗 -->
      <NModal
        v-model:show="showCreateModal"
        preset="dialog"
        title="新建分组"
        positive-text="创建"
        negative-text="取消"
        :loading="creating"
        @positive-click="handleCreate"
      >
        <NInput 
          v-model:value="newGroupName" 
          placeholder="输入分组名称"
          :maxlength="50"
          @keyup.enter="handleCreate"
        />
      </NModal>
    </div>
  </MainLayout>
</template>

<style scoped>
/* ===== 赛博朋克主页样式 ===== */
.home-view {
  padding: 28px;
  height: 100%;
  overflow-y: auto;
  animation: cyber-fade-in 0.5s var(--ease-cyber) forwards;
  position: relative;
}

/* 网格背景 */
.home-view::before {
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
  margin-bottom: 28px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  position: relative;
  z-index: 1;
}

.page-header-left {
  flex: 1;
}

.page-version {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  letter-spacing: var(--tracking-wider);
  color: var(--neon-cyan);
  background: rgba(0, 245, 255, 0.08);
  padding: 6px 14px;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(0, 245, 255, 0.25);
  text-transform: uppercase;
  box-shadow: 0 0 10px var(--neon-cyan-glow);
}

.page-title {
  margin: 0 0 10px 0;
  font-family: var(--font-display);
  font-size: var(--text-4xl);
  font-weight: var(--font-bold);
  letter-spacing: var(--tracking-wider);
  color: var(--text-primary);
  text-shadow: 0 0 20px var(--neon-cyan-glow);
}

.page-subtitle {
  margin: 0;
  font-family: var(--font-body);
  font-size: var(--text-base);
  color: var(--text-secondary);
  letter-spacing: var(--tracking-wide);
}

.status-section {
  margin-bottom: 36px;
  position: relative;
  z-index: 1;
}

.stat-card {
  height: 100%;
  position: relative;
}

/* 统计卡片顶部霓虹条 */
.stat-card::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--neon-cyan), var(--neon-magenta));
  opacity: 0.7;
}

.stat-card :deep(.n-statistic-value) {
  font-family: var(--font-display);
  font-size: var(--text-xl);
  color: var(--neon-cyan);
  text-shadow: 0 0 10px var(--neon-cyan-glow);
}

.stat-card :deep(.n-statistic-label) {
  font-family: var(--font-body);
  letter-spacing: var(--tracking-wide);
  text-transform: uppercase;
  font-size: var(--text-xs);
}

.stat-detail {
  margin-top: 10px;
  font-size: var(--text-xs);
  color: var(--text-muted);
  font-family: var(--font-mono);
  letter-spacing: var(--tracking-wide);
}

.groups-section {
  flex: 1;
  position: relative;
  z-index: 1;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.section-title {
  margin: 0;
  font-family: var(--font-display);
  font-size: var(--text-xl);
  font-weight: var(--font-semibold);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 12px;
}

/* 标题前的霓虹装饰 */
.section-title::before {
  content: '//';
  color: var(--neon-cyan);
  font-family: var(--font-mono);
  opacity: 0.6;
}

.group-card {
  cursor: pointer;
  transition: all var(--transition-normal);
  display: flex;
  align-items: center;
  padding: 18px;
  position: relative;
  animation: group-card-in 0.4s cubic-bezier(0.23, 1, 0.32, 1) backwards;
}

@keyframes group-card-in {
  from {
    opacity: 0;
    transform: translateY(16px) scale(0.96);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* 分组卡片角落装饰 */
.group-card::after {
  content: '';
  position: absolute;
  top: 10px;
  right: 10px;
  width: 16px;
  height: 16px;
  border-top: 2px solid var(--neon-cyan);
  border-right: 2px solid var(--neon-cyan);
  opacity: 0.4;
  transition: opacity var(--transition-fast);
}

.group-card:hover::after {
  opacity: 0.8;
}

.group-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--glass-shadow-hover);
}

.group-content {
  flex: 1;
  min-width: 0;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 14px;
}

.group-icon {
  color: var(--neon-cyan);
  flex-shrink: 0;
  filter: drop-shadow(0 0 6px var(--neon-cyan-glow));
}

.group-name {
  margin: 0;
  font-family: var(--font-display);
  font-size: var(--text-lg);
  font-weight: var(--font-medium);
  letter-spacing: var(--tracking-wide);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.group-info {
  display: flex;
  flex-wrap: wrap;
  gap: 18px;
  margin-bottom: 14px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-muted);
  letter-spacing: var(--tracking-wide);
}

.group-charts {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.group-arrow {
  color: var(--neon-cyan);
  flex-shrink: 0;
  margin-left: 14px;
  opacity: 0.6;
  transition: all var(--transition-fast);
}

.group-card:hover .group-arrow {
  opacity: 1;
  transform: translateX(4px);
}

.group-card-new {
  justify-content: center;
  min-height: 150px;
  border: 2px dashed var(--border-default);
  background: transparent;
}

.group-card-new:hover {
  border-color: var(--neon-cyan);
  background: var(--bg-hover);
  box-shadow: 0 0 25px var(--neon-cyan-glow);
}

.new-group-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  color: var(--text-muted);
  font-family: var(--font-display);
  letter-spacing: var(--tracking-wide);
}

.add-icon {
  opacity: 0.5;
  transition: all var(--transition-normal);
}

.group-card-new:hover .add-icon {
  opacity: 1;
  color: var(--neon-cyan);
  filter: drop-shadow(0 0 8px var(--neon-cyan-glow));
  transform: scale(1.1);
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
