import { ref } from "vue";
import { state } from "@/ts/global_states";
import { invoke } from "@tauri-apps/api/core";
import { switchPreset } from "@/ts/RightPanel";
import { updateStatusMessage } from "./LeftPanel";

export const editablePresetName = ref('');

export const initEditablePresetName = () => {
    if (state.current_preset) {
        editablePresetName.value = state.current_preset.name;
    }
};

export const handleRenamePreset = async () => {
    const oldName = state.current_preset.name;
    const newName = editablePresetName.value.trim();

    if (!newName || newName === oldName) {
        return;
    }

    try {
        await invoke("rename_preset", { oldName, newName });

        // 更新前端状态
        const index = state.presets.indexOf(oldName);
        if (index !== -1) {
            state.presets[index] = newName;
        }
        state.previousPreset = newName;
        await switchPreset();

    } catch (error) {
        console.error("重命名预设失败:", error);
        // 失败时，把输入框的值重置回原来的名字
        editablePresetName.value = oldName;
        updateStatusMessage(`重命名预设失败: ${error}`, true);
    }
};