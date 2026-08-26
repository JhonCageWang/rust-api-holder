<script setup lang="ts">
/**
 * Response 查看器
 *
 * 滚动策略:tab-body 内容区自己滚动(overflow:auto),滚动条出现在内容边缘。
 * meta-bar + tab-bar 是普通 flex 头部,天然固定不随滚动消失(不需要 sticky)。
 * v-show 切 tab,DOM 不销毁,内容不丢。
 */
import { computed, ref } from 'vue'
import { useMessage } from 'naive-ui'

import type { ApiResponse } from '@/types/api'

const props = defineProps<{
  loading: boolean
  error: string | null
  response: ApiResponse | null
}>()

const message = useMessage()

const activeTab = ref<'body' | 'headers' | 'raw'>('body')
const wrap = ref(false)

const bodyLanguage = computed<string>(() => {
  if (!props.response) return 'text'
  const ct = props.response.headers.find(
    (h) => h.key.toLowerCase() === 'content-type',
  )
  const v = ct?.value ?? ''
  if (v.includes('json')) return 'json'
  if (v.includes('html')) return 'html'
  if (v.includes('xml')) return 'xml'
  if (!v) {
    // 没有 content-type(老历史记录):按内容嗅探 JSON
    const body = props.response.body.trim()
    if (body.startsWith('{') || body.startsWith('[')) {
      try {
        JSON.parse(body)
        return 'json'
      } catch {
        // 不是合法 JSON,按 text 展示
      }
    }
  }
  return 'text'
})

const prettyBody = computed<string>(() => {
  const raw = props.response?.body ?? ''
  if (!raw) return ''
  if (bodyLanguage.value !== 'json') return raw
  try {
    const parsed = JSON.parse(raw)
    return JSON.stringify(parsed, null, 2)
  } catch {
    return raw
  }
})

const statusColor = computed<
  'default' | 'success' | 'warning' | 'error'
>(() => {
  if (!props.response) return 'default'
  const s = props.response.status
  if (s === 0) return 'error'
  if (s >= 200 && s < 300) return 'success'
  if (s >= 300 && s < 400) return 'warning'
  if (s >= 400 && s < 600) return 'error'
  return 'default'
})

function fmtMs(ms: number): string {
  return ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(2)} s`
}

function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(2)} MB`
}

async function copy(text: string, label = '已复制'): Promise<void> {
  try {
    await navigator.clipboard.writeText(text)
    message.success(label, { duration: 1500 })
  } catch (e) {
    message.error(`复制失败: ${String(e)}`)
  }
}

async function copyCurrent(): Promise<void> {
  if (!props.response) return
  if (activeTab.value === 'body') {
    await copy(prettyBody.value, '已复制 Body(JSON 已格式化)')
  } else if (activeTab.value === 'raw') {
    await copy(props.response.body, '已复制原始 Body')
  } else {
    const text = props.response.headers
      .map((h) => `${h.key}: ${h.value}`)
      .join('\n')
    await copy(text, '已复制 Headers')
  }
}

async function copyAll(): Promise<void> {
  if (!props.response) return
  const r = props.response
  const headerText = r.headers.map((h) => `${h.key}: ${h.value}`).join('\n')
  const text = [
    `HTTP/${r.status} ${r.status_text}`,
    '',
    headerText,
    '',
    r.body,
  ].join('\n')
  await copy(text, '已复制完整响应')
}
</script>

