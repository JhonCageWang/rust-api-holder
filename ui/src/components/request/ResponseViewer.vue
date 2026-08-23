<script setup lang="ts">
/**
 * Response 查看器
 *
 * 显示状态码 badge、耗时、大小,然后切标签页看 Body / Headers。
 * Body 用 NCode 自带语法高亮(JSON / HTML / XML 自动识别)。
 */
import { computed, ref } from 'vue'
import type { ApiResponse } from '@/types/api'

const props = defineProps<{
  loading: boolean
  error: string | null
  response: ApiResponse | null
}>()

// 当前展示哪个标签页
const activeTab = ref<'body' | 'headers'>('body')

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

// 格式化耗时 / 大小(让数字更易读)
function fmtMs(ms: number): string {
  return ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(2)} s`
}

function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(2)} MB`
}
</script>

<template>
  <div class="response-viewer">
    <n-spin :show="loading">
      <!-- 错误优先显示(就算有 response 也提示) -->
      <n-alert
        v-if="error"
        type="error"
        :title="'请求失败'"
        closable
        style="margin-bottom: 12px"
      >
        {{ error }}
      </n-alert>

      <template v-if="response">
        <!-- 顶部 meta 条 -->
        <n-space align="center" :wrap-item="false" style="margin-bottom: 12px">
          <n-tag :type="statusColor" round size="large" strong>
            {{ response.status }} {{ response.status_text || '' }}
          </n-tag>
          <n-text depth="3">⏱ {{ fmtMs(response.duration_ms) }}</n-text>
          <n-text depth="3">📦 {{ fmtBytes(response.size_bytes) }}</n-text>
          <n-tag
            v-if="response.headers.length > 0"
            size="small"
            style="margin-left: auto"
          >
            {{ response.headers.length }} headers
          </n-tag>
        </n-space>

        <!-- 标签页:Body / Headers -->
        <n-tabs v-model:value="activeTab" type="line" animated>
          <n-tab-pane name="body" tab="Body">
            <n-code
              v-if="response.body"
              :code="response.body"
              :language="bodyLanguage"
              style="margin-top: 8px"
            />
            <n-empty
              v-else
              description="(响应体为空)"
              size="small"
              style="margin-top: 16px"
            />
          </n-tab-pane>
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
</style>
