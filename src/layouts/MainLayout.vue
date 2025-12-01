<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { 
  NLayout, 
  NLayoutHeader, 
  NLayoutContent,
  NMenu,
  NIcon,
  NSpace
} from 'naive-ui'
import { 
  HomeOutline, 
  SettingsOutline,
  ServerOutline
} from '@vicons/ionicons5'
import type { MenuOption } from 'naive-ui'
import { h } from 'vue'

const router = useRouter()

const menuOptions: MenuOption[] = [
  {
    label: '数据查看',
    key: '/',
    icon: () => h(NIcon, null, { default: () => h(HomeOutline) }),
  },
  {
    label: '设置',
    key: '/settings',
    icon: () => h(NIcon, null, { default: () => h(SettingsOutline) }),
  },
]

const activeKey = ref(router.currentRoute.value.path)

const handleMenuSelect = (key: string) => {
  router.push(key)
  activeKey.value = key
}
</script>

<template>
  <NLayout class="main-layout" has-sider>
    <NLayoutHeader class="header" bordered>
      <NSpace align="center">
        <NIcon size="24" color="#18a058">
          <ServerOutline />
        </NIcon>
        <span class="title">工业数据查看系统</span>
      </NSpace>
      <NMenu
        mode="horizontal"
        :options="menuOptions"
        :value="activeKey"
        @update:value="handleMenuSelect"
      />
    </NLayoutHeader>
    <NLayoutContent class="content">
      <slot />
    </NLayoutContent>
  </NLayout>
</template>

<style scoped>
.main-layout {
  height: 100vh;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  height: 56px;
}

.title {
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.content {
  height: calc(100vh - 56px);
  overflow: auto;
}
</style>
