<script setup lang="ts">
/**
 * 环境管理弹框
 *
 * 以 NModal 弹框形式展示,不打断主区域的请求 Tab。
 * - 显示环境列表,标记激活的
 * - 创建 / 重命名 / 删除环境
 * - 切换激活(原子操作,影响所有请求的 {{var}} 插值)
 * - 编辑环境变量(KV 对 + enabled toggle)
 * - 用 `bulk_replace_variables` 一次性保存
 */

import { computed, ref, watch } from 'vue'
import { useDialog, useMessage } from 'naive-ui'

import { invokeT } from '@/composables/useInvoke'
import { useAppStore } from '@/stores/app'
import type { Environment, Variable } from '@/types/api'

const show = defineModel<boolean>({ default: false })

const message = useMessage()
const dialog = useDialog()
const appStore = useAppStore()

// ─── State ──────────────────────────────────────────────
const envs = ref<Environment[]>([])
const varsByEnv = ref<Record<string, Variable[]>>({})
const activeEnvId = ref<string | null>(null)
const loading = ref(false)

const showCreate = ref(false)
const newEnvName = ref('')

const showRename = ref(false)
const renameTarget = ref<Environment | null>(null)
const renameValue = ref('')

const selectedEnvId = ref<string | null>(null)

// ─── Computed ───────────────────────────────────────────
const selectedEnv = computed(() =>
  envs.value.find((e) => e.id === selectedEnvId.value),
)

const selectedVars = computed<Variable[]>({
  get: () => varsByEnv.value[selectedEnvId.value ?? ''] ?? [],
  set: (v) => {
    if (selectedEnvId.value) varsByEnv.value[selectedEnvId.value] = v
  },
})

// ─── 弹框打开时加载 ────────────────────────────────────
watch(show, (v) => {
  if (v) load()
})

// ─── Actions ────────────────────────────────────────────
async function load() {
  loading.value = true
  try {
    envs.value = await invokeT('list_environments', undefined)
    const active = await invokeT('get_active_environment', undefined)
    activeEnvId.value = active?.id ?? null
    if (!selectedEnvId.value && envs.value.length > 0) {
      selectEnv(envs.value[0].id)
    }
  } catch (e) {
    message.error(`加载失败: ${(e as Error).message}`)
  } finally {
    loading.value = false
  }
}

async function selectEnv(id: string) {
  selectedEnvId.value = id
  if (!(id in varsByEnv.value)) {
    try {
      varsByEnv.value[id] = await invokeT('list_variables', {
        environmentId: id,
      })
    } catch (e) {
      message.error(`加载变量失败: ${(e as Error).message}`)
      varsByEnv.value[id] = []
    }
  }
}

async function createEnv() {
  const name = newEnvName.value.trim()
  if (!name) {
    message.warning('请输入环境名')
    return
  }
  try {
    const env = await invokeT('create_environment', { name })
    showCreate.value = false
    newEnvName.value = ''
    await load()
    selectEnv(env.id)
    appStore.bumpSidebar()
    message.success('环境已创建')
  } catch (e) {
    message.error(`创建失败: ${(e as Error).message}`)
  }
}

function renameEnv(e: Environment) {
  renameTarget.value = e
  renameValue.value = e.name
  showRename.value = true
}

async function confirmRename() {
  const target = renameTarget.value
  if (!target) return
  const name = renameValue.value.trim()
  if (!name) {
    message.warning('请输入环境名')
    return
  }
  if (name === target.name) {
    showRename.value = false
    return
  }
  try {
    await invokeT('rename_environment', { id: target.id, newName: name })
    showRename.value = false
    renameTarget.value = null
    await load()
    appStore.bumpSidebar()
    message.success('已重命名')
  } catch (err) {
    message.error(`重命名失败: ${(err as Error).message}`)
  }
}

function deleteEnv(e: Environment) {
  const n = varsByEnv.value[e.id]?.length ?? 0
  dialog.warning({
    title: '删除环境',
    content: `确定删除环境 "${e.name}"?里面的 ${n} 个变量会一起删除!`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await invokeT('delete_environment', { id: e.id })
        delete varsByEnv.value[e.id]
        if (selectedEnvId.value === e.id) selectedEnvId.value = null
        await load()
        appStore.bumpSidebar()
        message.success('已删除')
      } catch (err) {
        message.error(`删除失败: ${(err as Error).message}`)
      }
    },
  })
}

async function activate(e: Environment) {
  try {
    await invokeT('set_active_environment', { id: e.id })
    await load()
    appStore.bumpSidebar()
    message.success(`已激活 ${e.name}`)
  } catch (err) {
    message.error(`切换失败: ${(err as Error).message}`)
  }
}

