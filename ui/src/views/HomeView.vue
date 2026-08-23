<script setup lang="ts">
/**
 * 主界面:请求编辑器 + 响应查看器
 *
 * 布局(从上到下):
 * ┌──────────────────────────────────────────────┐
 * │ Method | URL | Send                            │   工具栏
 * ├──────────────────────────────────────────────┤
 * │ [Params] [Headers] [Body] [Auth]              │
 * │ ┌─────────────────────────────────────────┐  │
 * │ │  对应编辑器的子内容                       │  │   请求区
 * │ └─────────────────────────────────────────┘  │
 * ├──────────────────────────────────────────────┤
 * │ Response: [200 OK] ⏱234ms 📦1.2KB            │
 * │ [Body] [Headers]                              │
 * │ ┌─────────────────────────────────────────┐  │
 * │ │  响应内容(高亮 JSON)                     │  │   响应区
 * │ └─────────────────────────────────────────┘  │
 * └──────────────────────────────────────────────┘
 */
import { ref } from 'vue'
import { useMessage } from 'naive-ui'

import { invokeT } from '@/composables/useInvoke'
import type { ApiRequest, ApiResponse, HttpMethod } from '@/types/api'

import KeyValueEditor from '@/components/request/KeyValueEditor.vue'
import BodyEditor from '@/components/request/BodyEditor.vue'
import AuthEditor from '@/components/request/AuthEditor.vue'
import ResponseViewer from '@/components/request/ResponseViewer.vue'

// ─── 状态 ───────────────────────────────────────────────

// 当前编辑中的请求(完全受控状态,改字段直接写这里)
const request = ref<ApiRequest>({
  method: 'GET',
  url: 'https://httpbin.org/get',
  headers: [],
  query: [],
  body: { type: 'none' },
  auth: { type: 'none' },
})

// 当前展示哪个请求子标签页
type RequestTab = 'params' | 'headers' | 'body' | 'auth'
const activeTab = ref<RequestTab>('params')

// 7 种 HTTP 方法(顺序:最常用 → 最少用)
const METHOD_OPTIONS: { label: HttpMethod; value: HttpMethod }[] = [
  { label: 'GET', value: 'GET' },
  { label: 'POST', value: 'POST' },
  { label: 'PUT', value: 'PUT' },
  { label: 'PATCH', value: 'PATCH' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'HEAD', value: 'HEAD' },
  { label: 'OPTIONS', value: 'OPTIONS' },
]

// 响应相关状态
const loading = ref(false)
const error = ref<string | null>(null)
const response = ref<ApiResponse | null>(null)

// 当前激活的环境变量(Week 6 真的接 Environment,这里先空对象)
const activeVars = ref<Record<string, string>>({})

// ─── 行为 ───────────────────────────────────────────────

const message = useMessage()

function setMethod(m: HttpMethod): void {
  request.value.method = m
}

async function sendRequest(): Promise<void> {
  // 基本校验:URL 不能空
  const url = request.value.url.trim()
  if (!url) {
    message.warning('URL 不能为空')
    return
  }
  if (!/^https?:\/\//i.test(url)) {
    message.warning('URL 必须以 http:// 或 https:// 开头')
    return
  }

  loading.value = true
  error.value = null
  response.value = null

  try {
    // 调后端(在浏览器开发模式时走 useInvoke.ts 里的 mock)
    response.value = await invokeT('execute_request', {
      req: request.value,
      vars: activeVars.value,
    })
  } catch (e) {
    // invokeT 已经把 Tauri 的 anyhow 字符串 wrap 成 Error
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="home-view">
    <!-- ① 顶部:Method + URL + Send -->
    <div class="toolbar">
      <n-space :wrap-item="false" align="center" :size="8">
        <n-select
          :value="request.method"
          :options="METHOD_OPTIONS"
          style="width: 130px"
          @update:value="setMethod"
        />
        <n-input
          v-model:value="request.url"
          placeholder="https://example.com/path"
          clearable
          :input-props="{ autocomplete: 'off' }"
          style="flex: 1"
          @keydown.enter="sendRequest"
        />
        <n-button
          type="primary"
          :loading="loading"
          :disabled="!request.url.trim()"
          @click="sendRequest"
        >
          🚀 Send
        </n-button>
      </n-space>
    </div>

    <!-- ② 中间:请求编辑器 -->
    <section class="panel">
      <n-tabs
        v-model:value="activeTab"
        type="line"
        animated
        class="request-tabs"
      >
        <n-tab-pane
          name="params"
          :tab="`Query Params (${request.query.length})`"
        >
          <KeyValueEditor v-model="request.query" />
        </n-tab-pane>
        <n-tab-pane
          name="headers"
          :tab="`Headers (${request.headers.length})`"
        >
          <KeyValueEditor v-model="request.headers" />
        </n-tab-pane>
        <n-tab-pane name="body" tab="Body">
          <BodyEditor v-model="request.body" />
        </n-tab-pane>
        <n-tab-pane name="auth" tab="Auth">
          <AuthEditor v-model="request.auth" />
        </n-tab-pane>
      </n-tabs>
    </section>

    <!-- ③ 底部:响应查看器 -->
    <section class="panel">
      <h3 class="panel-title">Response</h3>
      <ResponseViewer
        :loading="loading"
        :error="error"
        :response="response"
      />
    </section>
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

/* 顶部工具栏:固定高度,不参与 flex 拉伸 */
.toolbar {
  flex-shrink: 0;
}

/* 请求区 + 响应区:各占一半高度,各自内部滚动 */
.panel {
  flex: 1 1 0;
  min-height: 0; /* 关键:让 flex 子项可以真正收缩并 overflow */
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
