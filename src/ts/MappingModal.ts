import { state } from "@/ts/global_states.ts";
import { updateStatusMessage } from "@/ts/LeftPanel.ts";
import { queryMappings } from "@/App.ts";
import { startKeyDetection, stopKeyDetection } from "@/ts/RightPanel.ts";
import { invoke } from "@tauri-apps/api/core";

/**
 * 映射更新配置接口，对应后端的 MappingUpdateConfig 结构体
 */
export interface MappingUpdateConfig {
    id: number;
    composed_button?: string;
    composed_shortcut_key?: string;
    trigger_state?: {
        continually_trigger: boolean;
        interval: number;
        initial_interval: number;
        min_interval: number;
        acceleration: number;
    };
    trigger_theshold?: number;
    amount?: number | null;
    check_mode?: string;
    check_mode_param?: number;
}

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
        continually_trigger: state.triggerState.continually_trigger,
        interval: state.triggerState.initial_interval,
        initial_interval: state.triggerState.initial_interval,
        min_interval: state.triggerState.min_interval,
        acceleration: state.triggerState.acceleration,
    };

    let result = false;
    const payload: MappingUpdateConfig = {
        id: state.editingMappingId || 0,
        composed_button: composed_button,
        composed_shortcut_key: raw_shortcut_key,
        trigger_state: trigger_state,
        trigger_theshold: state.triggerTheshold,
        amount: (() => {
            const lowerCaseKey = raw_shortcut_key.toLowerCase();
            if (lowerCaseKey.includes('mousewheelup')) {
                return -state.mapping_amount;
            } else if (lowerCaseKey.includes('mousewheeldown')) {
                return state.mapping_amount;
            }
            return null;
        })(),
        check_mode: state.checkMode,
        check_mode_param: state.checkModeParam,
    };

    console.log("提交的映射配置:", payload);

    if (state.editingMappingId) {
        result = await invoke<boolean>("update_mapping", { config: payload });
        updateStatusMessage('映射已更新');
    } else {
        result = await invoke<boolean>("add_mapping", { config: payload });
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
