<template>
  <div class="window-header"
       id="titlebar">
    <div class="header-left">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="white">
        <path d="M18 2h-3.5l-1-1h-5l-1 1H6v2h12V2zm3 7H3v11c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V9zM8 12h2v5H8v-5zm6 0h2v5h-2v-5z"/>
      </svg>
      <span>XenoControl</span>
    </div>
    <div>
      <button id="minimize-button">🗕</button>
      <button id="close-button">✖</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { initUIElements } from "@/ts/global_states";
import { updateTitlebar, setupWindowButtons } from "@/ts/WindowHeader";

onMounted(async () => {
  await initUIElements();      // 获取 DOM 元素
  await updateTitlebar();      // 更新 titlebar 显示
  setupWindowButtons();        // 绑定最小化/关闭事件
});
</script>

<style scoped>
/* 如果只作用于这个组件，可以写 scoped 样式 */
</style>
