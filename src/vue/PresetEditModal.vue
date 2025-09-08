<template>
    <transition name="modal-fade">
        <div class="modal-overlay" :class="{ active: state.showPresetEditModal }" @click.self="closePresetEditModal()">
            <div class="modal">
                <div class="modal-header">
                    <span>编辑预设</span>
                    <button class="modal-close" @click="closePresetEditModal()">&times;</button>
                </div>
                <div class="modal-body">
                    <div class="form-group">
                        <label>预设名称</label>
                        <div class="input-group">
                            <input type="text" class="form-control" v-model="editablePresetName" placeholder="输入新的预设名称"
                                @keyup.enter="handleRenamePreset">
                            <button class="icon-button" @click="handleRenamePreset">
                                <svg t="1757211709577" class="icon" viewBox="0 0 1024 1024" version="1.1"
                                    xmlns="http://www.w3.org/2000/svg" p-id="4551" width="200" height="200">
                                    <path
                                        d="M75.9808 969.5232l86.016-299.2128 214.7328 213.7088-300.7488 85.504z m330.752-115.0976l-214.8352-214.016L664.576 170.1888l214.8352 213.7088-472.6784 470.528z m569.6512-567.296l-64.4096 64.3072L697.344 137.6256l64.512-64.1024c23.552-23.552 60.928-24.8832 82.944-2.6624l134.3488 133.632c22.3232 22.1184 21.0944 59.0848-2.6624 82.7392z"
                                        fill="#ffffff" p-id="4552"></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                    <div class="setting-item sub-preset-toggle">
                        <label for="as-sub-preset">作为副预设:</label>
                        <label class="switch">
                            <input type="checkbox" id="as-sub-preset" v-model="state.current_preset.items.as_sub">
                            <span class="slider round"></span>
                        </label>
                    </div>
                </div>
                <div class="modal-footer">
                    <!-- <button class="btn btn-outline" @click="closePresetEditModal()">取消</button>
                    <button class="btn btn-primary" @click="savePresetSettings()">保存</button> -->
                </div>
            </div>
        </div>
    </transition>
</template>

<script setup lang="ts">
import { watch } from "vue";
import { state } from "@/ts/global_states";
import { handleRenamePreset, editablePresetName, initEditablePresetName } from "@/ts/PresetEditModal";

watch(() => state.showPresetEditModal, (isVisible) => {
    if (isVisible) {
        initEditablePresetName();
    }
});

const closePresetEditModal = () => {
    state.showPresetEditModal = false;
};

</script>

<style scoped>
.modal {
    max-width: 300px;
}

.input-group {
    display: flex;
    align-items: center;
    gap: 10px;
}

.input-group .form-control {
    padding: 8px 12px;
    height: 36px;
    /* 与 icon-button 的默认高度对齐 */
}

/* 使图标按钮尺寸与输入框更协调 */
.item-action-btn {
    width: 36px;
    height: 36px;
    font-size: 16px;
}
</style>