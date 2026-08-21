/**
 * 应用入口
 *
 * 启动顺序:
 * 1. 注册 Naive UI 全局组件(用 app.component)
 * 2. 创建 Pinia 状态管理
 * 3. 注册 Vue Router
 * 4. 挂载 App.vue
 */

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'

// Naive UI 样式 + 字体
import 'vfonts/Lato.css'
import 'vfonts/FiraCode.css'
import './styles/global.css'

// 按需 import 用到的 Naive UI 组件
import {
  // 基础
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  // 布局
  NTabs,
  NTabPane,
  // 文本
  NH2,
  NP,
  NUl,
  NLi,
  NDivider,
  NText,
  NTag,
} from 'naive-ui'

const app = createApp(App)

// 注册 Naive UI 组件(app.component 是正确的注册方式)
const naiveComponents = {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  NTabs,
  NTabPane,
  NH2,
  NP,
  NUl,
  NLi,
  NDivider,
  NText,
  NTag,
}
for (const [name, comp] of Object.entries(naiveComponents)) {
  app.component(name, comp)
}

// 全局插件
app.use(createPinia())
app.use(router)

app.mount('#app')