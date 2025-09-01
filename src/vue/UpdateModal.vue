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
                    <!-- <pre>{{ state.updateInfo.body }}</pre> -->
                    <div class="markdown-body" v-html="renderedBody"></div>
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
import { computed } from 'vue';
import { marked } from 'marked';

const startUpdate = async () => {
    await performUpdate();
    closeUpdateModal();
};

const renderedBody = computed(() => {
    // 检查 state.updateInfo.body 是否存在
    if (state.updateInfo?.body) {
        // 将 Markdown 字符串解析成 HTML
        return marked(state.updateInfo.body) as string;
    }
    return ''; // 如果不存在，返回空字符串
});
</script>

<style scoped>
/* Using global styles from styles.css for consistency */
.modal {
    max-width: 600px;
}

.modal-body p {
    margin: 10px 0;
    font-size: 16px;
    line-height: 1.6;
}

.markdown-body {
    background-color: #f1f3f680;
    border: 1px solid #d1d5db;
    padding: 15px;
    border-radius: 8px;
    max-height: 300px;
    overflow-y: auto;
    font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
    font-size: 14px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
}

/* ✨ 这里是调整标题间距的部分！ ✨ */
/* `margin-top` 控制标题上方与上一个元素的距离
   `margin-bottom` 控制标题下方与下一个元素的距离
   你可以根据自己的喜好调整这些值。
   比如，可以尝试更小的值如 0.5em, 0.25em, 10px 等等。
*/
:deep(.markdown-body h1),
:deep(.markdown-body h2),
:deep(.markdown-body h3),
:deep(.markdown-body h4), /* 如果你的Markdown可能用到 h4, h5, h6，也一并加上 */
:deep(.markdown-body h5),
:deep(.markdown-body h6) {
    margin-top: 0.8em;   /* 调整标题上方的间距，比如从 1em 改为 0.8em */
    margin-bottom: 0.4em; /* 调整标题下方的间距，比如从 0.5em 改为 0.4em */
    font-weight: 600;
    line-height: 1.25;
}

/* 也可以更精细地分别调整不同级别的标题 */
:deep(.markdown-body h1) {
    margin-top: 0.1em;
    margin-bottom: 0.1em;
}
:deep(.markdown-body h2) {
    margin-top: 0.1em;
    margin-bottom: 0.1em;
}
:deep(.markdown-body h3) {
    margin-top: 0.1em;
    margin-bottom: 0.1em;
}


:deep(.markdown-body ul) {
    padding-left: 20px;
    margin-top: 0.5em; /* 也可以调整列表上方的间距 */
    margin-bottom: 0.5em; /* 也可以调整列表下方的间距 */
}

:deep(.markdown-body li) {
    margin-bottom: 0.25em;
}

:deep(.markdown-body a) {
    color: #0366d6;
    text-decoration: none;
}
:deep(.markdown-body a:hover) {
    text-decoration: underline;
}

/* 还可以调整段落的间距，让它看起来更紧凑 */
:deep(.markdown-body p) {
    margin-top: 0.5em;
    margin-bottom: 0.5em;
}

/* 如果有代码块，也可以调整代码块的间距 */
:deep(.markdown-body pre) {
    margin-top: 1em;
    margin-bottom: 1em;
}

/* 如果有图片，也可以调整图片间距 */
:deep(.markdown-body img) {
    max-width: 100%;
    height: auto;
    margin-top: 1em;
    margin-bottom: 1em;
}

</style>