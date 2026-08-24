<script setup lang="ts">
/**
 * Response 查看器
 *
 * 三种查看模式:
 * - Body  : 自动尝试 JSON 格式化(失败则原样),语法高亮
 * - Raw   : 原始字符串(未格式化)
 * - Headers: 键值对表格
 *
 * 顶部工具栏:status / 时间 / 大小 + Wrap toggle + Copy
 *
 * Wrap 行为:
 * - wrap=true  : 长行自动换行(`white-space: pre-wrap`),不丢信息
 * - wrap=false : 默认,横向滚动,适合看 JSON 原始结构
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

// 当前展示哪个标签页
const activeTab = ref<'body' | 'headers' | 'raw'>('body')

// Wrap toggle
const wrap = ref(false)

// 根据响应 content-type 推断 Body 高亮语言
const bodyLanguage = computed<string>(() => {
  if (!props.response) return 'text'
  const ct = props.response.headers.find(
    (h) => h.key.toLowerCase() === 'content-type',
  )
  const v = ct?.value ?? ''
  if (v.includes('json')) return 'json'
  if (v.includes('html')) return 'html'
  if (v.includes('xml')) return 'xml'
  return 'text'
})

// Body 自动 JSON 格式化(只在能 parse 的情况下)
const prettyBody = computed<string>(() => {
  const raw = props.response?.body ?? ''
  if (!raw) return ''
  // 只在 JSON 语言时尝试美化,其他(text/html/xml)保持原样
  if (bodyLanguage.value !== 'json') return raw
  try {
    const parsed = JSON.parse(raw)
    return JSON.stringify(parsed, null, 2)
  } catch {
    return raw
  }
})

// 状态码 → tag 颜色
const statusColor = computed<
  'default' | 'success' | 'warning' | 'error'
>(() => {
  if (!props.response) return 'default'
  const s = props.response.status
  if (s === 0) return 'error' // 网络失败 / 连接被拒
  if (s >= 200 && s < 300) return 'success'
  if (s >= 300 && s < 400) return 'warning'
  if (s >= 400 && s < 600) return 'error'
  return 'default'
})

// 工具函数
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

// 复制当前激活 tab 的内容
async function copyCurrent(): Promise<void> {
  if (!props.response) return
  if (activeTab.value === 'body') {
    await copy(prettyBody.value, '已复制 Body(JSON 已格式化)')
  } else if (activeTab.value === 'raw') {
    await copy(props.response.body, '已复制原始 Body')
  } else {
    // headers
    const text = props.response.headers
      .map((h) => `${h.key}: ${h.value}`)
      .join('\n')
    await copy(text, '已复制 Headers')
  }
}

// 复制全部(响应三部分拼接)
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
    <n-spin :show="loading">
      <!-- 错误优先显示 -->
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
        <!-- 顶部 meta 条 + 工具按钮 -->
        <div class="meta-bar">
          <n-space align="center" :wrap-item="false">
            <n-tag :type="statusColor" round size="large" strong>
              {{ response.status }} {{ response.status_text || '' }}
            </n-tag>
            <n-text depth="3">⏱ {{ fmtMs(response.duration_ms) }}</n-text>
            <n-text depth="3">📦 {{ fmtBytes(response.size_bytes) }}</n-text>
            <n-tag
              v-if="response.headers.length > 0"
              size="small"
              :bordered="false"
            >
              {{ response.headers.length }} headers
            </n-tag>
          </n-space>

          <n-space :wrap-item="false">
            <!-- Wrap toggle(只在 body/raw tab 有意义) -->
            <n-tooltip
              v-if="activeTab !== 'headers'"
              placement="bottom"
            >
              <template #trigger>
                <n-button
                  size="small"
                  quaternary
                  @click="wrap = !wrap"
                >
                  {{ wrap ? '↩ No Wrap' : '↩ Wrap' }}
                </n-button>
              </template>
              {{ wrap ? '当前:长行自动换行' : '当前:横向滚动' }}
            </n-tooltip>

            <!-- 复制当前 tab 内容 -->
            <n-button
              size="small"
              quaternary
              :disabled="!response.body && response.headers.length === 0"
              @click="copyCurrent"
            >
              📋 Copy
            </n-button>

            <!-- 复制全部 -->
            <n-button
              size="small"
              quaternary
              @click="copyAll"
            >
              📋 Copy All
            </n-button>
          </n-space>
        </div>

        <!-- 标签页:Body / Raw / Headers -->
        <n-tabs v-model:value="activeTab" type="line" animated>
          <!-- Body:自动 JSON 格式化 + 语法高亮 -->
          <n-tab-pane name="body" tab="Body">
            <div
              v-if="response.body"
              class="code-container"
              :class="{ wrap }"
            >
              <n-code
                :code="prettyBody"
                :language="bodyLanguage"
                style="margin-top: 8px; font-size: 13px; border-radius: 4px"
              />
            </div>
            <n-empty
              v-else
              description="(响应体为空)"
              size="small"
              style="margin-top: 16px"
            />
          </n-tab-pane>

          <!-- Raw:原始 body,未格式化 -->
          <n-tab-pane name="raw" tab="Raw">
            <div
              v-if="response.body"
              class="code-container"
              :class="{ wrap }"
            >
              <n-code
                :code="response.body"
                language="text"
                style="margin-top: 8px; font-size: 13px; border-radius: 4px"
              />
            </div>
            <n-empty
              v-else
              description="(响应体为空)"
              size="small"
              style="margin-top: 16px"
            />
          </n-tab-pane>

          <!-- Headers:键值对表格 -->
          <n-tab-pane
            name="headers"
            :tab="`Headers (${response.headers.length})`"
          >
            <n-table
              v-if="response.headers.length > 0"
              :bordered="false"
              :single-line="false"
              size="small"
              striped
              style="margin-top: 8px"
            >
              <thead>
                <tr>
                  <th style="width: 30%">Key</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(h, idx) in response.headers" :key="idx">
                  <td>
                    <n-text code>{{ h.key }}</n-text>
                  </td>
                  <td>
                    <n-text code>{{ h.value }}</n-text>
                  </td>
                </tr>
              </tbody>
            </n-table>
            <n-empty
              v-else
              description="(响应头为空)"
              size="small"
              style="margin-top: 16px"
            />
          </n-tab-pane>
        </n-tabs>
      </template>

      <n-empty
        v-else-if="!loading && !error"
        description="还没发送请求 · 点击右上角 Send"
        size="medium"
        style="margin-top: 32px"
      />
    </n-spin>
  </div>
</template>

<style scoped>
.response-viewer {
  width: 100%;
}

.meta-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.code-container {
  margin-top: 8px;
  border-radius: 4px;
  background: rgba(250, 250, 250, 0.6);
}

/* Wrap 模式:长行换行,适合窄屏 */
.code-container.wrap :deep(.n-code) {
  white-space: pre-wrap;
  word-break: break-word;
}

/* No Wrap 模式(默认):横向滚动,保留原始结构 */
.code-container:not(.wrap) {
  overflow-x: auto;
}

.code-container:not(.wrap) :deep(.n-code) {
  white-space: pre;
}
</style>