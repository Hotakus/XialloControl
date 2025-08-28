import { Preset, state } from "@/ts/global_states.ts";
import { invoke } from "@tauri-apps/api/core";
import { updateStatusMessage } from "@/ts/LeftPanel.ts";
import { queryMappings, refreshMappings } from "@/App.ts";
import { nextTick } from "vue";


/**
 * 切换标签页
 * @param tabName 要切换到的标签页名称
 */
export function switchTab(tabName: string) {
    state.activeTab = tabName;
}


export async function setPollingFrequency() {
    await invoke("set_frequency", { freq: state.pollingFrequency });
    await updateSettings();
}

/**
 * 保存设置
 */
export async function updateSettings() {
    const newSettings = {
        auto_start: state.autoStart,
        minimize_to_tray: state.minimizeToTray,
        remember_last_connection: state.rememberLastConnection,
        last_connected_device: state.lastConnectedDevice,
        theme: state.theme,
        polling_frequency: state.pollingFrequency,
        previous_preset: state.previousPreset
    };

    try {
        await invoke("update_settings", { newSettings });
    } catch (error) {
        console.error("保存设置失败:", error);
    }
}


/**
 * 切换主题
 */
export async function changeTheme() {
    // TODO: 切换主题

    await updateSettings();
}


export async function openButtonMapModal(title = "添加按键映射", selectedButton = "", keyDisplayText = "", mappingId: any = null) {
    state.modalErrorVisible = false;
    state.modalErrorMessage = '';

    state.modalTitle = title;

    state.keyDisplayText = keyDisplayText;
    state.selectedButton = selectedButton;

    state.editingMappingId = mappingId;

    updateControllerButtons();
    state.showMappingModal = true;
}

export async function editButtonMap(id: number) {
    // 从后端直接获取最新的映射数据
    const mapping: any = await invoke("get_mapping_by_id", { id });

    if (mapping) {
        // --- 新增转换和状态恢复逻辑 ---
        const raw_key: string = mapping.composed_shortcut_key; // 获取英文原始值

        // 1. 转换为中文显示值，用于弹窗UI
        const display_key = raw_key.split(' + ').map(part => keyDisplayNames[part] || part.toUpperCase()).join(' + ');

        // 2. 反向解析原始值，恢复 state.currentKeys 状态
        const parts = raw_key.split(' + ');
        state.currentKeys = { ctrl: false, shift: false, alt: false, meta: false, key: null }; // 重置
        state.currentKeys.ctrl = parts.includes('Control');
        state.currentKeys.shift = parts.includes('Shift');
        state.currentKeys.alt = parts.includes('Alt');
        state.currentKeys.meta = parts.includes('Meta');
        // 查找不是修饰键的部分作为主键
        state.currentKeys.key = parts.find(p => !['Control', 'Shift', 'Alt', 'Meta'].includes(p)) || null;

        // 3. 恢复 trigger state (假设后端返回的字段名是 snake_case)
        state.triggerState.initial_interval = mapping.initial_interval ?? 300;
        state.triggerState.min_interval = mapping.min_interval ?? 100;
        state.triggerState.acceleration = mapping.acceleration ?? 0.8;

        // 使用转换后的中文值和恢复的状态打开模态窗口
        console.log("编辑按钮映射", id);
        await openButtonMapModal("编辑按键映射", mapping.composed_button, display_key, mapping.id);
        // --- 逻辑结束 ---
    } else {
        updateStatusMessage(`无法找到 ID 为 ${id} 的映射`, true);
    }
}

export async function deleteButtonMap(id: number) {
    invoke('delete_mapping', { id: id })
        .then(success => {
            if (success) {
                updateStatusMessage('映射已删除');
                // 从后端重新加载列表，保证数据同步
                queryMappings();
            } else {
                updateStatusMessage('删除映射失败，项目可能已被移除', true);
            }
        })
        .catch(err => {
            updateStatusMessage(`删除失败: ${err}`, true);
        });
    // renderMappings();
}


export async function addButtonMap() {
    // 重置 trigger state 为默认值
    state.triggerState.initial_interval = 300;
    state.triggerState.min_interval = 100;
    state.triggerState.acceleration = 0.8;
    await openButtonMapModal();
}


