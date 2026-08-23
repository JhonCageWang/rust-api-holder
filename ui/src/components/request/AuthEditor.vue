<script setup lang="ts">
/**
 * Auth 编辑器
 *
 * 通过 radio 切换 4 种 auth 类型:
 * - none:    无认证
 * - bearer:  Bearer Token (JWT/OAuth 风格)
 * - basic:   Basic Auth(user/pass)
 * - api_key: 自定义 key,可放 header 或 query
 */
import { computed } from 'vue'
import type { Auth } from '@/types/api'

const props = defineProps<{ modelValue: Auth }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: Auth): void
}>()

type AuthType = 'none' | 'bearer' | 'basic' | 'api_key'

function setType(type: AuthType): void {
  switch (type) {
    case 'none':
      emit('update:modelValue', { type: 'none' })
      return
    case 'bearer':
      emit('update:modelValue', { type: 'bearer', token: '' })
      return
    case 'basic':
      emit('update:modelValue', {
        type: 'basic',
        username: '',
        password: '',
      })
      return
    case 'api_key':
      emit('update:modelValue', {
        type: 'api_key',
        key: '',
        value: '',
        in_header: true,
      })
      return
  }
}

// 採 Partial<Auth> 类型安全的字段覆盖
function patch(updates: Partial<Auth>): void {
  emit('update:modelValue', { ...props.modelValue, ...updates } as Auth)
}

// ─── 局部别名 + early return + ?? '' 模式 ─────────────────────────────
// vue-tsc 补的 reactive 包装会推 props 字段为 `T | undefined`，
// computed 反馈类型 ComputedGetter<string> 要求严格 string，所以用 `?? ''` 收尾。

const bearerToken = computed<string>(() => {
  const a = props.modelValue
  if (a.type !== 'bearer') return ''
  return a.token ?? ''
})
const basicUsername = computed<string>(() => {
  const a = props.modelValue
  if (a.type !== 'basic') return ''
  return a.username ?? ''
})
const basicPassword = computed<string>(() => {
  const a = props.modelValue
  if (a.type !== 'basic') return ''
  return a.password ?? ''
})
const apiKeyName = computed<string>(() => {
  const a = props.modelValue
  if (a.type !== 'api_key') return ''
  return a.key ?? ''
})
const apiKeyValue = computed<string>(() => {
  const a = props.modelValue
  if (a.type !== 'api_key') return ''
  return a.value ?? ''
})
const apiKeyInHeader = computed<boolean>(() => {
  const a = props.modelValue
  if (a.type !== 'api_key') return true
  return a.in_header ?? true
})

function setBearerToken(v: string): void {
  if (props.modelValue.type === 'bearer') patch({ token: v })
}
function setBasicUsername(v: string): void {
  if (props.modelValue.type === 'basic') patch({ username: v })
}
function setBasicPassword(v: string): void {
  if (props.modelValue.type === 'basic') patch({ password: v })
}
function setApiKeyName(v: string): void {
  if (props.modelValue.type === 'api_key') patch({ key: v })
}
function setApiKeyValue(v: string): void {
  if (props.modelValue.type === 'api_key') patch({ value: v })
}
function setApiKeyInHeader(v: boolean): void {
  if (props.modelValue.type === 'api_key') patch({ in_header: v })
}
</script>

<template>
  <div class="auth-editor">
    <n-radio-group
      :value="modelValue.type"
      @update:value="(v: string) => setType(v as AuthType)"
    >
      <n-radio-button value="none">none</n-radio-button>
      <n-radio-button value="bearer">Bearer</n-radio-button>
      <n-radio-button value="basic">Basic</n-radio-button>
      <n-radio-button value="api_key">Api Key</n-radio-button>
    </n-radio-group>

    <div class="auth-content">
      <!-- none: nothing -->

      <template v-if="modelValue.type === 'bearer'">
        <div class="auth-row">
          <label>Token</label>
          <n-input
            :value="bearerToken"
            placeholder="eyJhbGciOi..."
            type="password"
            show-password-on="click"
            :input-props="{ autocomplete: 'off' }"
            @update:value="setBearerToken"
          />
        </div>
      </template>

      <template v-else-if="modelValue.type === 'basic'">
        <div class="auth-row">
          <label>Username</label>
          <n-input
            :value="basicUsername"
            placeholder="alice"
            :input-props="{ autocomplete: 'off' }"
            @update:value="setBasicUsername"
          />
        </div>
        <div class="auth-row">
          <label>Password</label>
          <n-input
            :value="basicPassword"
            type="password"
            show-password-on="click"
            :input-props="{ autocomplete: 'off' }"
            @update:value="setBasicPassword"
          />
        </div>
      </template>

      <template v-else-if="modelValue.type === 'api_key'">
        <div class="auth-row">
          <label>Key 名</label>
          <n-input
            :value="apiKeyName"
            placeholder="X-API-Key"
            :input-props="{ autocomplete: 'off' }"
            @update:value="setApiKeyName"
          />
        </div>
        <div class="auth-row">
          <label>Value</label>
          <n-input
            :value="apiKeyValue"
            :input-props="{ autocomplete: 'off' }"
            @update:value="setApiKeyValue"
          />
        </div>
        <div class="auth-row">
          <label>位置</label>
          <n-radio-group
            :value="apiKeyInHeader"
            @update:value="setApiKeyInHeader"
          >
            <n-radio :value="true">in Header</n-radio>
            <n-radio :value="false">in Query</n-radio>
          </n-radio-group>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.auth-editor {
  width: 100%;
}

.auth-content {
  margin-top: 16px;
}

.auth-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.auth-row > label {
  width: 80px;
  flex-shrink: 0;
  text-align: right;
  color: var(--n-text-color-3);
}

.auth-row > :not(label) {
  flex: 1;
}
</style>
