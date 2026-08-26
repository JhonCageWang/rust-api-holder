<script setup lang="ts">
/**
 * 集合(Collection)管理视图
 *
 * 显示所有 collection 列表,支持创建 / 重命名 / 删除。
 * 每个 collection 显示包含的请求数。
 */

import { onMounted, ref } from 'vue'
import { useDialog, useMessage } from 'naive-ui'

import { invokeT } from '@/composables/useInvoke'
import type { Collection } from '@/types/api'

const message = useMessage()
const dialog = useDialog()

// ─── State ────────────────────────────────────────────
const collections = ref<Collection[]>([])
const requestCounts = ref<Record<string, number>>({})
const loading = ref(false)
const showCreate = ref(false)
const newName = ref('')
const newDescription = ref('')

const showRename = ref(false)
const renameTarget = ref<Collection | null>(null)
const renameValue = ref('')

// ─── Lifecycle ───────────────────────────────────────
onMounted(load)

// ─── Actions ─────────────────────────────────────────
async function load() {
  loading.value = true
  try {
    collections.value = await invokeT('list_collections', undefined)
    // 每个 collection 的请求数
    for (const c of collections.value) {
      if (!(c.id in requestCounts.value)) {
        try {
          requestCounts.value[c.id] = await invokeT(
            'count_collection_requests',
            { id: c.id },
          )
        } catch {
          requestCounts.value[c.id] = 0
        }
      }
    }
  } catch (e) {
    message.error(`加载失败: ${(e as Error).message}`)
  } finally {
    loading.value = false
  }
}

async function create() {
  const name = newName.value.trim()
  if (!name) {
    message.warning('请输入集合名称')
    return
  }
  try {
    await invokeT('create_collection', {
      name,
      description: newDescription.value.trim() || null,
      parentId: null,
    })
    message.success('集合已创建')
    showCreate.value = false
    newName.value = ''
    newDescription.value = ''
    requestCounts.value = {}
    await load()
  } catch (e) {
    message.error(`创建失败: ${(e as Error).message}`)
  }
}

function rename(c: Collection) {
  renameTarget.value = c
  renameValue.value = c.name
  showRename.value = true
}

async function confirmRename() {
  const target = renameTarget.value
  if (!target) return
  const name = renameValue.value.trim()
  if (!name) {
    message.warning('请输入名称')
    return
  }
  if (name === target.name) {
    showRename.value = false
    return
  }
  try {
    await invokeT('rename_collection', { id: target.id, newName: name })
    showRename.value = false
    renameTarget.value = null
    message.success('已重命名')
    await load()
  } catch (e) {
    message.error(`重命名失败: ${(e as Error).message}`)
  }
}

function remove(c: Collection) {
  const n = requestCounts.value[c.id] ?? 0
  dialog.warning({
    title: '删除集合',
    content: `确定删除集合 "${c.name}"?\n里面的 ${n} 个请求也会被删除!`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await invokeT('delete_collection', { id: c.id })
        message.success('已删除')
        delete requestCounts.value[c.id]
        await load()
      } catch (e) {
        message.error(`删除失败: ${(e as Error).message}`)
      }
    },
  })
}
</script>

<template>
  <div class="collections-view">
    <header class="header">
      <h2>📁 集合</h2>
      <n-button type="primary" @click="showCreate = true">+ 新建集合</n-button>
    </header>

    <div v-if="loading && collections.length === 0" class="loading">
      加载中...
    </div>

    <n-empty
      v-else-if="collections.length === 0"
      description="还没有集合"
      style="margin-top: 80px"
    >
      <template #extra>
        <n-button type="primary" @click="showCreate = true">+ 新建第一个集合</n-button>
      </template>
    </n-empty>

    <div v-else class="list">
      <div v-for="c in collections" :key="c.id" class="item">
        <div class="info">
          <div class="name">📁 {{ c.name }}</div>
          <div v-if="c.description" class="desc">{{ c.description }}</div>
          <div class="meta">
            {{ requestCounts[c.id] ?? 0 }} 个请求 ·
            更新于 {{ new Date(c.updated_at).toLocaleString() }}
          </div>
        </div>
        <n-space>
          <n-button size="small" @click="rename(c)">重命名</n-button>
          <n-button size="small" type="error" @click="remove(c)">删除</n-button>
        </n-space>
      </div>
    </div>

    <!-- 新建集合弹窗 -->
    <n-modal
      v-model:show="showCreate"
      preset="card"
      title="新建集合"
      style="max-width: 500px"
    >
      <n-space vertical>
        <n-input
          v-model:value="newName"
          placeholder="集合名称"
          @keydown.enter="create"
        />
        <n-input
          v-model:value="newDescription"
          placeholder="描述(可选)"
          type="textarea"
          :rows="3"
        />
      </n-space>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showCreate = false">取消</n-button>
          <n-button type="primary" @click="create">创建</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 重命名集合弹窗 -->
    <n-modal
      v-model:show="showRename"
      preset="card"
      title="重命名集合"
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
  </div>
</template>

<style scoped>
.collections-view {
  height: 100%;
  padding: 16px 24px;
  overflow-y: auto;
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

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border: 1px solid var(--n-border-color);
  border-radius: 6px;
  background: var(--n-color);
  transition: background 0.15s;
}

.item:hover {
  background: var(--n-hover-color, rgba(0, 0, 0, 0.02));
}

.info {
  flex: 1;
  min-width: 0;
}

.name {
  font-weight: 600;
  font-size: 14px;
}

.desc {
  font-size: 12px;
  color: var(--n-text-color-3);
  margin-top: 2px;
}

.meta {
  font-size: 11px;
  color: var(--n-text-color-3);
  margin-top: 4px;
}
</style>