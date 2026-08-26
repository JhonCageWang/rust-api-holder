<script setup lang="ts">
/**
 * 请求 Tab 切换条(浏览器风格)
 *
 * ┌─[GET /users ×]─[POST /login ×]─[POST /upload ×]─[+]─┐
 *
 * 每个 Tab 显示 method + 路径,有未保存修改显示圆点
 * 最后一个 Tab 不能关(关了会变成新建空白 Tab)
 * "+" 按钮永远在最后,新建空白 Tab
 * 右键 Tab 弹出上下文菜单(关闭/关闭其他/关闭左侧/关闭右侧/关闭全部)
 */

import { computed, ref } from 'vue'
import type { DropdownOption } from 'naive-ui'

import { useTabsStore } from '@/stores/tabs'

const tabsStore = useTabsStore()

const tabs = computed(() => tabsStore.tabs)
const activeId = computed(() => tabsStore.activeId)

function activate(id: string): void {
  tabsStore.activate(id)
}

function close(id: string, e: MouseEvent): void {
  e.stopPropagation()
  tabsStore.closeTab(id)
}

function create(): void {
  tabsStore.createTab()
}

// ─── 右键上下文菜单 ────────────────────────────────────
const showMenu = ref(false)
const menuX = ref(0)
const menuY = ref(0)
const menuTabId = ref<string | null>(null)

function onContextMenu(e: MouseEvent, tabId: string): void {
  e.preventDefault()
  menuTabId.value = tabId
  menuX.value = e.clientX
  menuY.value = e.clientY
  showMenu.value = true
}

const menuOptions = computed<DropdownOption[]>(() => {
  const id = menuTabId.value
  if (!id) return []
  const idx = tabs.value.findIndex((t) => t.id === id)
  const total = tabs.value.length
  return [
    { label: '关闭', key: 'close' },
    { label: '关闭其他', key: 'closeOthers', disabled: total <= 1 },
    { label: '关闭左侧', key: 'closeLeft', disabled: idx <= 0 },
    { label: '关闭右侧', key: 'closeRight', disabled: idx === -1 || idx >= total - 1 },
    { label: '关闭全部', key: 'closeAll', disabled: total <= 1 },
  ]
})

function onMenuSelect(key: string): void {
  showMenu.value = false
  const id = menuTabId.value
  if (!id) return
  switch (key) {
    case 'close':
      tabsStore.closeTab(id)
      break
    case 'closeOthers':
      tabsStore.closeOthers(id)
      break
    case 'closeLeft':
      tabsStore.closeLeft(id)
      break
    case 'closeRight':
      tabsStore.closeRight(id)
      break
    case 'closeAll':
      tabsStore.closeAllTabs()
      break
  }
  menuTabId.value = null
}
</script>

<template>
  <div class="request-tabs-bar">
    <div class="tabs-scroll">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        type="button"
        class="tab"
        :class="{ active: tab.id === activeId }"
        @click="activate(tab.id)"
        @contextmenu="onContextMenu($event, tab.id)"
      >
        <span class="tab-method" :class="`m-${tab.request.method.toLowerCase()}`">
          {{ tab.request.method }}
        </span>
        <span class="tab-title">{{ tab.title }}</span>
        <span v-if="tab.isDirty" class="dirty-dot" title="有未保存修改">●</span>
        <!-- × 关闭按钮:最后一个 Tab 也保留(点了会变成新建) -->
        <span class="close" role="button" title="关闭" @click="close(tab.id, $event)">×</span>
      </button>
    </div>
    <button type="button" class="new-tab" title="新建 Tab" @click="create">+</button>

    <n-dropdown
      trigger="manual"
      placement="bottom-start"
      :show="showMenu"
      :x="menuX"
      :y="menuY"
      :options="menuOptions"
      @select="onMenuSelect"
      @clickoutside="showMenu = false"
    />
  </div>
</template>

<style scoped>
.request-tabs-bar {
  display: flex;
  align-items: stretch;
  gap: 4px;
  background: var(--n-action-color);
  border-radius: 10px;
  padding: 4px;
  flex-shrink: 0;
}

.tabs-scroll {
  display: flex;
  flex: 1;
  overflow-x: auto;
  gap: 2px;
  /* 隐藏滚动条但保留功能 */
  scrollbar-width: thin;
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: transparent;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  color: var(--n-text-color-2);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  max-width: 220px;
  min-width: 80px;
  transition: background 0.15s;
}

.tab:hover {
  background: var(--n-hover-color);
}

.tab.active {
  background: var(--n-card-color);
  color: var(--n-text-color-1);
  box-shadow: var(--n-box-shadow-1);
}

/* HTTP 方法标签(颜色类 .m-* 在 global.css) */
.tab-method {
  font-weight: 600;
  font-size: 11px;
  padding: 1px 5px;
  border-radius: 4px;
  font-family: 'Fira Code', monospace;
}

.tab-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'Fira Code', monospace;
  font-size: 12px;
}

.dirty-dot {
  color: var(--n-warning-color);
  font-size: 10px;
  line-height: 1;
}

.close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  color: var(--n-text-color-3);
  font-size: 14px;
  line-height: 1;
  flex-shrink: 0;
}

.close:hover {
  background: var(--n-hover-color);
  color: var(--n-error-color);
}

.new-tab {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  min-width: 36px;
  padding: 0 8px;
  flex-shrink: 0;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--n-text-color-2);
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  line-height: 1;
  transition: background 0.15s, color 0.15s;
}

.new-tab:hover {
  background: var(--n-card-color);
  color: var(--n-primary-color);
}

.new-tab:active {
  background: var(--n-hover-color);
}
</style>