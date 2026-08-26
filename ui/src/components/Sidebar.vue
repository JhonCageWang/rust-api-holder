<script setup lang="ts">
/**
 * 侧边栏:Collections + History + Environment 选择器
 *
 * 布局(从上到下):
 * - Collections(可展开,内含请求列表)
 * - History(最近请求记录)
 * - Environment 选择器(底部)
 *
 * 点击请求/历史 → 在主区域打开新 Tab
 */

import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'

import { invokeT } from '@/composables/useInvoke'
import { useTabsStore } from '@/stores/tabs'
import { useAppStore } from '@/stores/app'
import EnvManagerModal from '@/components/EnvManagerModal.vue'
import type { Collection, Environment, HistoryEntry, RequestItem } from '@/types/api'

const router = useRouter()
const message = useMessage()
const tabsStore = useTabsStore()
const appStore = useAppStore()

// ─── State ──────────────────────────────────────────────
const collections = ref<Collection[]>([])
const requestsByColl = ref<Record<string, RequestItem[]>>({})
const expandedColls = ref<Set<string>>(new Set())
const history = ref<HistoryEntry[]>([])
const environments = ref<Environment[]>([])
const activeEnvId = ref<string | null>(null)

const showNewCollModal = ref(false)
const newCollName = ref('')
const showEnvModal = ref(false)

// ─── Lifecycle ──────────────────────────────────────────
onMounted(loadAll)

// 当 sidebarVersion 变化时刷新(其他组件保存/删除后触发)
watch(() => appStore.sidebarVersion, () => loadAll())

// ─── Actions ────────────────────────────────────────────
async function loadAll() {
  await Promise.all([loadCollections(), loadHistory(), loadEnvironments()])
}

async function loadCollections() {
  requestsByColl.value = {}
  try {
    collections.value = await invokeT('list_collections', undefined)
    for (const c of collections.value) {
      if (!(c.id in requestsByColl.value)) {
        try {
          requestsByColl.value[c.id] = await invokeT('list_requests', {
            collectionId: c.id,
          })
        } catch {
          requestsByColl.value[c.id] = []
        }
      }
    }
  } catch (e) {
    message.error(`加载集合失败: ${(e as Error).message}`)
  }
}

async function loadHistory() {
  try {
    history.value = await invokeT('list_history', { limit: 30, offset: 0 })
  } catch {
    history.value = []
  }
}

async function loadEnvironments() {
  try {
    environments.value = await invokeT('list_environments', undefined)
    const active = await invokeT('get_active_environment', undefined)
    activeEnvId.value = active?.id ?? null
    appStore.activeEnvironmentId = active?.id ?? null
  } catch {
    environments.value = []
  }
}

function toggleColl(id: string) {
  if (expandedColls.value.has(id)) {
    expandedColls.value.delete(id)
  } else {
    expandedColls.value.add(id)
  }
}

function openRequest(item: RequestItem) {
  tabsStore.loadRequest(item)
  router.push('/')
}

function openHistory(entry: HistoryEntry) {
  tabsStore.loadHistory(entry)
  router.push('/')
}

async function createCollection() {
  const name = newCollName.value.trim()
  if (!name) {
    message.warning('请输入集合名称')
    return
  }
  try {
    await invokeT('create_collection', {
      name,
      description: null,
      parentId: null,
    })
    message.success('集合已创建')
    showNewCollModal.value = false
    newCollName.value = ''
    requestsByColl.value = {}
    await loadCollections()
  } catch (e) {
    message.error(`创建失败: ${(e as Error).message}`)
  }
}

async function deleteCollection(c: Collection) {
  try {
    await invokeT('delete_collection', { id: c.id })
    message.success('已删除')
    delete requestsByColl.value[c.id]
    await loadCollections()
  } catch (e) {
    message.error(`删除失败: ${(e as Error).message}`)
  }
}

async function deleteRequest(item: RequestItem) {
  try {
    await invokeT('delete_request', { id: item.id })
    message.success('已删除')
    if (item.collection_id in requestsByColl.value) {
      requestsByColl.value[item.collection_id] = requestsByColl.value[
        item.collection_id
      ].filter((r) => r.id !== item.id)
    }
  } catch (e) {
    message.error(`删除失败: ${(e as Error).message}`)
  }
}

// ─── History 右键菜单 ──────────────────────────────────
const histMenu = ref({ show: false, x: 0, y: 0, id: '' })

function onHistMenu(e: MouseEvent, id: string) {
  e.preventDefault()
  histMenu.value = { show: true, x: e.clientX, y: e.clientY, id }
}

async function onHistMenuSelect(key: string) {
  histMenu.value.show = false
  if (key !== 'delete') return
  try {
    await invokeT('delete_history', { id: histMenu.value.id })
    history.value = history.value.filter((h) => h.id !== histMenu.value.id)
    message.success('已删除')
  } catch (e) {
    message.error(`删除失败: ${(e as Error).message}`)
  }
}

