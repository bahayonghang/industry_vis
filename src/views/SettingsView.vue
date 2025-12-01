<script setup lang="ts">
import MainLayout from '@/layouts/MainLayout.vue'
import ConnectionSettings from '@/components/ConnectionSettings.vue'
</script>

<template>
  <MainLayout>
    <div class="settings-view">
      <n-h2>设置</n-h2>
      <ConnectionSettings />
    </div>
  </MainLayout>
</template>

<style scoped>
.settings-view {
  padding: 16px;
}
</style>
