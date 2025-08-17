<template>
  <div class="window theme-light">
    <WindowHeader/>
    <div class="window-body">
      <LeftPanel/>
      <RightPanel/>
    </div>
    <WindowFooter/>
    <MappingModal/>
    <JoystickCaliModal/>
  </div>
</template>

<script setup lang="ts">
import WindowHeader from './vue/WindowHeader.vue'
import LeftPanel from './vue/LeftPanel.vue'
import RightPanel from './vue/RightPanel.vue'
import WindowFooter from './vue/WindowFooter.vue'
import MappingModal from './vue/MappingModal.vue'
import JoystickCaliModal from './vue/JoystickCaliModal.vue'
import {onMounted} from "vue";
import {initApp} from "@/App.ts";

onMounted(() => {
  initApp();
})
</script>

<style>
@import './assets/styles.css'; /* 全局样式 */
</style>
