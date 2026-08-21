/**
 * 应用入口
 *
 * 启动顺序:
 * 1. 创建 Pinia 状态管理
 * 2. 注册 Vue Router
 * 3. 加载 Naive UI(全局)
 * 4. 挂载 App.vue
 */

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'

// Naive UI 全局样式
import 'vfonts/Lato.css'
import 'vfonts/FiraCode.css'
import './styles/global.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')