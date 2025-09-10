import { ref, reactive, watch, computed } from "vue";
import { state } from "@/ts/global_states";
import { invoke } from "@tauri-apps/api/core";
import { switchPreset, updateControllerButtons } from "@/ts/RightPanel";
import { updateStatusMessage } from "./LeftPanel";
import { translate } from "./i18n";

export const editablePresetName = ref('');

export const controllerButtons = computed(() => state.buttonsText.map(btn => btn.value));

// 这个 reactive 对象现在直接镜像 state.current_preset.items 的一部分
export const subPresetOptions = reactive({
    sub_preset_name: null as string | null,
    sub_preset_switch_button: null as string | null,
    sub_preset_switch_mode: 'Hold' as string | null,
});

export function initializeSubPresetOptions() {
    updateControllerButtons();
    if (state.current_preset && state.current_preset.items) {
        subPresetOptions.sub_preset_name = state.current_preset.items.sub_preset_name;
        subPresetOptions.sub_preset_switch_button = state.current_preset.items.sub_preset_switch_button;
        subPresetOptions.sub_preset_switch_mode = state.current_preset.items.sub_preset_switch_mode || 'Hold';
    }
}

watch(
    subPresetOptions,
    async (newOptions) => {
        if (!state.current_preset) return;

        // 创建一个新的 items 对象，包含所有字段
        const updatedItems = {
            ...state.current_preset.items,
            sub_preset_name: newOptions.sub_preset_name,
            sub_preset_switch_button: newOptions.sub_preset_switch_button,
            sub_preset_switch_mode: newOptions.sub_preset_switch_mode,
        };

        try {
            await invoke("update_preset_items", { items: updatedItems });
            // 更新本地状态以保持同步
            state.current_preset.items = updatedItems;
        } catch (error) {
            console.error("更新预设失败:", error);
            updateStatusMessage(`更新预设失败: ${error}`, true);
        }
    },
    { deep: true }
);

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

        const index = state.presets.indexOf(oldName);
        if (index !== -1) {
            state.presets[index] = newName;
        }
        state.previousPreset = newName;
        await switchPreset();

    } catch (error) {
        console.error("重命名预设失败:", error);
        editablePresetName.value = oldName;
        updateStatusMessage(`重命名预设失败: ${error}`, true);
    }
};