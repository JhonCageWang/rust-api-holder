<script setup lang="ts">
// TODO(Week 6): 请求历史列表 + 回填到编辑器
</script>

<template>
  <div class="history-view">
    <div class="placeholder">
      <n-h2>📜 请求历史</n-h2>
      <n-p>Week 6 实现</n-p>
      <n-ul>
        <n-li>所有发送过的请求(成功 / 失败)</n-li>
        <n-li>点击历史项 → 回填到编辑器</n-li>
        <n-li>按时间倒序排列,支持搜索</n-li>
      </n-ul>
    </div>
  </div>
</template>

<style scoped>
.history-view {
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