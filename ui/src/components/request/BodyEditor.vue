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
import { computed, watch } from 'vue'
import type { KeyValue, RequestBody } from '@/types/api'
import KeyValueEditor from './KeyValueEditor.vue'

const props = defineProps<{ modelValue: RequestBody }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: RequestBody): void
}>()

type BodyType = 'none' | 'json' | 'form' | 'raw'

// 缓存每种类型的内容,切换类型时不丢数据
const bodyCache = {
  json: '',
  rawContent: '',
  rawContentType: 'text/plain',
  formFields: [] as KeyValue[],
}

// 外部加载请求时同步缓存
watch(
  () => props.modelValue,
  (body) => {
    if (body.type === 'json') bodyCache.json = body.content ?? ''
    else if (body.type === 'raw') {
      bodyCache.rawContent = body.content ?? ''
      bodyCache.rawContentType = body.content_type ?? 'text/plain'
    } else if (body.type === 'form') bodyCache.formFields = body.fields ?? []
  },
  { immediate: true },
)

function setType(type: BodyType): void {
  const old = props.modelValue

  // 切换前保存当前内容
  if (old.type === 'json') bodyCache.json = old.content ?? ''
  else if (old.type === 'raw') {
    bodyCache.rawContent = old.content ?? ''
    bodyCache.rawContentType = old.content_type ?? 'text/plain'
  } else if (old.type === 'form') bodyCache.formFields = old.fields ?? []

  // 从缓存恢复目标类型
  switch (type) {
    case 'none':
      emit('update:modelValue', { type: 'none' })
      return
    case 'json':
      emit('update:modelValue', { type: 'json', content: bodyCache.json })
      return
    case 'form':
      emit('update:modelValue', { type: 'form', fields: bodyCache.formFields })
      return
    case 'raw':
      emit('update:modelValue', {
        type: 'raw',
        content: bodyCache.rawContent,
        content_type: bodyCache.rawContentType,
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
          class="fill-area"
          :value="jsonContent"
          placeholder='{ "key": "value" }'
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
          class="fill-area"
          :value="rawContent"
          placeholder="raw body content..."
          :input-props="{ autocomplete: 'off' }"
          @update:value="(v: string) => patch({ content: v })"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
/* 撑满 tab-pane:radio 固定,内容区占满剩余高度 */
.body-editor {
  width: 100%;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.body-content {
  margin-top: 12px;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

/* textarea 填满剩余高度(n-input 内部:root > wrapper > textarea) */
.fill-area {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.fill-area :deep(.n-input-wrapper) {
  flex: 1;
  min-height: 0;
}

.fill-area :deep(.n-input__textarea-el) {
  height: 100%;
}
</style>