async function switchEnv(id: string | null) {
  if (!id) return
  try {
    await invokeT('set_active_environment', { id })
    activeEnvId.value = id
    appStore.activeEnvironmentId = id
    message.success('环境已切换')
  } catch (e) {
    message.error(`切换失败: ${(e as Error).message}`)
  }
}

function newTab() {
  tabsStore.createTab()
  router.push('/')
}

// ─── Helpers ────────────────────────────────────────────
function fmtTime(s: string): string {
  const d = new Date(s)
  const now = Date.now()
  const diff = now - d.getTime()
  if (diff < 60_000) return '刚刚'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}分钟前`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}小时前`
  return d.toLocaleDateString()
}

function statusType(code: number | undefined): 'success' | 'warning' | 'error' | 'default' {
  if (!code) return 'error'
  if (code >= 200 && code < 300) return 'success'
  if (code >= 300 && code < 400) return 'warning'
  return 'error'
}

const envOptions = computed(() => [
  { label: '(无)', value: '' },
  ...environments.value.map((e) => ({ label: e.name, value: e.id })),
])
</script>

<template>
  <aside class="sidebar">
    <!-- 顶部:新建按钮 -->
    <div class="sidebar-header">
      <span class="sidebar-title">API Holder</span>
      <n-button size="tiny" quaternary @click="newTab">+ 请求</n-button>
    </div>

    <div class="sidebar-scroll">
      <!-- Collections -->
      <section class="section section-collections">
        <header class="section-header">
          <span class="section-title">Collections</span>
          <n-button size="tiny" quaternary circle @click="showNewCollModal = true">+</n-button>
        </header>

        <div class="coll-list">
          <div v-if="collections.length === 0" class="empty-hint">
            还没有集合
          </div>

          <div
            v-for="c in collections"
            :key="c.id"
            class="coll-item"
          >
            <div class="coll-header" @click="toggleColl(c.id)">
              <span class="coll-arrow" :class="{ expanded: expandedColls.has(c.id) }">▶</span>
              <span class="coll-name">{{ c.name }}</span>
              <span class="coll-count">{{ requestsByColl[c.id]?.length ?? 0 }}</span>
              <n-popconfirm @positive-click="deleteCollection(c)">
                <template #trigger>
                  <button class="coll-del" @click.stop>×</button>
                </template>
                确定删除集合 "{{ c.name }}"?
              </n-popconfirm>
            </div>

            <div v-if="expandedColls.has(c.id)" class="coll-requests">
              <div
                v-for="r in requestsByColl[c.id] ?? []"
                :key="r.id"
                class="req-item"
                @click="openRequest(r)"
              >
                <span class="req-method" :class="`m-${r.method.toLowerCase()}`">
                  {{ r.method }}
                </span>
                <span class="req-name" :title="r.url">{{ r.name }}</span>
                <n-popconfirm @positive-click="deleteRequest(r)">
                  <template #trigger>
                    <button class="req-del" @click.stop>×</button>
                  </template>
                  确定删除 "{{ r.name }}"?
                </n-popconfirm>
              </div>
              <div
                v-if="(requestsByColl[c.id]?.length ?? 0) === 0"
                class="empty-hint small"
              >
                空集合
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- History -->
      <section class="section section-history">
        <header class="section-header">
          <span class="section-title">History</span>
          <n-button
            v-if="history.length > 0"
            size="tiny"
            quaternary
            circle
            @click="loadHistory"
          >
            ↻
          </n-button>
        </header>

        <div class="hist-list">
          <div v-if="history.length === 0" class="empty-hint">
            还没有请求历史
          </div>

          <div
            v-for="h in history"
            :key="h.id"
            class="hist-item"
            @click="openHistory(h)"
            @contextmenu="onHistMenu($event, h.id)"
          >
            <n-tag :type="statusType(h.response?.status)" size="tiny" round :bordered="false">
              {{ h.response?.status ?? 'ERR' }}
            </n-tag>
            <span class="hist-method" :class="`m-${h.request_snapshot.method.toLowerCase()}`">
              {{ h.request_snapshot.method }}
            </span>
            <span class="hist-url" :title="h.request_snapshot.url">
              {{ h.request_snapshot.url }}
            </span>
            <span class="hist-time">{{ fmtTime(h.sent_at) }}</span>
          </div>
        </div>
      </section>
    </div>

    <!-- History 右键菜单 -->
    <n-dropdown
      trigger="manual"
      placement="bottom-start"
      :show="histMenu.show"
      :x="histMenu.x"
      :y="histMenu.y"
      :options="[{ label: '删除该条', key: 'delete' }]"
      @select="onHistMenuSelect"
      @clickoutside="histMenu.show = false"
    />

    <!-- Environment selector (bottom) -->
    <div class="sidebar-footer">
      <n-select
        :value="activeEnvId ?? ''"
        :options="envOptions"
        size="small"
        placeholder="选择环境"
        @update:value="switchEnv"
      />
      <n-button size="tiny" quaternary @click="showEnvModal = true">
        管理
      </n-button>
    </div>

    <!-- New collection modal -->
    <n-modal
      v-model:show="showNewCollModal"
      preset="card"
      title="新建集合"
      style="max-width: 400px"
    >
      <n-input
        v-model:value="newCollName"
        placeholder="集合名称"
        @keydown.enter="createCollection"
      />
      <template #footer>
        <n-space justify="end">
          <n-button @click="showNewCollModal = false">取消</n-button>
          <n-button type="primary" @click="createCollection">创建</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- Environment manager modal -->
    <EnvManagerModal v-model="showEnvModal" />
  </aside>
