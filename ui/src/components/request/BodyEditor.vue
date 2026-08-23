<script setup lang="ts">
/**
 * Body 编辑器
 *
 * 通过 radio 切换 4 种 body 类型,根据类型显示对应的子编辑器:
 * - none:    什么都不显示
 * - json:    textarea(放 JSON 字符串)
 * - form:    复用 KeyValueEditor 编辑 fields
 * - raw:     Content-Type 输入框 + textarea
 */
import { computed } from 'vue'
import type { KeyValue, RequestBody } from '@/types/api'
import KeyValueEditor from './KeyValueEditor.vue'

const props = defineProps<{ modelValue: RequestBody }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: RequestBody): void
}>()

// 切换 body 类型时,要构造一个新的 body(因为每种变体的字段不一样)
type BodyType = 'none' | 'json' | 'form' | 'raw'

function setType(type: BodyType): void {
  switch (type) {
    case 'none':
      emit('update:modelValue', { type: 'none' })
      return
    case 'json':
      emit('update:modelValue', { type: 'json', content: '' })
      return
    case 'form':
      emit('update:modelValue', { type: 'form', fields: [] })
      return
    case 'raw':
      emit('update:modelValue', {
        type: 'raw',
        content: '',
        content_type: 'text/plain',
      })
      return
  }
}

// 採 Partial<RequestBody> 而不是 K extends keyof<RequestBody>,
// 因为 'fields' / 'content' 这些字段不是所有变体都有的,
// keyof RequestBody 在 union 上取交集,会丢掉这些局部字段。
function patch(updates: Partial<RequestBody>): void {
  emit('update:modelValue', { ...props.modelValue, ...updates } as RequestBody)
}

// ─── 局部别名 + early return + ?? '' 模式 ─────────────────────────────
// vue-tsc 补的 reactive 包装会推所有 props 字段为 `T | undefined` (为了 `?:` 兼容)，
// 所以访问 body.content 后 是 `string | undefined`。需要 `?? ''` 传给对变类型签名
// ComputedGetter<string>。

const jsonContent = computed<string>(() => {
  const body = props.modelValue
  if (body.type !== 'json') return ''
  return body.content ?? ''
})

const rawContent = computed<string>(() => {
  const body = props.modelValue
  if (body.type !== 'raw') return ''
  return body.content ?? ''
})

const rawContentType = computed<string>(() => {
  const body = props.modelValue
  if (body.type !== 'raw') return ''
  return body.content_type ?? ''
})

const formFields = computed<KeyValue[]>(() => {
  const body = props.modelValue
  if (body.type !== 'form') return []
  return body.fields ?? []
})

function setFormFields(v: KeyValue[]): void {
  if (props.modelValue.type === 'form') patch({ fields: v })
}
</script>

<template>
  <div class="body-editor">
    <n-radio-group
      :value="modelValue.type"
      @update:value="(v: string) => setType(v as BodyType)"
    >
      <n-radio-button value="none">none</n-radio-button>
      <n-radio-button value="json">JSON</n-radio-button>
      <n-radio-button value="form">form-data</n-radio-button>
      <n-radio-button value="raw">raw</n-radio-button>
    </n-radio-group>

    <div class="body-content">
      <!-- none: nothing to show -->

      <template v-if="modelValue.type === 'json'">
        <n-input
          type="textarea"
          :value="jsonContent"
          placeholder='{ "key": "value" }'
          :autosize="{ minRows: 8, maxRows: 20 }"
          :input-props="{ autocomplete: 'off' }"
          @update:value="(v: string) => patch({ content: v })"
        />
      </template>

      <template v-else-if="modelValue.type === 'form'">
        <KeyValueEditor
          :model-value="formFields"
          @update:model-value="setFormFields"
        />
      </template>

      <template v-else-if="modelValue.type === 'raw'">
        <n-space align="center" style="margin-bottom: 8px">
          <n-text depth="3">Content-Type:</n-text>
          <n-input
            :value="rawContentType"
            placeholder="text/plain"
            size="small"
            style="width: 220px"
            :input-props="{ autocomplete: 'off' }"
            @update:value="(v: string) => patch({ content_type: v })"
          />
        </n-space>
        <n-input
          type="textarea"
          :value="rawContent"
          placeholder="raw body content..."
          :autosize="{ minRows: 6, maxRows: 20 }"
          :input-props="{ autocomplete: 'off' }"
          @update:value="(v: string) => patch({ content: v })"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
.body-editor {
  width: 100%;
}

.body-content {
  margin-top: 12px;
}
</style>
