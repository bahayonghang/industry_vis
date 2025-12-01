<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NForm, NFormItem, NInput, NInputNumber, NButton, NCard, NSpace, NAlert } from 'naive-ui'
import { useConfigStore } from '@/stores/config'

const configStore = useConfigStore()

const formValue = ref({
  server: 'localhost',
  port: 1433,
  database: '控制器数据库',
  username: 'sa',
  password: '',
  defaultTable: '历史表',
})

const testing = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)

const handleTestConnection = async () => {
  testing.value = true
  testResult.value = null
  
  try {
    const result = await configStore.testConnection(formValue.value)
    testResult.value = result
  } catch (error) {
    testResult.value = { success: false, message: String(error) }
  } finally {
    testing.value = false
  }
}

const handleSave = async () => {
  await configStore.saveConfig(formValue.value)
}

onMounted(async () => {
  const config = await configStore.loadConfig()
  if (config) {
    formValue.value = { ...formValue.value, ...config }
  }
})
</script>

<template>
  <NCard title="数据库连接配置">
    <NForm :model="formValue" label-placement="left" label-width="100">
      <NFormItem label="服务器" path="server">
        <NInput v-model:value="formValue.server" placeholder="localhost" />
      </NFormItem>
      
      <NFormItem label="端口" path="port">
        <NInputNumber v-model:value="formValue.port" :min="1" :max="65535" />
      </NFormItem>
      
      <NFormItem label="数据库" path="database">
        <NInput v-model:value="formValue.database" placeholder="控制器数据库" />
      </NFormItem>
      
      <NFormItem label="用户名" path="username">
        <NInput v-model:value="formValue.username" placeholder="sa" />
      </NFormItem>
      
      <NFormItem label="密码" path="password">
        <NInput
          v-model:value="formValue.password"
          type="password"
          show-password-on="click"
          placeholder="请输入密码"
        />
      </NFormItem>
      
      <NFormItem label="默认表" path="defaultTable">
        <NInput v-model:value="formValue.defaultTable" placeholder="历史表" />
      </NFormItem>
      
      <NFormItem>
        <NSpace>
          <NButton @click="handleTestConnection" :loading="testing">
            测试连接
          </NButton>
          <NButton type="primary" @click="handleSave">
            保存配置
          </NButton>
        </NSpace>
      </NFormItem>
    </NForm>
    
    <NAlert
      v-if="testResult"
      :type="testResult.success ? 'success' : 'error'"
      :title="testResult.success ? '连接成功' : '连接失败'"
      closable
    >
      {{ testResult.message }}
    </NAlert>
  </NCard>
</template>