<template>
  <div class="response-viewer">
    <div v-if="loading" class="loading-tip">请求中...</div>

    <n-alert
      v-if="error"
      type="error"
      title="请求失败"
      closable
      style="margin-bottom: 12px"
    >
      {{ error }}
    </n-alert>

    <template v-if="response">
      <!-- 固定头部:meta-bar + tab-bar 不随内容滚动 -->
      <div class="resp-header">
        <div class="meta-bar">
          <n-space align="center" :wrap-item="false">
            <n-tag :type="statusColor" round size="large" strong>
              {{ response.status }} {{ response.status_text || '' }}
            </n-tag>
            <n-text depth="3">⏱ {{ fmtMs(response.duration_ms) }}</n-text>
            <n-text depth="3">📦 {{ fmtBytes(response.size_bytes) }}</n-text>
          </n-space>

          <n-space :wrap-item="false">
            <n-button
              v-if="activeTab !== 'headers'"
              size="small"
              quaternary
              @click="wrap = !wrap"
            >
              {{ wrap ? '↩ No Wrap' : '↩ Wrap' }}
            </n-button>
            <n-button size="small" quaternary @click="copyCurrent">
              📋 Copy
            </n-button>
            <n-button size="small" quaternary @click="copyAll">
              📋 Copy All
            </n-button>
          </n-space>
        </div>

        <div class="tab-bar">
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'body' }"
            @click="activeTab = 'body'"
          >Body</button>
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'raw' }"
            @click="activeTab = 'raw'"
          >Raw</button>
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'headers' }"
            @click="activeTab = 'headers'"
          >Headers ({{ response.headers.length }})</button>
        </div>
      </div>

      <!-- 内容区:v-show 切 tab 不销毁 DOM,内容保留 -->
      <div v-show="activeTab === 'body'" class="tab-body">
        <pre v-if="response.body" class="code-block" :class="{ wrap }">{{ prettyBody }}</pre>
        <n-empty v-else description="(响应体为空)" size="small" style="margin-top:16px" />
      </div>

      <div v-show="activeTab === 'raw'" class="tab-body">
        <pre v-if="response.body" class="code-block" :class="{ wrap }">{{ response.body }}</pre>
        <n-empty v-else description="(响应体为空)" size="small" style="margin-top:16px" />
      </div>

      <div v-show="activeTab === 'headers'" class="tab-body">
        <n-table
          v-if="response.headers.length > 0"
          :bordered="false"
          :single-line="false"
          size="small"
          striped
        >
          <thead>
            <tr><th style="width:30%">Key</th><th>Value</th></tr>
          </thead>
          <tbody>
            <tr v-for="(h, idx) in response.headers" :key="idx">
              <td><n-text code>{{ h.key }}</n-text></td>
              <td><n-text code>{{ h.value }}</n-text></td>
            </tr>
          </tbody>
        </n-table>
        <n-empty v-else description="(响应头为空)" size="small" style="margin-top:16px" />
      </div>
    </template>

    <n-empty
      v-else-if="!loading && !error"
      description="还没发送请求 · 点击右上角 Send"
      size="medium"
      style="margin-top: 32px"
    />
  </div>
</template>

<style scoped>
/*
 * 自己是 flex 列:撑满外层 panel,头部固定,内容区滚动。
 * 外层 panel(HomeView) 是 overflow:hidden,不再负责滚动。
 */
.response-viewer {
  width: 100%;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.loading-tip {
  text-align: center;
  padding: 24px;
  color: var(--n-text-color-3);
}

/* 固定头部:不滚动 */
.resp-header {
  flex-shrink: 0;
}

.meta-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.tab-bar {
  display: flex;
  border-bottom: 1px solid var(--n-border-color);
}

.tab-btn {
  padding: 6px 16px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 13px;
  color: var(--n-text-color-3);
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: color 0.2s, border-color 0.2s;
}

.tab-btn:hover {
  color: var(--n-text-color-1);
}

.tab-btn.active {
  color: var(--n-primary-color);
  border-bottom-color: var(--n-primary-color);
  font-weight: 600;
}

/* 内容区:自己滚动,滚动条在内容边缘;长行(pre)也能横向滚 */
.tab-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  margin-top: 8px;
}

.code-block {
  margin: 0;
  padding: 8px;
  min-height: 100%;
  font-size: 13px;
  font-family: 'Fira Code', 'Cascadia Code', monospace;
  background: var(--n-action-color);
  border-radius: 6px;
  white-space: pre;
  line-height: 1.5;
  tab-size: 2;
}

.code-block.wrap {
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
