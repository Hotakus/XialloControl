<template>
    <transition name="modal-fade">
        <div class="modal-overlay" :class="{active: state.showUpdateModal}" @click.self="closeUpdateModal()">
            <div class="modal">
                <div class="modal-header">
                    <span>发现新版本！</span>
                    <button class="modal-close" @click="closeUpdateModal()">&times;</button>
                </div>
                <div class="modal-body" v-if="state.updateInfo">
                    <p>我们找到了一个新的版本: <strong>{{ state.updateInfo.version }} ({{ state.updateInfo.date }})</strong></p>
                    <p>更新内容:</p>
                    <pre>{{ state.updateInfo.body }}</pre>
                    <p>是否立即下载并安装更新？</p>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-outline" @click="closeUpdateModal()">稍后提醒</button>
                    <button class="btn btn-primary" @click="startUpdate">立即更新</button>
                </div>
            </div>
        </div>
    </transition>
</template>

<script setup lang="ts">
import { state } from '@/ts/global_states';
import { performUpdate } from '@/ts/WindowHeader';
import { closeUpdateModal } from '@/ts/UpdateModal';

const startUpdate = async () => {
    await performUpdate();
    closeUpdateModal();
};
</script>

<style scoped>
/* Using global styles from styles.css for consistency */
.modal {
    max-width: 500px;
}

.modal-body p {
    margin: 10px 0;
    font-size: 16px;
    line-height: 1.6;
}

.modal-body pre {
    background-color: #f1f3f6;
    padding: 15px;
    border-radius: 8px;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 200px;
    overflow-y: auto;
    font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
    font-size: 14px;
    line-height: 1.5;
}
</style>