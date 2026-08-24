<script setup lang="ts">
/**
 * 请求历史视图
 *
 * 列出所有发过的请求(成功 / 失败),点击可以重看响应。
 *
 * Week 4+ 会在 `execute_request` 里自动入库历史,
 * 所以这里只需要读 + 显示。
 */

import { onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'

import { invokeT } from '@/composables/useInvoke'
import type { HistoryEntry } from '@/types/api'

const message = useMessage()

const history = ref<HistoryEntry[]>([])
const loading = ref(false)
const expanded = ref<Set<string>>(new Set())

const PAGE_SIZE = 50

onMounted(load)

async function load() {
  loading.value = true
  try {
    history.value = await invokeT('list_history', {
      limit: PAGE_SIZE,
      offset: 0,
    })
  } catch (e) {
    message.error(`加载失败: ${(e as Error).message}`)
  } finally {
    loading.value = false
  }
}

async function remove(h: HistoryEntry) {
  const ok = window.confirm('确定删除这条历史?')
  if (!ok) return
  try {
    await invokeT('delete_history', { id: h.id })
    history.value = history.value.filter((x) => x.id !== h.id)
    message.success('已删除')
  } catch (e) {
    message.error(`删除失败: ${(e as Error).message}`)
  }
}

async function cleanupOld() {
  const days = window.prompt('清理多少天前的历史?', '30')
  if (!days) return
  const n = parseInt(days, 10)
  if (isNaN(n) || n < 1) {
    message.warning('请输入有效天数')
    return
  }
  try {
    const deleted = await invokeT('delete_old_history', { days: n })
    message.success(`已清理 ${deleted} 条`)
    await load()
  } catch (e) {
    message.error(`清理失败: ${(e as Error).message}`)
  }
}

function toggleExpand(id: string) {
  if (expanded.value.has(id)) {
    expanded.value.delete(id)
  } else {
    expanded.value.add(id)
  }
}

function fmtTime(s: string): string {
  return new Date(s).toLocaleString()
}

function statusColor(code: number | undefined): string {
  if (!code) return 'error'
  if (code >= 200 && code < 300) return 'success'
  if (code >= 300 && code < 400) return 'warning'
  return 'error'
}
</script>

<template>
  <div class="history-view">
    <header class="header">
      <h2>📜 请求历史</h2>
      <n-space>
        <n-button size="small" @click="load">刷新</n-button>
        <n-button size="small" @click="cleanupOld">清理旧记录</n-button>
      </n-space>
    </header>

    <div v-if="loading && history.length === 0" class="loading">
      加载中...
    </div>

    <n-empty
      v-else-if="history.length === 0"
      description="还没有请求历史"
      style="margin-top: 80px"
    >
      <template #extra>
        <n-p>发个请求试试?切到「请求」页面</n-p>
      </template>
    </n-empty>

    <div v-else class="list">
      <div
        v-for="h in history"
        :key="h.id"
        class="item"
        :class="{ expanded: expanded.has(h.id) }"
      >
        <div class="row" @click="toggleExpand(h.id)">
          <n-tag
            :type="statusColor(h.response?.status)"
            size="small"
            style="font-family: monospace; min-width: 44px; text-align: center"
          >
            {{ h.response?.status ?? 'ERR' }}
          </n-tag>
          <span class="method" :class="`m-${h.request_snapshot.method.toLowerCase()}`">
            {{ h.request_snapshot.method }}
          </span>
          <span class="url" :title="h.request_snapshot.url">
            {{ h.request_snapshot.url }}
          </span>
          <span class="time">{{ fmtTime(h.sent_at) }}</span>
          <n-button
            size="tiny"
            type="error"
            @click.stop="remove(h)"
          >
            删
          </n-button>
        </div>

        <div v-if="expanded.has(h.id)" class="detail">
          <div v-if="h.error" class="error">
            ❌ 错误:{{ h.error }}
          </div>
          <div v-else-if="h.response" class="resp">
            <div class="resp-meta">
              <span>⏱ {{ h.response.duration_ms }}ms</span>
              <span>📦 {{ h.response.size_bytes }}B</span>
              <span v-if="h.response.headers.length">
                📋 {{ h.response.headers.length }} 个响应头
              </span>
            </div>
            <pre class="body">{{ h.response.body }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-view {
  height: 100%;
  padding: 16px 24px;
  overflow-y: auto;
  box-sizing: border-box;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header h2 {
  margin: 0;
  font-size: 18px;
}

.loading {
  text-align: center;
  padding: 40px;
  color: #999;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.item {
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  overflow: hidden;
}

.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.15s;
}

.row:hover {
  background: var(--n-hover-color, rgba(0, 0, 0, 0.04));
}

.method {
  font-weight: 600;
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: monospace;
  min-width: 50px;
  text-align: center;
}

.m-get    { color: #18a058; background: rgba(24, 160, 88, 0.1); }
.m-post   { color: #2080f0; background: rgba(32, 128, 240, 0.1); }
.m-put    { color: #f0a020; background: rgba(240, 160, 32, 0.1); }
.m-patch  { color: #9b59b6; background: rgba(155, 89, 182, 0.1); }
.m-delete { color: #d03050; background: rgba(208, 48, 80, 0.1); }
.m-head, .m-options { color: #707070; background: rgba(112, 112, 112, 0.1); }

.url {
  flex: 1;
  font-family: 'Fira Code', monospace;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.time {
  font-size: 11px;
  color: var(--n-text-color-3);
  min-width: 130px;
}

.detail {
  padding: 12px 16px;
  background: var(--n-hover-color, rgba(0, 0, 0, 0.02));
  border-top: 1px solid var(--n-border-color);
}

.error {
  color: #d03050;
  font-size: 12px;
}

.resp-meta {
  display: flex;
  gap: 12px;
  font-size: 11px;
  color: var(--n-text-color-3);
  margin-bottom: 8px;
}

.body {
  margin: 0;
  padding: 8px;
  background: var(--n-color);
  border: 1px solid var(--n-border-color);
  border-radius: 3px;
  font-family: 'Fira Code', monospace;
  font-size: 11px;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 300px;
  overflow-y: auto;
}
</style>