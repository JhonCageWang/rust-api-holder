import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

// Tauri 期望一个固定的端口,且需要禁用 HMR 之外的源
const host = process.env.TAURI_DEV_HOST

export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  // Vite 5 / Tauri 2 推荐配置
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 监听 src-tauri 变动时自动重启(可选)
      ignored: ['**/src-tauri/**'],
    },
  },
}))