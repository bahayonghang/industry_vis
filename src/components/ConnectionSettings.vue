<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { 
  NForm, 
  NFormItem, 
  NInput, 
  NInputNumber, 
  NButton, 
  NCard, 
  NSpace, 
  NAlert,
  NSelect,
  NIcon,
  NTooltip
} from 'naive-ui'
import { AlertCircleOutline } from '@vicons/ionicons5'
import { useConfigStore } from '@/stores/config'

// 数据库类型定义
type DatabaseType = 'sqlserver' | 'postgres'

interface DatabaseTypeOption {
  label: string
  value: DatabaseType
  port: number
  icon: string
  implemented: boolean
}

const databaseTypes: DatabaseTypeOption[] = [
  { 
    label: 'SQL Server', 
    value: 'sqlserver', 
    port: 1433, 
    icon: '🗄️',
    implemented: true 
  },
  { 
    label: 'PostgreSQL', 
    value: 'postgres', 
    port: 5432, 
    icon: '🐘',
    implemented: false 
  },
]

const configStore = useConfigStore()

// 当前选中的数据库类型
const selectedDbType = ref<DatabaseType>('sqlserver')

// 判断当前选中的数据库是否已实现
const isImplemented = computed(() => {
  const dbType = databaseTypes.find(t => t.value === selectedDbType.value)
  return dbType?.implemented ?? false
})

// 数据库类型选择器选项
const dbTypeOptions = computed(() => 
  databaseTypes.map(t => ({
    label: `${t.icon} ${t.label}`,
    value: t.value,
    disabled: false,
  }))
)

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

// 切换数据库类型时更新默认端口
watch(selectedDbType, (newType) => {
  const dbType = databaseTypes.find(t => t.value === newType)
  if (dbType) {
    formValue.value.port = dbType.port
    // 清除测试结果
    testResult.value = null
  }
})

const handleTestConnection = async () => {
  if (!isImplemented.value) {
    testResult.value = { 
      success: false, 
      message: `${selectedDbType.value === 'postgres' ? 'PostgreSQL' : selectedDbType.value} 连接功能尚未实现，敬请期待！` 
    }
    return
  }
  
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
  if (!isImplemented.value) {
    testResult.value = { 
      success: false, 
      message: `${selectedDbType.value === 'postgres' ? 'PostgreSQL' : selectedDbType.value} 连接功能尚未实现，无法保存配置。` 
    }
    return
  }
  
  await configStore.saveConfig(formValue.value)
  testResult.value = { success: true, message: '配置已保存' }
}

onMounted(async () => {
  const config = await configStore.loadConfig()
  if (config) {
    formValue.value = { ...formValue.value, ...config }
  }
})
</script>

<template>
  <NCard title="数据库连接配置" class="connection-card">
    <NForm :model="formValue" label-placement="left" label-width="100">
      <!-- 数据库类型选择 -->
      <NFormItem label="数据库类型" path="dbType">
        <NSelect
          v-model:value="selectedDbType"
          :options="dbTypeOptions"
          style="width: 200px"
        />
        <NTooltip v-if="!isImplemented">
          <template #trigger>
            <NIcon 
              :component="AlertCircleOutline" 
              :size="18" 
              color="#f59e0b"
              style="margin-left: 8px; cursor: help"
            />
          </template>
          该数据库类型尚未实现
        </NTooltip>
      </NFormItem>
      
      <!-- 未实现提示 -->
      <NAlert 
        v-if="!isImplemented" 
        type="warning" 
        style="margin-bottom: 16px"
      >
        <template #icon>
          <NIcon :component="AlertCircleOutline" />
        </template>
        <strong>功能预留</strong>：{{ selectedDbType === 'postgres' ? 'PostgreSQL' : selectedDbType }} 
        数据源支持正在开发中，当前仅支持 SQL Server 连接。
      </NAlert>
      
      <NFormItem label="服务器" path="server">
        <NInput 
          v-model:value="formValue.server" 
          placeholder="localhost" 
          :disabled="!isImplemented"
        />
      </NFormItem>
      
      <NFormItem label="端口" path="port">
        <NInputNumber 
          v-model:value="formValue.port" 
          :min="1" 
          :max="65535" 
          :disabled="!isImplemented"
        />
      </NFormItem>
      
      <NFormItem label="数据库" path="database">
        <NInput 
          v-model:value="formValue.database" 
          :placeholder="selectedDbType === 'postgres' ? 'postgres' : '控制器数据库'" 
          :disabled="!isImplemented"
        />
      </NFormItem>
      
      <NFormItem label="用户名" path="username">
        <NInput 
          v-model:value="formValue.username" 
          :placeholder="selectedDbType === 'postgres' ? 'postgres' : 'sa'" 
          :disabled="!isImplemented"
        />
      </NFormItem>
      
      <NFormItem label="密码" path="password">
        <NInput
          v-model:value="formValue.password"
          type="password"
          show-password-on="click"
          placeholder="请输入密码"
          :disabled="!isImplemented"
        />
      </NFormItem>
      
      <NFormItem label="默认表" path="defaultTable">
        <NInput 
          v-model:value="formValue.defaultTable" 
          placeholder="历史表" 
          :disabled="!isImplemented"
        />
      </NFormItem>
      
      <NFormItem>
        <NSpace>
          <NButton 
            @click="handleTestConnection" 
            :loading="testing"
            :type="isImplemented ? 'default' : 'warning'"
          >
            测试连接
          </NButton>
          <NButton 
            type="primary" 
            @click="handleSave"
            :disabled="!isImplemented"
          >
            保存配置
          </NButton>
        </NSpace>
      </NFormItem>
    </NForm>
    
    <NAlert
      v-if="testResult"
      :type="testResult.success ? 'success' : 'error'"
      :title="testResult.success ? '成功' : '失败'"
      closable
      @close="testResult = null"
      style="margin-top: 16px"
    >
      {{ testResult.message }}
    </NAlert>
  </NCard>
</template>

<style scoped>
.connection-card {
  max-width: 600px;
}

.connection-card :deep(.n-card__content) {
  padding: 20px;
}
</style>
