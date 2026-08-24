<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const activeTab = computed(() => route.name as string)
const setTab = (name: string) => router.push({ name })
</script>

<template>
  <n-config-provider>
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <div class="app">
            <n-tabs
              :value="activeTab"
              type="line"
              @update:value="setTab"
              class="app-tabs"
            >
              <!-- ⚠️ Naive UI 必须用 #tab slot,不能用 tab="..." prop -->
              <n-tab-pane name="home">
                <template #tab>📡 请求</template>
              </n-tab-pane>
              <n-tab-pane name="collections">
                <template #tab>📁 集合</template>
              </n-tab-pane>
              <n-tab-pane name="environments">
                <template #tab>🌍 环境</template>
              </n-tab-pane>
              <n-tab-pane name="history">
                <template #tab>📜 历史</template>
              </n-tab-pane>
            </n-tabs>
            <!-- router-view 在 tabs 外面,只渲染当前路由对应的 view -->
            <div class="app-content">
              <router-view />
            </div>
          </div>
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style scoped>
.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-tabs {
  flex-shrink: 0;
}

.app-content {
  flex: 1;
  overflow: hidden;
}
</style>