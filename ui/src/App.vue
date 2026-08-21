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
              <n-tab-pane name="home" tab="📡 请求" display-directive="show">
                <router-view />
              </n-tab-pane>
              <n-tab-pane name="environments" tab="🌍 环境" display-directive="show">
                <router-view />
              </n-tab-pane>
              <n-tab-pane name="history" tab="📜 历史" display-directive="show">
                <router-view />
              </n-tab-pane>
            </n-tabs>
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
  flex: 1;
  display: flex;
  flex-direction: column;
}

.app-tabs :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  overflow: hidden;
}
</style>