export function formatKeyDisplay(rawKey: string): string {
    return rawKey.split('+').map(part => keyDisplayNames[part] || part.toUpperCase()).join(' + ');
}

// 特殊键的显示名称映射
const keyDisplayNames: Record<string, string> = {
    ' ': '空格键',
    Control: 'Ctrl',
    Shift: 'Shift',
    Alt: 'Alt',
    Meta: 'Cmd',
    ArrowUp: '↑',
    ArrowDown: '↓',
    ArrowLeft: '←',
    ArrowRight: '→',
    Escape: 'Esc',
    Tab: 'Tab',
    CapsLock: 'Caps Lock',
    Enter: 'Enter',
    Backspace: 'Backspace',
    Delete: 'Delete',
    Insert: 'Insert',
    Home: 'Home',
    End: 'End',
    PageUp: 'Page Up',
    PageDown: 'Page Down',
    ContextMenu: '菜单键',
    F1: 'F1',
    F2: 'F2',
    F3: 'F3',
    F4: 'F4',
    F5: 'F5',
    F6: 'F6',
    F7: 'F7',
    F8: 'F8',
    F9: 'F9',
    F10: 'F10',
    F11: 'F11',
    F12: 'F12',
    MouseLeft: '鼠标左键',
    MouseRight: '鼠标右键',
    MouseMiddle: '鼠标中键',
    MouseX1: '鼠标侧键1',
    MouseX2: '鼠标侧键2',
    MouseWheelUp: '滚轮上',
    MouseWheelDown: '滚轮下',
};


// 更新按键显示
function updateKeyDisplay() {
    let displayText = '';

    if (state.currentKeys.ctrl) displayText += 'Ctrl + ';
    if (state.currentKeys.shift) displayText += 'Shift + ';
    if (state.currentKeys.alt) displayText += 'Alt + ';
    if (state.currentKeys.meta) displayText += 'Cmd + ';

    if (state.currentKeys.key) {
        const key = state.currentKeys.key;
        displayText += keyDisplayNames[key] || key.toUpperCase();
    }

    state.keyDisplayText = displayText;
}

// 移除按键监听器
function removeKeyListeners() {
    window.removeEventListener('keydown', handleKeyDown);
    window.removeEventListener('keyup', handleKeyUp);
    window.removeEventListener('mousedown', handleMouseDown);
    window.removeEventListener('mouseup', handleMouseUp);
    window.removeEventListener('wheel', handleWheel);
}

// 停止按键检测
export function stopKeyDetection(resetText = true) {
    if (!state.keyListenerActive) return;
    state.keyListenerActive = false;

    if (resetText) {
        state.keyDetectorText = '点击此处并按下键盘按键、鼠标按键或滚动滚轮';
    }

    removeKeyListeners();
}

// 处理按键事件
function handleKeyDown(e: any) {
    e.preventDefault();
    if (e.key === 'Control' || e.key === 'Ctrl') state.currentKeys.ctrl = true;
    else if (e.key === 'Shift') state.currentKeys.shift = true;
    else if (e.key === 'Alt') state.currentKeys.alt = true;
    else if (e.key === 'Meta') state.currentKeys.meta = true;
    else state.currentKeys.key = e.key;
    updateKeyDisplay();
}

function handleKeyUp(e: any) {
    if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
        stopKeyDetection();
    }
}

// 处理鼠标事件
function handleMouseDown(e: any) {
    e.preventDefault();
    e.stopPropagation();
    state.preventNextClick = true;

    state.currentKeys.ctrl = e.ctrlKey;
    state.currentKeys.shift = e.shiftKey;
    state.currentKeys.alt = e.altKey;
    state.currentKeys.meta = e.metaKey;

    const mouseKeys = ['MouseLeft', 'MouseMiddle', 'MouseRight', 'MouseX1', 'MouseX2'];
    state.currentKeys.key = mouseKeys[e.button] || null;

    if (state.currentKeys.key) {
        updateKeyDisplay();
        stopKeyDetection(false);
        window.removeEventListener('mouseup', handleMouseUp);
    }
}

