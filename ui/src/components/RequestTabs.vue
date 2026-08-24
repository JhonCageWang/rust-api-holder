<script setup lang="ts">
/**
 * 请求 Tab 切换条(浏览器风格)
 *
 * ┌─[GET /users ×]─[POST /login ×]─[POST /upload ×]─[+]─┐
 *
 * 每个 Tab 显示 method + 路径,有未保存修改显示圆点
 * 最后一个 Tab 不能关(关了会变成新建空白 Tab)
 * "+" 按钮永远在最后,新建空白 Tab
 */

import { computed } from 'vue'

import { useTabsStore } from '@/stores/tabs'

const tabsStore = useTabsStore()

const tabs = computed(() => tabsStore.tabs)
const activeId = computed(() => tabsStore.activeId)

function activate(id: string): void {
  tabsStore.activate(id)
}

function close(id: string, e: MouseEvent): void {
  e.stopPropagation() // 防止冒泡触发激活
  // 简化:暂不做"未保存"提示(W7+ 再加)
  tabsStore.closeTab(id)
}

function create(): void {
  tabsStore.createTab()
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
  </div>
</template>

<style scoped>
.request-tabs-bar {
  display: flex;
  align-items: stretch;
  gap: 4px;
  background: var(--n-tab-color, #f5f5f5);
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
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
  border: 1px solid transparent;
  border-radius: 4px;
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
  background: rgba(0, 0, 0, 0.04);
}

.tab.active {
  background: var(--n-color, #fff);
  border-color: var(--n-border-color);
  color: var(--n-text-color, #1f1f1f);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

/* HTTP 方法标签色 */
.tab-method {
  font-weight: 600;
  font-size: 11px;
  padding: 1px 5px;
  border-radius: 3px;
  font-family: monospace;
}

.m-get    { color: #18a058; background: rgba(24, 160, 88, 0.1); }
.m-post   { color: #2080f0; background: rgba(32, 128, 240, 0.1); }
.m-put    { color: #f0a020; background: rgba(240, 160, 32, 0.1); }
.m-patch  { color: #9b59b6; background: rgba(155, 89, 182, 0.1); }
.m-delete { color: #d03050; background: rgba(208, 48, 80, 0.1); }
.m-head,
.m-options { color: #707070; background: rgba(112, 112, 112, 0.1); }

.tab-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: 'Fira Code', monospace;
  font-size: 12px;
}

.dirty-dot {
  color: #f0a020;
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
  color: #999;
  font-size: 14px;
  line-height: 1;
  flex-shrink: 0;
}

.close:hover {
  background: rgba(0, 0, 0, 0.08);
  color: #d03050;
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
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  color: var(--n-text-color-2);
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  line-height: 1;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}

.new-tab:hover {
  background: var(--n-color, #fff);
  color: #18a058;
  border-color: #18a058;
}

.new-tab:active {
  background: rgba(24, 160, 88, 0.08);
}
</style>