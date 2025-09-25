import {state} from "@/ts/global_states.ts";
import {updateStatusMessage} from "@/ts/LeftPanel.ts";
import {queryMappings} from "@/App.ts";
import {startKeyDetection, stopKeyDetection} from "@/ts/RightPanel.ts";
import {invoke} from "@tauri-apps/api/core";

export async function mappingsConfirm() {
    stopKeyDetection();
    state.modalErrorVisible = false;
    state.modalErrorMessage = '';

    const composed_button = state.selectedButton;
    if (!composed_button) {
        state.modalErrorMessage = '请选择一个输入源';
        state.modalErrorVisible = true;
        return;
    }

    // 从 state.currentKeys 构建用于后端的原始快捷键字符串
    const shortcut_parts = [];
    if (state.currentKeys.ctrl) shortcut_parts.push('Control');
    if (state.currentKeys.shift) shortcut_parts.push('Shift');
    if (state.currentKeys.alt) shortcut_parts.push('Alt');
    if (state.currentKeys.meta) shortcut_parts.push('Meta');
    if (state.currentKeys.key) shortcut_parts.push(state.currentKeys.key);
    const raw_shortcut_key = shortcut_parts.join('+');

    if (!raw_shortcut_key) {
        state.modalErrorMessage = '请设置映射输出动作';
        state.modalErrorVisible = true;
        return;
    }

    // For all mapping types, we now use trigger_state from the UI
    const trigger_state = {
        interval: state.triggerState.initial_interval,
        initial_interval: state.triggerState.initial_interval,
        min_interval: state.triggerState.min_interval,
        acceleration: state.triggerState.acceleration,
    };

    let result = false;
    const payload = {
        id: state.editingMappingId,
        composedButton: composed_button,
        composedShortcutKey: raw_shortcut_key,
        triggerState: trigger_state,
        triggerTheshold: state.triggerTheshold,
        amount: (() => {
            const lowerCaseKey = raw_shortcut_key.toLowerCase();
            if (lowerCaseKey.includes('mousewheelup')) {
                return -state.mapping_amount;
            } else if (lowerCaseKey.includes('mousewheeldown')) {
                return state.mapping_amount;
            }
            return null;
        })(),
    };

    if (state.editingMappingId) {
        result = await invoke("update_mapping", payload);
        updateStatusMessage('映射已更新');
    } else {
        result = await invoke("add_mapping", payload);
        updateStatusMessage('映射已添加');
    }

    if (result) {
        await queryMappings();
    } else {
        updateStatusMessage('映射操作失败', true);
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