// 变量编辑
function addVar() {
  if (!selectedEnvId.value) return
  const newVar: Variable = {
    id: `tmp-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
    environment_id: selectedEnvId.value,
    key: '',
    value: '',
    enabled: true,
  }
  varsByEnv.value[selectedEnvId.value] = [...selectedVars.value, newVar]
}

function removeVar(idx: number) {
  if (!selectedEnvId.value) return
  const list = [...selectedVars.value]
  list.splice(idx, 1)
  varsByEnv.value[selectedEnvId.value] = list
}

async function saveVars() {
  if (!selectedEnvId.value) return
  const valid = selectedVars.value.filter((v) => v.key.trim() !== '')
  if (valid.length !== selectedVars.value.length) {
    message.warning('已跳过空 key 的变量')
  }
  try {
    const normalized = valid.map((v) => ({
      ...v,
      id: v.id.startsWith('tmp-') ? crypto.randomUUID() : v.id,
    }))
    await invokeT('bulk_replace_variables', {
      environmentId: selectedEnvId.value,
      variables: normalized,
    })
    varsByEnv.value[selectedEnvId.value] = normalized
    message.success('变量已保存')
  } catch (e) {
    message.error(`保存失败: ${(e as Error).message}`)
  }
}
</script>

<template>
  <n-modal v-model:show="show" preset="card" title="🌍 环境管理" style="width: 820px; max-width: 90vw">
    <div v-if="loading && envs.length === 0" class="loading">加载中...</div>

    <div v-else class="layout">
      <!-- 左:环境列表 -->
      <div class="env-list">
        <div class="env-list-header">
          <span class="env-list-title">环境列表</span>
          <n-button size="tiny" type="primary" @click="showCreate = true">+</n-button>
        </div>
        <n-empty
          v-if="envs.length === 0"
          description="还没有环境"
          size="small"
        />
        <div
          v-for="e in envs"
          :key="e.id"
          class="env-item"
          :class="{ active: e.id === selectedEnvId }"
          @click="selectEnv(e.id)"
        >
          <div class="env-info">
            <div class="env-name">
              {{ e.is_active ? '🟢' : '⚪' }} {{ e.name }}
            </div>
            <div v-if="e.is_active" class="active-tag">激活中</div>
          </div>
          <n-space size="small" :wrap-item="false">
            <n-button
              v-if="!e.is_active"
              size="tiny"
              @click.stop="activate(e)"
            >
              激活
            </n-button>
            <n-button size="tiny" @click.stop="renameEnv(e)">改名</n-button>
            <n-button size="tiny" type="error" @click.stop="deleteEnv(e)">
              删
            </n-button>
          </n-space>
        </div>
      </div>

      <!-- 右:变量编辑 -->
      <div class="var-panel">
        <div v-if="!selectedEnv" class="empty">选择一个环境</div>
        <template v-else>
          <div class="var-header">
            <span>变量 — {{ selectedEnv.name }}</span>
            <n-space>
              <n-button size="small" @click="addVar">+ 添加变量</n-button>
              <n-button size="small" type="primary" @click="saveVars">
                保存
              </n-button>
            </n-space>
          </div>

          <div v-if="selectedVars.length === 0" class="empty">
            还没有变量
          </div>

          <div v-else class="var-list">
            <div v-for="(v, idx) in selectedVars" :key="v.id" class="var-row">
              <n-input
                v-model:value="v.key"
                placeholder="KEY"
                size="small"
                style="width: 140px; font-family: monospace"
              />
              <n-input
                v-model:value="v.value"
                placeholder="value"
                size="small"
                style="flex: 1; font-family: monospace"
              />
              <n-checkbox v-model:checked="v.enabled">启用</n-checkbox>
              <n-button
                size="small"
                type="error"
                @click="removeVar(idx)"
              >
                ×
              </n-button>
            </div>
            <div class="hint">
              💡 使用方式:URL/Headers 里写
              <code v-pre>{{key}}</code>,发请求时自动替换为 value。
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- 新建环境弹窗 -->
    <n-modal
      v-model:show="showCreate"
      preset="card"
      title="新建环境"
      style="max-width: 400px"
    >
      <n-input
        v-model:value="newEnvName"
        placeholder="环境名(如 Dev / Prod)"
        @keydown.enter="createEnv"
      />
      <template #footer>
        <n-space justify="end">
          <n-button @click="showCreate = false">取消</n-button>
          <n-button type="primary" @click="createEnv">创建</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 重命名环境弹窗 -->
    <n-modal
      v-model:show="showRename"
      preset="card"
      title="重命名环境"
      style="max-width: 400px"
    >
      <n-input
        v-model:value="renameValue"
        placeholder="新名称"
        @keydown.enter="confirmRename"
      />
      <template #footer>
        <n-space justify="end">
          <n-button @click="showRename = false">取消</n-button>
          <n-button type="primary" @click="confirmRename">保存</n-button>
        </n-space>
      </template>
    </n-modal>
  </n-modal>
</template>

<style scoped>
.loading {
  text-align: center;
  padding: 40px;
  color: var(--n-text-color-3);
}

.layout {
  display: grid;
  grid-template-columns: 220px 1fr;
  gap: 12px;
  max-height: 60vh;
  min-height: 300px;
}

.env-list,
.var-panel {
  border: 1px solid var(--n-border-color);
  border-radius: 6px;
  padding: 8px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.env-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2px 4px 6px;
  border-bottom: 1px solid var(--n-border-color);
  margin-bottom: 4px;
}

.env-list-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--n-text-color-3);
}

.env-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.env-item:hover {
  background: var(--n-hover-color);
}

.env-item.active {
  background: rgba(24, 160, 88, 0.12);
}

.env-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.env-name {
  font-weight: 500;
  font-size: 13px;
  flex: 1;
}

.active-tag {
  font-size: 10px;
  background: var(--n-primary-color);
  color: white;
  padding: 1px 6px;
  border-radius: 8px;
}

.var-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 4px 8px;
  font-weight: 500;
}

.empty {
  text-align: center;
  padding: 40px;
  color: var(--n-text-color-3);
  font-size: 13px;
}

.var-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 4px;
}

.var-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.hint {
  font-size: 12px;
  color: var(--n-text-color-3);
  margin-top: 8px;
  padding: 8px;
  background: var(--n-action-color);
  border-radius: 6px;
}

code {
  background: var(--n-action-color);
  padding: 1px 6px;
  border-radius: 4px;
  font-family: 'Fira Code', monospace;
}
</style>
