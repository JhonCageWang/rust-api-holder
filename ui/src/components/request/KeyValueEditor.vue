<script setup lang="ts">
/**
 * 通用 KeyValue 列表编辑器
 *
 * 用于:
 * - Query Params
 * - Headers(可传 presets 显示预填行)
 * - Form Body 的 fields
 *
 * 完全受控组件(parent 持有 modelValue 状态,通过 v-model 双向绑定)。
 *
 * presets: 预填占位行(如 Authorization),默认不勾选、不进 modelValue。
 * 用户编辑或勾选某行预填行时,该行才落入 modelValue(填了值会自动勾选)。
 * key 已存在(忽略大小写)的预填行不再显示。
 */
import { computed } from 'vue'
import type { KeyValue } from '@/types/api'

type PresetRow = KeyValue & { placeholder?: string }

const props = defineProps<{ modelValue: KeyValue[]; presets?: PresetRow[] }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: KeyValue[]): void
}>()

const ghostRows = computed<PresetRow[]>(() => {
  const existing = new Set(
    props.modelValue.map((kv) => kv.key.trim().toLowerCase()),
  )
  return (props.presets ?? []).filter((p) => !existing.has(p.key.toLowerCase()))
})

const displayRows = computed<PresetRow[]>(() => [
  ...props.modelValue,
  ...ghostRows.value,
])

function isGhost(idx: number): boolean {
  return idx >= props.modelValue.length
}

// 改某一行(只返回新数组,immutable,触发响应式更新)
function updateRow(idx: number, patch: Partial<KeyValue>): void {
  if (isGhost(idx)) {
    const ghost = ghostRows.value[idx - props.modelValue.length]
    const enabled =
      patch.enabled ??
      (patch.value !== undefined ? patch.value.trim() !== '' : ghost.enabled)
    const base: KeyValue = {
      key: ghost.key,
      value: ghost.value,
      enabled: ghost.enabled,
    }
    emit('update:modelValue', [...props.modelValue, { ...base, ...patch, enabled }])
    return
  }
  const next = props.modelValue.map((kv, i) =>
    i === idx ? { ...kv, ...patch } : kv,
  )
  emit('update:modelValue', next)
}

// 加一行
function addRow(): void {
  emit('update:modelValue', [
    ...props.modelValue,
    { key: '', value: '', enabled: true },
  ])
}

// 删某一行(预填行不可删)
function removeRow(idx: number): void {
  if (isGhost(idx)) return
  emit('update:modelValue', props.modelValue.filter((_, i) => i !== idx))
}

// 一键清空(只清真实行,预填行会回来)
function clearAll(): void {
  emit('update:modelValue', [])
}
</script>

<template>
  <div class="kv-editor">
    <!-- 空状态(没有任何行,包括预填行) -->
    <n-empty
      v-if="displayRows.length === 0"
      description="还没有配置项"
      size="small"
    >
      <template #extra>
        <n-button size="small" tertiary @click="addRow">+ 添加一行</n-button>
      </template>
    </n-empty>

    <!-- 列表 -->
    <table v-else class="kv-table">
      <colgroup>
        <col style="width: 40px" />
        <col style="width: 30%" />
        <col />
        <col style="width: 40px" />
      </colgroup>
      <tbody>
        <tr
          v-for="(kv, idx) in displayRows"
          :key="idx"
          :class="{ ghost: isGhost(idx) }"
        >
          <td>
            <n-checkbox
              :checked="kv.enabled"
              @update:checked="(v: boolean) => updateRow(idx, { enabled: v })"
            />
          </td>
          <td>
            <n-input
              :value="kv.key"
              placeholder="Key"
              size="small"
              :input-props="{ autocomplete: 'off' }"
              @update:value="(v: string) => updateRow(idx, { key: v })"
            />
          </td>
          <td>
            <n-input
              :value="kv.value"
              :placeholder="kv.placeholder ?? 'Value'"
              size="small"
              :input-props="{ autocomplete: 'off' }"
              @update:value="(v: string) => updateRow(idx, { value: v })"
            />
          </td>
          <td>
            <n-button
              v-if="!isGhost(idx)"
              quaternary
              circle
              size="small"
              type="error"
              :title="'删除'"
              @click="removeRow(idx)"
            >
              ✕
            </n-button>
          </td>
        </tr>
      </tbody>
    </table>

    <!-- 底部按钮 -->
    <n-space v-if="displayRows.length > 0" style="margin-top: 12px">
      <n-button size="small" tertiary @click="addRow">+ 添加</n-button>
      <n-button v-if="modelValue.length > 0" size="small" quaternary @click="clearAll">
        清空
      </n-button>
    </n-space>
  </div>
</template>

<style scoped>
.kv-editor {
  width: 100%;
}

.kv-table {
  width: 100%;
  border-collapse: collapse;
}

.kv-table td {
  padding: 4px 8px 4px 0;
  vertical-align: middle;
}

/* 预填行:淡显,hover 恢复正常 */
.kv-table tr.ghost {
  opacity: 0.5;
  transition: opacity 0.12s;
}

.kv-table tr.ghost:hover {
  opacity: 1;
}
</style>
