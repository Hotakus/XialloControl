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
                        <div class="preset-name-display">{{ state.current_preset.name }}</div>
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
import {state} from "@/ts/global_states";

const closePresetEditModal = () => {
    state.showPresetEditModal = false;
};

</script>

<style scoped></style>