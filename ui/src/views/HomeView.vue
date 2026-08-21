<script setup lang="ts">
import { onMounted } from 'vue'

import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

onMounted(async () => {
  await appStore.checkBackend()
})
</script>

<template>
  <div class="home-view">
    <div class="placeholder">
      <n-h2>📡 请求编辑器</n-h2>
      <n-p>这里将会有:</n-p>
      <n-ul>
        <n-li>左侧:集合 / 文件夹树</n-li>
        <n-li>中间:请求编辑(URL / Headers / Body / Auth)</n-li>
        <n-li>底部:响应查看(JSON 高亮 / Header / 耗时)</n-li>
      </n-ul>
      <n-divider />
      <n-p v-if="appStore.appInfo">
        后端已连接:<n-text code>{{ appStore.appInfo.name }} v{{ appStore.appInfo.version }}</n-text>
        <br />
        数据库状态:<n-tag :type="appStore.appInfo.db_status === 'ready' ? 'success' : 'warning'">
          {{ appStore.appInfo.db_status }}
        </n-tag>
      </n-p>
      <n-p v-else>正在连接后端...</n-p>
    </div>
  </div>
</template>

<style scoped>
.home-view {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.placeholder {
  max-width: 600px;
  padding: 32px;
}
</style>