</template>

<style scoped>
.sidebar {
  /* 宽度由 App.vue 内联样式控制(可拖动),分隔线由 .sizer 提供 */
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--n-card-color);
  flex-shrink: 0;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  border-bottom: 1px solid var(--n-border-color);
  flex-shrink: 0;
}

.sidebar-title {
  font-weight: 700;
  font-size: 14px;
  letter-spacing: 0.2px;
}

/* 两 section 共享纵向空间,头部固定,列表各自滚动 */
.sidebar-scroll {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.section {
  display: flex;
  flex-direction: column;
  padding-top: 6px;
}

/* Collections: 内容少时自然高度,多时最多 45%,列表自己滚动 */
.section-collections {
  flex: 0 0 auto;
  max-height: 45%;
}

/* History: 占剩余空间,列表自己滚动 */
.section-history {
  flex: 1;
  min-height: 0;
  border-top: 1px solid var(--n-border-color);
}

.coll-list,
.hist-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 2px 8px 6px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2px 14px 6px;
  flex-shrink: 0;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--n-text-color-3);
  letter-spacing: 0.8px;
}

.empty-hint {
  padding: 8px 6px;
  font-size: 12px;
  color: var(--n-text-color-3);
}

.empty-hint.small {
  padding: 4px 6px 4px 26px;
  font-size: 11px;
}

/* Collection items */
.coll-item {
  user-select: none;
}

.coll-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 8px;
  margin: 1px 0;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
}

.coll-header:hover {
  background: var(--n-hover-color);
}

.coll-arrow {
  font-size: 8px;
  color: var(--n-text-color-3);
  transition: transform 0.15s;
  display: inline-block;
  width: 10px;
}

.coll-arrow.expanded {
  transform: rotate(90deg);
}

.coll-name {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.coll-count {
  font-size: 11px;
  color: var(--n-text-color-3);
  background: var(--n-action-color);
  padding: 0 5px;
  border-radius: 8px;
  min-width: 16px;
  text-align: center;
}

.coll-del {
  border: none;
  background: transparent;
  color: var(--n-text-color-3);
  cursor: pointer;
  font-size: 14px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.1s;
}

.coll-header:hover .coll-del {
  opacity: 1;
}

.coll-del:hover {
  background: rgba(208, 48, 80, 0.12);
  color: var(--n-error-color);
}

/* Request items */
.req-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px 4px 22px;
  margin: 1px 0;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
}

.req-item:hover {
  background: var(--n-hover-color);
}

.req-method {
  font-size: 9px;
  font-weight: 700;
  font-family: 'Fira Code', monospace;
  padding: 1px 4px;
  border-radius: 4px;
  min-width: 32px;
  text-align: center;
  flex-shrink: 0;
}

.req-name {
  flex: 1;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.req-del {
  border: none;
  background: transparent;
  color: var(--n-text-color-3);
  cursor: pointer;
  font-size: 12px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  flex-shrink: 0;
}

.req-item:hover .req-del {
  opacity: 1;
}

.req-del:hover {
  background: rgba(208, 48, 80, 0.12);
  color: var(--n-error-color);
}

/* History items */
.hist-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  margin: 1px 0;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
}

.hist-item:hover {
  background: var(--n-hover-color);
}

.hist-method {
  font-size: 9px;
  font-weight: 700;
  font-family: 'Fira Code', monospace;
  padding: 1px 4px;
  border-radius: 4px;
  min-width: 32px;
  text-align: center;
  flex-shrink: 0;
}

.hist-url {
  flex: 1;
  font-size: 11px;
  font-family: 'Fira Code', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hist-time {
  font-size: 10px;
  color: var(--n-text-color-3);
  flex-shrink: 0;
}

/* Footer */
.sidebar-footer {
  display: flex;
  gap: 6px;
  padding: 10px 12px;
  border-top: 1px solid var(--n-border-color);
  flex-shrink: 0;
}

.sidebar-footer .n-select {
  flex: 1;
}

/* Method 颜色类 .m-* 在 global.css(全局共享,深浅主题适配) */
</style>
