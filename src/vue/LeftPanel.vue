<template>
  <div class="left-panel">
    <div class="card">
      <label for="device">🎮 选择设备:</label>
      <div class="device-select-row">
        <select id="device">
          <option value="null">请选择设备</option>
        </select>
        <button id="connect-button" title="连接设备" class="icon-button">
          <svg t="1753595424804" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="19662" width="200" height="200">
            <path
                d="M521.6 158.08a111.36 111.36 0 0 1 111.36 111.36V480l96 96V269.44a206.72 206.72 0 0 0-394.24-87.68L411.52 256a110.72 110.72 0 0 1 110.08-97.92zM864 846.08l-135.04-135.04-96-96-222.08-222.08-96-96-135.04-134.4L112 230.4l202.88 202.24v306.56a206.72 206.72 0 0 0 394.24 87.68l87.04 87.04z m-341.76 4.48a111.36 111.36 0 0 1-111.36-111.36V528.64l221.44 221.44a110.72 110.72 0 0 1-110.72 100.48z"
                fill="#ffffff" p-id="19663"></path>
          </svg>
        </button>
        <button id="scan-button" title="扫描设备" class="icon-button">
          <svg t="1753598068869" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="23500" width="200" height="200">
            <path
                d="M511.94 125.24c-118 0-227 54.22-298.6 142.52V85.33H128V469.34h341.33V384H240.67c48.43-104.78 154.16-173.43 271.27-173.43 164.72 0 298.73 134.01 298.73 298.73S676.66 808.03 511.94 808.03c-111.71 0-213.25-61.57-265-160.69l-75.64 39.5c66.51 127.39 197.04 206.52 340.64 206.52C723.71 893.37 896 721.08 896 509.3S723.71 125.24 511.94 125.24z"
                fill="#ffffff" p-id="23501"></path>
          </svg>
        </button>
      </div>
      <div id="status-message"
           class="status-message"
           :class="{ success: state.statusMessageIsSuccess, error: state.statusMessageIsError }">
        {{ state.statusMessage }}
      </div>
    </div>

    <div class="card controller-image"></div>
  </div>
</template>

<script setup lang="ts">
// 可以写组件逻辑
import {state} from "@/ts/global_states.ts";
</script>

<style scoped>
/* 如果只作用于这个组件，可以写 scoped 样式 */
</style>