function stopMouseDetection() {
    if (!state.keyListenerActive) return;
    window.removeEventListener('mousedown', handleMouseDown);
    window.removeEventListener('mouseup', handleMouseUp);
    state.keyListenerActive = false;
}

function handleMouseUp(e: any) {
    stopMouseDetection();
}

// 处理滚轮事件
function handleWheel(e: any) {
    e.preventDefault();
    e.stopPropagation();

    state.currentKeys.ctrl = e.ctrlKey;
    state.currentKeys.shift = e.shiftKey;
    state.currentKeys.alt = e.altKey;
    state.currentKeys.meta = e.metaKey;
    state.currentKeys.key = e.deltaY < 0 ? 'MouseWheelUp' : 'MouseWheelDown';

    updateKeyDisplay();
    stopKeyDetection(false);
}


export function startKeyDetection() {
    if (state.keyListenerActive) return;
    state.preventNextClick = false;
    state.keyListenerActive = true;
    state.currentKeys = { ctrl: false, shift: false, alt: false, meta: false, key: null };

    state.keyDetectorText = '请按下键盘按键、鼠标按键或滚动滚轮...';
    state.keyDisplayText = '';

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    window.addEventListener('mousedown', handleMouseDown);
    window.addEventListener('mouseup', handleMouseUp);
    window.addEventListener('wheel', handleWheel);
}

// TODO: 从后端请求手柄按键映射列表
// TODO: 也许会添加更多
const buttonTextMapLists = {
    xbox: [
        { value: 'A', text: 'A 按钮' },
        { value: 'B', text: 'B 按钮' },
        { value: 'X', text: 'X 按钮' },
        { value: 'Y', text: 'Y 按钮' },
        { value: 'LB', text: '左肩键 (LB)' },
        { value: 'RB', text: '右肩键 (RB)' },
        { value: 'LeftStick', text: '左摇杆' },
        { value: 'RightStick', text: '右摇杆' },
        { value: 'Back', text: 'Back 按钮' },
        { value: 'Start', text: 'Start 按钮' },
        { value: 'Guide', text: 'Guide 按钮' },
        { value: 'DPadUp', text: '方向键上' },
        { value: 'DPadDown', text: '方向键下' },
        { value: 'DPadLeft', text: '方向键左' },
        { value: 'DPadRight', text: '方向键右' },
    ],
    ps: [
        { value: 'Cross', text: '叉按钮 (Cross)' },
        { value: 'Circle', text: '圆按钮 (Circle)' },
        { value: 'Square', text: '方按钮 (Square)' },
        { value: 'Triangle', text: '三角按钮 (Triangle)' },
        { value: 'L1', text: '左肩键 (L1)' },
        { value: 'R1', text: '右肩键 (R1)' },
        { value: 'LeftStick', text: '左摇杆' },
        { value: 'RightStick', text: '右摇杆' },
        { value: 'Share', text: 'Share 按钮' },
        { value: 'Options', text: 'Options 按钮' },
        { value: 'PS', text: 'PS 按钮' },
        { value: 'DPadUp', text: '方向键上' },
        { value: 'DPadDown', text: '方向键下' },
        { value: 'DPadLeft', text: '方向键左' },
        { value: 'DPadRight', text: '方向键右' },
    ],
    switch: [ // 新增 Switch 布局
        { value: 'A', text: 'A 按钮' },
        { value: 'B', text: 'B 按钮' },
        { value: 'X', text: 'X 按钮' },
        { value: 'Y', text: 'Y 按钮' },
        { value: 'L', text: '左肩键 (L)' },
        { value: 'R', text: '右肩键 (R)' },
        { value: 'LeftStick', text: '左摇杆' },
        { value: 'RightStick', text: '右摇杆' },
        { value: 'Minus', text: 'Minus 按钮' },
        { value: 'Plus', text: 'Plus 按钮' },
        { value: 'Home', text: 'Home 按钮' },
        { value: 'DPadUp', text: '方向键上' },
        { value: 'DPadDown', text: '方向键下' },
        { value: 'DPadLeft', text: '方向键左' },
        { value: 'DPadRight', text: '方向键右' },
    ]
}

