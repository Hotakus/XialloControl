<template>
  <div class="window-footer">
    <div class="indicator-container">
      <span class="indicator-label">连接状态</span>
      <div id="status-indicator" class="indicator" :class="{on: state.isConnected}"></div>
    </div>

    <div>
      <span>© 2025 XenoControl {{ state.version }}</span>
      <a
          id="github-link"
          href="https://github.com/Hotakus/XenoControl"
          target="_blank"
          rel="noopener noreferrer"
          title="GitHub 仓库"
          @click.prevent="openGithubLink"
      >GitHub</a>
    </div>
  </div>
</template>

<script setup lang="ts">
// 可以写组件逻辑
import {getVersion} from '@tauri-apps/api/app'
import {onMounted} from "vue";
import {state} from "@/ts/global_states.ts";
import {invoke} from "@tauri-apps/api/core";

const openGithubLink = () => {
  invoke("open_url", { url: "https://github.com/Hotakus/XenoControl" });
};

async function v() {
  state.version = await getVersion()
  console.log("XenoControl Version: ", state.version)
}

onMounted(() => {
  v()
})
</script>

<style scoped>
/* 如果只作用于这个组件，可以写 scoped 样式 */
</style>
