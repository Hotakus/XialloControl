<template>
  <div class="window-header"
       :class="{ 'show': state.titlebar_visible, 'hide':!state.titlebar_visible }"
       id="titlebar" data-tauri-drag-region>
    <div class="header-left">
      <!-- <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="white">
        <path d="M18 2h-3.5l-1-1h-5l-1 1H6v2h12V2zm3 7H3v11c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V9zM8 12h2v5H8v-5zm6 0h2v5h-2v-5z"/>
      </svg> -->
      <span>XialloControl</span>
      <button v-if="state.updateInfo" class="new-update-btn" title="有新版本" @click="openUpdateModal()">New</button>
    </div>
    <div class="window-controls">
      <button id="minimize-button" @click="minimize()">
        <svg t="1755230652519" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="7096" width="200" height="200">
          <path d="M128 448h768v128H128z" p-id="7097"></path>
        </svg>
      </button>
      <button id="maximize-button" @click="maximize()">
        <svg t="1755231281539" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="8298" width="200" height="200">
          <path
              d="M307.2 0H153.6C68.7744 0 0 68.7744 0 153.6v153.6a51.2 51.2 0 1 0 102.4 0V153.6a51.2 51.2 0 0 1 51.2-51.2h153.6a51.2 51.2 0 1 0 0-102.4z m716.8 307.2V153.6c0-84.8256-68.7744-153.6-153.6-153.6H716.8a51.2 51.2 0 1 0 0 102.4h153.6a51.2 51.2 0 0 1 51.2 51.2v153.6a51.2 51.2 0 1 0 102.4 0zM716.8 1024h153.6c84.8256 0 153.6-68.7744 153.6-153.6V716.8a51.2 51.2 0 1 0-102.4 0v153.6a51.2 51.2 0 0 1-51.2 51.2H716.8a51.2 51.2 0 1 0 0 102.4zM0 716.8v153.6c0 84.8256 68.7744 153.6 153.6 153.6h153.6a51.2 51.2 0 1 0 0-102.4H153.6a51.2 51.2 0 0 1-51.2-51.2V716.8a51.2 51.2 0 1 0-102.4 0z"
              fill="#ffffff" p-id="8299"></path>
        </svg>
      </button>
      <button id="close-button" @click="close()">
        <svg t="1755230585996" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="4903" width="200" height="200">
          <path
              d="M614.4 512l387.657143-387.657143c29.257143-29.257143 29.257143-73.142857 0-102.4s-73.142857-29.257143-102.4 0L512 409.6 124.342857 21.942857C95.085714-7.314286 51.2-7.314286 21.942857 21.942857s-29.257143 73.142857 0 102.4L409.6 512 21.942857 899.657143c-29.257143 29.257143-29.257143 73.142857 0 102.4s73.142857 29.257143 102.4 0L512 614.4l387.657143 387.657143c29.257143 29.257143 73.142857 29.257143 102.4 0s29.257143-73.142857 0-102.4L614.4 512z"
              fill="#ffffff" p-id="4904"></path>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {onMounted} from "vue";
import {initUIElements, state} from "@/ts/global_states";
import {close, maximize, minimize, updateTitlebar} from "@/ts/WindowHeader";
import { openUpdateModal, checkUpdate } from "@/ts/UpdateModal";

onMounted(async () => {
  await initUIElements();      // 获取 DOM 元素
  await updateTitlebar();      // 更新 titlebar 显示
  await checkUpdate();         // 检查更新
});
</script>

<style scoped>
/* 如果只作用于这个组件，可以写 scoped 样式 */
.new-update-btn {
  background-color: #c15552;
  color: #fff;
  padding: 0 6px;
  height: 18px;
  line-height: 18px;
  font-size: 11px;
  font-weight: 600;
  border: none;
  border-radius: 4px;
  margin-left: 8px;
  user-select: none;
  box-shadow: none;
  transition: background-color 120ms ease, opacity 120ms ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.new-update-btn:hover {
  background-color: rgb(202, 69, 69);
}
.new-update-btn:active {
  background-color: #c44545; /* A slightly darker red for click feedback */
  transform: none; /* Remove any scaling or movement */
}
</style>
