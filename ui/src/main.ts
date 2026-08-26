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
  // 基础 / Provider
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  // 反馈
  NSpin,
  NAlert,
  NEmpty,
  // 基础控件
  NButton,
  NCheckbox,
  // 表单
  NInput,
  NSelect,
  NRadio,
  NRadioButton,
  NRadioGroup,
  // 数据展示
  NCode,
  NTable,
  // 布局
  NTabs,
  NTabPane,
  NSpace,
  // 弹层
  NModal,
  NTooltip,
  NPopconfirm,
  NDropdown,
  NCollapse,
  NCollapseItem,
  // 滚动
  NScrollbar,
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
  NSpin,
  NAlert,
  NEmpty,
  NButton,
  NCheckbox,
  NInput,
  NSelect,
  NRadio,
  NRadioButton,
  NRadioGroup,
  NCode,
  NTable,
  NTabs,
  NTabPane,
  NSpace,
  NModal,
  NTooltip,
  NPopconfirm,
  NDropdown,
  NCollapse,
  NCollapseItem,
  NScrollbar,
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