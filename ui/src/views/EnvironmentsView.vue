<script setup lang="ts">
// TODO(Week 6): 环境变量 CRUD + 切换
</script>

<template>
  <div class="env-view">
    <div class="placeholder">
      <n-h2>🌍 环境管理</n-h2>
      <n-p>Week 6 实现</n-p>
      <n-ul>
        <n-li>创建 / 编辑 / 删除环境(dev / staging / prod)</n-li>
        <n-li>环境变量 CRUD</n-li>
        <n-li>激活切换(影响所有请求的 <n-text code>{{ '{{var}}' }}</n-text> 插值)</n-li>
      </n-ul>
    </div>
  </div>
</template>

<style scoped>
.env-view {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.placeholder {
  max-width: 600px;
  padding: 32px;
}
</style>