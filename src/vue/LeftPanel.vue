<template>
  <div class="left-panel">
    <div class="card">
      <label for="device">🎮 选择设备:</label>
      <div class="device-select-row">
        <select id="device" v-model="state.deviceSelectedIndex" @change="onDeviceSelected()">
          <option disabled value="null">请选择设备</option>
          <!-- 用 v-for 渲染设备 -->
          <option v-for="(device, index) in state.currentDevices"
                  :key="device.device_path ?? index"
                  :value="index">
            {{ index }}: {{ device.name }}
          </option>
        </select>
        <button id="connect-button" title="连接设备" class="icon-button"
                :disabled="state.connectButtonDisabled"
                :class="{disabled: state.connectButtonDisabled, connected: state.isConnected}"
                v-html="state.connectIcon"
                @click="toggleDeviceConnection()">
        </button>
        <button id="scan-button" title="扫描设备" class="icon-button" :class="{scanning: state.isScanning}" @click="scanDevices()">
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

    <div class="card controller-image">
      <component :is="currentControllerSvg" class="controller-svg"/>
    </div>
  </div>
</template>

<script setup lang="ts">
// 可以写组件逻辑
import {appWindow, state} from "@/ts/global_states.ts";
import {connectStatusIcons, onDeviceSelected, scanDevices, toggleDeviceConnection} from "@/ts/LeftPanel.ts";
import {onMounted} from "vue";
import {currentControllerSvg, test} from "@/ts/ControllerGraph.ts";

onMounted(() => {
  state.connectIcon = connectStatusIcons.disconnected;
  scanDevices();
});
</script>

<style scoped>
/* 如果只作用于这个组件，可以写 scoped 样式 */
</style>