// 根据设备类型更新手柄按键选项
export function updateControllerButtons() {
    // while (uiElements.controllerButtonSelect.options.length > 1) {
    //     uiElements.controllerButtonSelect.remove(1);
    // }

    switch (state.deviceSelected?.controller_type) {
        case "Xbox": {
            state.buttonsText = buttonTextMapLists.xbox;
            break;
        }
        case "PlayStation": {
            state.buttonsText = buttonTextMapLists.ps;
            break;
        }
        case "Switch": { // 添加 Switch case
            state.buttonsText = buttonTextMapLists.switch;
            break;
        }
        default: {
            state.buttonsText = buttonTextMapLists.xbox;
            break;
        }
    }

    // buttons.forEach(button => {
    //     const option = document.createElement('option');
    //     option.value = button.value;
    //     option.textContent = button.text;
    //     uiElements.controllerButtonSelect.appendChild(option);
    // });
}

export async function openDevTools() {
    await invoke('open_devtools');
}

export async function resetSettings() {
    // TODO: 重置设置
}

export const openGithubLink = () => {
    invoke("open_url", { url: "https://github.com/Hotakus/XialloControl" });
};

export async function saveDeadzoneSettings() {
    try {
        await invoke("update_deadzone", {
            deadzone: state.current_preset.items.deadzone,
            deadzoneLeft: state.current_preset.items.deadzone_left
        });
        updateStatusMessage("摇杆死区已保存", false);
    } catch (error) {
        console.error("保存摇杆死区失败:", error);
        updateStatusMessage(`保存摇杆死区失败: ${error}`, true);
    }
}


export async function switchPreset() {
    if (!invoke) return;

    try {
        const preset = await invoke<Preset>("switch_to_preset", { name: state.previousPreset });
        state.current_preset = preset;
        await refreshMappings();
        console.log("Switched to preset:", preset);
    } catch (error) {
        console.error("Failed to switch preset:", error);
    }
}

export async function savePreset() {
    try {
        // TODO: 实现保存预设功能
        console.log("保存预设");
        updateStatusMessage("预设已保存", false);
    } catch (error) {
        console.error("保存预设失败:", error);
        updateStatusMessage(`保存预设失败: ${error}`, true);
    }
}

export async function confirmNewPreset() {
    if (!state.newPresetName.trim()) {
        updateStatusMessage("方案名称不能为空", true);
        return;
    }

    if (state.presets.includes(state.newPresetName)) {
        updateStatusMessage("方案名称已存在", true);
        return;
    }

    try {
        // 调用后端创建新预设
        await invoke<Preset>("create_preset", { name: state.newPresetName });
        state.presets.push(state.newPresetName);
        state.previousPreset = state.newPresetName;
        await switchPreset();
        updateStatusMessage(`方案 "${state.newPresetName}" 创建成功`, false);
    } catch (error) {
        state.previousPreset = "default";
        await switchPreset();
        console.error("创建预设失败:", error);
        updateStatusMessage(`创建预设失败: ${error}`, true);
    } finally {
        // 无论成功失败都重置状态
        state.isCreatingNewPreset = false;
        state.newPresetName = "";
        // 刷新映射列表
        await refreshMappings();
    }
}

export function cancelNewPreset() {
    state.isCreatingNewPreset = false;
    state.newPresetName = "";
}

export async function importPreset() {
    try {
        // TODO: 实现导入预设功能
        console.log("导入预设");
        updateStatusMessage("预设已导入", false);
    } catch (error) {
        console.error("导入预设失败:", error);
        updateStatusMessage(`导入预设失败: ${error}`, true);
    }
}


export async function newPreset() {
    try {
        // 启动新建预设模式
        state.isCreatingNewPreset = true;
        state.newPresetName = "";

        // 等待 Vue 更新 DOM 后聚焦输入框
        await nextTick();
        const input = document.querySelector('.preset-input') as HTMLInputElement;
        if (input) {
            input.focus();
        }
    } catch (error) {
        console.error("新建预设失败:", error);
        updateStatusMessage(`新建预设失败: ${error}`, true);
    }
}