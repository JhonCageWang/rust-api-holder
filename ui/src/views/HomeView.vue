<script setup lang="ts">
/**
 * 主界面:多 Tab 请求编辑器 + 响应查看器
 *
 * 布局(从上到下):
 * ┌──────────────────────────────────────────────┐
 * │ [GET /users ×] [POST /login ×] [+]          │   Tab 切换条
 * ├──────────────────────────────────────────────┤
 * │ Method | URL | Send | 💾Save                │   工具栏
 * ├──────────────────────────────────────────────┤
 * │ [Params] [Headers] [Body] [Auth]              │
 * │  子编辑器                                    │   请求区
 * ├──────────────────────────────────────────────┤
 * │ Response: [200] ⏱234ms 📦1.2KB 📋 Copy    │
 * │ [Body] [Raw] [Headers]                      │
 * │  响应内容(高亮 JSON)                       │   响应区
 * └──────────────────────────────────────────────┘
 *
 * 数据来自 tabs store,每个 Tab 有自己的完整状态
 */

import { computed, onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'

import { useTabsStore } from '@/stores/tabs'
import { invokeT } from '@/composables/useInvoke'
import type { Collection, HttpMethod } from '@/types/api'

import RequestTabs from '@/components/RequestTabs.vue'
import KeyValueEditor from '@/components/request/KeyValueEditor.vue'
import BodyEditor from '@/components/request/BodyEditor.vue'
import AuthEditor from '@/components/request/AuthEditor.vue'
import ResponseViewer from '@/components/request/ResponseViewer.vue'

const message = useMessage()

// ─── Store ──────────────────────────────────────────────

const tabsStore = useTabsStore()

// ─── Lifecycle ──────────────────────────────────────────

onMounted(() => {
  // 确保至少有 1 个 Tab
  tabsStore.ensureNonEmpty()
})

// ─── Computed ───────────────────────────────────────────

const activeTab = computed(() => tabsStore.activeTab)

// 当前激活 Tab 的本地代理(用 computed getter/setter,实时同步到 store)
const method = computed<HttpMethod>({
  get: () => activeTab.value?.request.method ?? 'GET',
  set: (m) => tabsStore.updateActiveRequest({ method: m }),
})
const url = computed<string>({
  get: () => activeTab.value?.request.url ?? '',
  set: (v) => tabsStore.updateActiveRequest({ url: v }),
})
const query = computed({
  get: () => activeTab.value?.request.query ?? [],
  set: (v) => tabsStore.updateActiveRequest({ query: v }),
})
const headers = computed({
  get: () => activeTab.value?.request.headers ?? [],
  set: (v) => tabsStore.updateActiveRequest({ headers: v }),
})
const body = computed({
  get: () => activeTab.value?.request.body ?? { type: 'none' },
  set: (v) => tabsStore.updateActiveRequest({ body: v }),
})
const auth = computed({
  get: () => activeTab.value?.request.auth ?? { type: 'none' },
  set: (v) => tabsStore.updateActiveRequest({ auth: v }),
})

const loading = computed(() => activeTab.value?.isLoading ?? false)
const error = computed(() => activeTab.value?.error ?? null)
const response = computed(() => activeTab.value?.response ?? null)

// 请求子 tab(Params / Headers / Body / Auth)— 仅 UI 状态
type SubTab = 'params' | 'headers' | 'body' | 'auth'
const activeSubTab = ref<SubTab>('params')

// ─── Save to collection ──────────────────────────────────
const collections = ref<Collection[]>([])
const showSaveModal = ref(false)
const saveTargetCollId = ref<string | null>(null)
const saveRequestName = ref('')

async function loadCollections() {
  try {
    collections.value = await invokeT('list_collections', undefined)
  } catch (e) {
    message.error(`加载集合失败: ${(e as Error).message}`)
  }
}

function openSaveDialog() {
  if (!activeTab.value) return
  loadCollections()
  // 从 URL 提取默认名字(最后一段路径)
  try {
    const u = new URL(url.value)
    const last = u.pathname.split('/').filter(Boolean).pop() || u.host
    saveRequestName.value = last
  } catch {
    saveRequestName.value = url.value.slice(0, 30)
  }
  saveTargetCollId.value = collections.value[0]?.id ?? null
  showSaveModal.value = true
}

async function confirmSave() {
  if (!activeTab.value) return
  if (!saveTargetCollId.value) {
    message.warning('请选择一个集合')
    return
  }
  const name = saveRequestName.value.trim() || 'Untitled'
  try {
    await invokeT('create_request', {
      new: {
        collection_id: saveTargetCollId.value,
        name,
        method: method.value,
        url: url.value,
        headers: headers.value,
        query: query.value,
        body: body.value,
        auth: auth.value,
      },
    })
    message.success(`已保存到集合`)
    showSaveModal.value = false
  } catch (e) {
    message.error(`保存失败: ${(e as Error).message}`)
  }
}

// ─── Constants ──────────────────────────────────────────

const METHOD_OPTIONS: { label: HttpMethod; value: HttpMethod }[] = [
  { label: 'GET', value: 'GET' },
  { label: 'POST', value: 'POST' },
  { label: 'PUT', value: 'PUT' },
  { label: 'PATCH', value: 'PATCH' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'HEAD', value: 'HEAD' },
  { label: 'OPTIONS', value: 'OPTIONS' },
]

// ─── Actions ────────────────────────────────────────────

async function sendRequest(): Promise<void> {
  await tabsStore.sendActive()
}
</script>

<template>
  <div class="home-view">
    <!-- ① Tab 切换条 -->
    <RequestTabs />

    <!-- ② 顶部:Method + URL + Send(只要有 activeTab 就显示) -->
    <template v-if="activeTab">
      <div class="toolbar">
        <n-space :wrap-item="false" align="center" :size="8">
          <n-select
            :value="method"
            :options="METHOD_OPTIONS"
            style="width: 130px"
            @update:value="(v: HttpMethod) => (method = v)"
          />
          <n-input
            v-model:value="url"
            placeholder="https://example.com/path"
            clearable
            :input-props="{ autocomplete: 'off' }"
            style="flex: 1"
            @keydown.enter="sendRequest"
          />
          <n-button @click="openSaveDialog">💾 保存</n-button>
          <n-button
            type="primary"
            :loading="loading"
            :disabled="!url.trim()"
            @click="sendRequest"
          >
            🚀 Send
          </n-button>
        </n-space>
      </div>

      <!-- ③ 中间:请求编辑器 -->
      <section class="panel">
        <n-tabs
          v-model:value="activeSubTab"
          type="line"
          animated
          class="request-tabs"
        >
          <n-tab-pane
            name="params"
            :tab="`Query Params (${query.length})`"
          >
            <KeyValueEditor v-model="query" />
          </n-tab-pane>
          <n-tab-pane
            name="headers"
            :tab="`Headers (${headers.length})`"
          >
            <KeyValueEditor v-model="headers" />
          </n-tab-pane>
          <n-tab-pane name="body" tab="Body">
            <BodyEditor v-model="body" />
          </n-tab-pane>
          <n-tab-pane name="auth" tab="Auth">
            <AuthEditor v-model="auth" />
          </n-tab-pane>
        </n-tabs>
      </section>

      <!-- ④ 底部:响应查看器 -->
      <section class="panel">
        <h3 class="panel-title">Response</h3>
        <ResponseViewer
          :loading="loading"
          :error="error"
          :response="response"
        />
      </section>
    </template>

    <n-empty
      v-else
      description="点击 + 新建一个请求"
      size="large"
      style="margin-top: 64px"
    />

    <!-- 保存请求到 collection 弹窗 -->
    <n-modal
      v-model:show="showSaveModal"
      preset="card"
      title="💾 保存请求到集合"
      style="max-width: 480px"
    >
      <n-space vertical>
        <n-text>选择目标集合:</n-text>
        <n-select
          v-model:value="saveTargetCollId"
          :options="collections.map(c => ({ label: c.name, value: c.id }))"
          placeholder="选择一个集合"
        />
        <n-text>请求名称:</n-text>
        <n-input
          v-model:value="saveRequestName"
          placeholder="请求名称"
          @keydown.enter="confirmSave"
        />
      </n-space>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showSaveModal = false">取消</n-button>
          <n-button type="primary" @click="confirmSave">保存</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
.home-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 12px;
  box-sizing: border-box;
  gap: 12px;
}

/* 顶部工具栏:固定高度 */
.toolbar {
  flex-shrink: 0;
}

/* 请求区 + 响应区:各占剩余高度,各自内部滚动 */
.panel {
  flex: 1 1 0;
  min-height: 0;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  padding: 8px 12px 12px;
  overflow: auto;
  display: flex;
  flex-direction: column;
}

.panel-title {
  margin: 0 0 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color-2);
}

/* 让 NTabs 内部的 n-tab-pane 占满剩余高度 */
.panel :deep(.n-tabs) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel :deep(.n-tab-pane) {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}
</style>