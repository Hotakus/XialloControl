import {state} from "@/ts/global_states.ts";
import {updateStatusMessage} from "@/ts/LeftPanel.ts";
import {queryMappings} from "@/App.ts";
import {startKeyDetection, stopKeyDetection} from "@/ts/RightPanel.ts";
import {invoke} from "@tauri-apps/api/core";

export async function mappingsConfirm() {
    const composed_button = state.selectedButton;

    // 从 state.currentKeys 构建用于后端的原始快捷键字符串
    const shortcut_parts = [];
    if (state.currentKeys.ctrl) shortcut_parts.push('Control');
    if (state.currentKeys.shift) shortcut_parts.push('Shift');
    if (state.currentKeys.alt) shortcut_parts.push('Alt');
    if (state.currentKeys.meta) shortcut_parts.push('Meta');
    if (state.currentKeys.key) shortcut_parts.push(state.currentKeys.key);
    const raw_shortcut_key = shortcut_parts.join('+'); // 这就是后端需要的英文值

    stopKeyDetection();

    state.modalErrorVisible = false;
    state.modalErrorMessage = '';

    if (!composed_button) {
        state.modalErrorMessage = '请选择手柄按键';
        state.modalErrorVisible = true;
        return;
    }

    if (!raw_shortcut_key) { // 使用新生成的原始值进行校验
        state.modalErrorMessage = '请设置键盘映射按键';
        state.modalErrorVisible = true;
        return;
    }

    // 从全局状态中获取触发器状态
    const trigger_state = {
        interval: state.triggerState.initial_interval, // 关键：添加 interval 字段
        initial_interval: state.triggerState.initial_interval,
        min_interval: state.triggerState.min_interval,
        acceleration: state.triggerState.acceleration,
        // Rust 结构中的 `last_trigger` 是在后端处理的，前端不需要发送
    };


    let result = false;
    if (state.editingMappingId) {
        result = await invoke("update_mapping", {
            id: state.editingMappingId,
            composedButton: composed_button,
            composedShortcutKey: raw_shortcut_key,
            triggerState: trigger_state,
        });
        updateStatusMessage('按键映射已更新');
    } else {
        result = await invoke("add_mapping", {
            composedButton: composed_button,
            composedShortcutKey: raw_shortcut_key,
            triggerState: trigger_state,
        });
        updateStatusMessage('按键映射已添加');
    }

    if (result) {
        // 保存成功，重新加载映射列表
        await queryMappings();
    } else {
        updateStatusMessage('按键映射操作失败', true);
    }

    await closeButtonMapModal();
}


export async function detectKey() {
    if (!state.keyListenerActive) {
        if (state.preventNextClick) {
            state.preventNextClick = false;
            return;
        }
        startKeyDetection();
        console.log("开始按键监听");
    }
}

export async function closeButtonMapModal() {
    stopKeyDetection(true);
    state.showMappingModal = false;
}
