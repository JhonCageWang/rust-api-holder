<script setup lang="ts">
import { computed, onMounted, ref, watchEffect } from 'vue'
import { darkTheme, lightTheme, useOsTheme } from 'naive-ui'
import type { GlobalThemeOverrides } from 'naive-ui'
import { useAppStore } from '@/stores/app'
import Sidebar from '@/components/Sidebar.vue'

const appStore = useAppStore()

// ─── 侧边栏宽度拖动 ─────────────────────────────────────
const WIDTH_KEY = 'api-holder-sidebar-width'
const MIN_W = 200
const MAX_W = 480

const sidebarWidth = ref(
  Math.min(
    MAX_W,
    Math.max(MIN_W, Number(localStorage.getItem(WIDTH_KEY)) || 260),
  ),
)
const resizing = ref(false)

function startResize(e: MouseEvent): void {
  resizing.value = true
  const startX = e.clientX
  const startW = sidebarWidth.value
  const onMove = (ev: MouseEvent): void => {
    sidebarWidth.value = Math.min(MAX_W, Math.max(MIN_W, startW + ev.clientX - startX))
  }
  const onUp = (): void => {
    resizing.value = false
    localStorage.setItem(WIDTH_KEY, String(sidebarWidth.value))
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }
  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
}

// 跟随系统主题
const osTheme = useOsTheme()
const theme = computed(() => (osTheme.value === 'dark' ? darkTheme : null))

// 全局圆润化
const themeOverrides: GlobalThemeOverrides = {
  common: {
    borderRadius: '8px',
    borderRadiusSmall: '6px',
    fontFamilyMono: "'Fira Code', 'Cascadia Code', 'Consolas', monospace",
  },
}

// 把 naive 主题变量同步到 <html> 的 CSS 变量,
// 组件 scoped 样式和 teleport 到 body 的弹层都能用 var(--n-*)
watchEffect(() => {
  const isDark = osTheme.value === 'dark'
  const common = (isDark ? darkTheme : lightTheme).common ?? {}
  const root = document.documentElement
  for (const [k, v] of Object.entries(common)) {
    const kebab = k.replace(/[A-Z]/g, (m) => `-${m.toLowerCase()}`)
    root.style.setProperty(`--n-${kebab}`, String(v))
  }
  root.style.colorScheme = isDark ? 'dark' : 'light'
})

onMounted(() => {
  appStore.checkBackend()
})
</script>

<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <div class="app" :class="{ resizing }">
            <Sidebar :style="{ width: sidebarWidth + 'px' }" />
            <div
              class="sizer"
              :class="{ active: resizing }"
              title="拖动调整侧栏宽度"
              @mousedown="startResize"
            />
            <main class="app-main">
              <router-view />
            </main>
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
  overflow: hidden;
  background: var(--n-body-color);
  color: var(--n-text-color-1);
  transition: background-color 0.2s;
}

/* 拖动时禁止选中文字,光标统一 */
.app.resizing {
  user-select: none;
  cursor: col-resize;
}

/* 分隔条:6px 热区压在边框上,平时只显示 1px 细线 */
.sizer {
  width: 6px;
  margin-left: -3px;
  flex-shrink: 0;
  cursor: col-resize;
  position: relative;
  z-index: 5;
}

.sizer::after {
  content: '';
  position: absolute;
  left: 2px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--n-border-color);
  transition: width 0.12s, background-color 0.12s;
}

.sizer:hover::after,
.sizer.active::after {
  left: 1px;
  width: 3px;
  background: var(--n-primary-color);
}

.app-main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
