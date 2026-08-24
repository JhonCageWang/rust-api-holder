<script setup lang="ts">
/**
 * 环境(Environment)管理视图
 *
 * - 显示环境列表,标记激活的
 * - 创建 / 重命名 / 删除环境
 * - 切换激活(原子操作,影响所有请求的 {{var}} 插值)
 * - 编辑环境变量(KV 对 + enabled toggle)
 * - 用 `bulk_replace_variables` 一次性保存(避免频繁单条更新)
 */

import { computed, onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'

import { invokeT } from '@/composables/useInvoke'
import type { Environment, Variable } from '@/types/api'

const message = useMessage()

// ─── State ──────────────────────────────────────────────
const envs = ref<Environment[]>([])
const varsByEnv = ref<Record<string, Variable[]>>({})
const activeEnvId = ref<string | null>(null)
const loading = ref(false)

const showCreate = ref(false)
const newEnvName = ref('')

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

// ─── Lifecycle ──────────────────────────────────────────
onMounted(load)

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
        environment_id: id,
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
    message.success('环境已创建')
  } catch (e) {
    message.error(`创建失败: ${(e as Error).message}`)
  }
}

async function renameEnv(e: Environment) {
  const name = window.prompt('新名称', e.name)
  if (!name || name === e.name) return
  try {
    await invokeT('rename_environment', { id: e.id, new_name: name })
    await load()
    message.success('已重命名')
  } catch (err) {
    message.error(`重命名失败: ${(err as Error).message}`)
  }
}

async function deleteEnv(e: Environment) {
  const n = varsByEnv.value[e.id]?.length ?? 0
  const ok = window.confirm(
    `确定删除环境 "${e.name}"?\n里面的 ${n} 个变量会一起删除!`,
  )
  if (!ok) return
  try {
    await invokeT('delete_environment', { id: e.id })
    delete varsByEnv.value[e.id]
    if (selectedEnvId.value === e.id) selectedEnvId.value = null
    await load()
    message.success('已删除')
  } catch (err) {
    message.error(`删除失败: ${(err as Error).message}`)
  }
}

async function activate(e: Environment) {
  try {
    await invokeT('set_active_environment', { id: e.id })
    activeEnvId.value = e.id
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
  // 过滤掉空 key
  const valid = selectedVars.value.filter((v) => v.key.trim() !== '')
  if (valid.length !== selectedVars.value.length) {
    message.warning('已跳过空 key 的变量')
  }
  try {
    // 给新行生成稳定 id(后端需要 id 字段)
    const normalized = valid.map((v) => ({
      ...v,
      id: v.id.startsWith('tmp-') ? crypto.randomUUID() : v.id,
    }))
    await invokeT('bulk_replace_variables', {
      environment_id: selectedEnvId.value,
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
  <div class="env-view">
    <header class="header">
      <h2>🌍 环境</h2>
      <n-button type="primary" @click="showCreate = true">+ 新建环境</n-button>
    </header>

    <div v-if="loading && envs.length === 0" class="loading">加载中...</div>

    <div v-else class="layout">
      <!-- 左:环境列表 -->
      <div class="env-list">
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
              <code>{{ '{{key}}' }}</code>,发请求时自动替换为 value。
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
  </div>
</template>

<style scoped>
.env-view {
  height: 100%;
  padding: 16px 24px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
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

.layout {
  display: grid;
  grid-template-columns: 240px 1fr;
  gap: 16px;
  flex: 1;
  min-height: 0;
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

.env-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.env-item:hover {
  background: var(--n-hover-color, rgba(0, 0, 0, 0.04));
}

.env-item.active {
  background: rgba(24, 160, 88, 0.1);
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
  background: #18a058;
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
  color: #999;
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
  background: var(--n-hover-color, rgba(0, 0, 0, 0.02));
  border-radius: 4px;
}

code {
  background: var(--n-border-color);
  padding: 1px 6px;
  border-radius: 3px;
  font-family: 'Fira Code', monospace;
}
</style>