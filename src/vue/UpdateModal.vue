<template>
  <transition name="modal-fade">
    <div class="modal-overlay" v-if="state.showUpdateModal" @click.self="closeUpdateModal">
      <div class="modal">
        <div class="modal-header">
          <span>发现新版本！</span>
          <button class="modal-close" @click="closeUpdateModal">&times;</button>
        </div>
        <div class="modal-body">
          <p>我们找到了一个新的版本: <strong>{{ state.newVersionInfo }}</strong></p>
          <p>是否立即下载并安装更新？</p>
        </div>
        <div class="modal-footer">
          <button class="btn btn-outline" @click="closeUpdateModal">稍后提醒</button>
          <button class="btn btn-primary" @click="startUpdate">立即更新</button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { state } from '@/ts/global_states';
import { performUpdate } from '@/ts/WindowHeader';

const closeUpdateModal = () => {
  state.showUpdateModal = false;
};

const startUpdate = async () => {
  await performUpdate();
  closeUpdateModal();
};
</script>

<style scoped>
/* Using global styles from styles.css for consistency */
.modal-body p {
  margin: 10px 0;
  font-size: 16px;
  line-height: 1.6;
}
</style>