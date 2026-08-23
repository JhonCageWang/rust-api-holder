<script setup lang="ts">
/**
 * 通用 KeyValue 列表编辑器
 *
 * 用于:
 * - Query Params
 * - Headers
 * - Form Body 的 fields
 *
 * 完全受控组件(parent 持有 modelValue 状态,通过 v-model 双向绑定)。
 */
import type { KeyValue } from '@/types/api'

const props = defineProps<{ modelValue: KeyValue[] }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: KeyValue[]): void
}>()

// 改某一行(只返回新数组,immutable,触发响应式更新)
function updateRow(idx: number, patch: Partial<KeyValue>): void {
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

// 删某一行
function removeRow(idx: number): void {
  emit('update:modelValue', props.modelValue.filter((_, i) => i !== idx))
}

// 一键清空(给"空状态"用)
function clearAll(): void {
  emit('update:modelValue', [])
}
</script>

<template>
  <div class="kv-editor">
    <!-- 空状态 -->
    <n-empty
      v-if="modelValue.length === 0"
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
        <tr v-for="(kv, idx) in modelValue" :key="idx">
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
              placeholder="Value"
              size="small"
              :input-props="{ autocomplete: 'off' }"
              @update:value="(v: string) => updateRow(idx, { value: v })"
            />
          </td>
          <td>
            <n-button
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
    <n-space v-if="modelValue.length > 0" style="margin-top: 12px">
      <n-button size="small" tertiary @click="addRow">+ 添加</n-button>
      <n-button size="small" quaternary @click="clearAll">清空</n-button>
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
</style>
