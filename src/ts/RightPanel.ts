import { Preset, state } from "@/ts/global_states.ts";
import { invoke } from "@tauri-apps/api/core";
import { updateStatusMessage } from "@/ts/LeftPanel.ts";
import { queryMappings, refreshMappings } from "@/App.ts";
import { nextTick } from "vue";
import { setLanguage, translate } from "@/ts/i18n.ts";


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
        previous_preset: state.previousPreset,
        language: state.language
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


/**
 * 切换语言
 */
export async function changeLanguage() {
    let targetLocale = state.language;
    if (targetLocale === 'system') {
        targetLocale = state.locale; // e.g. 'zh-CN'
    }
    setLanguage(targetLocale);
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

        // 1. 恢复原始值和显示值
        state.rawKeyDisplayText = raw_key;
        const display_key = formatKeyDisplay(raw_key);

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

        // 4. 恢复 amount (如果存在)
        if (typeof mapping.amount === 'number') {
            // 使用绝对值，因为滑块只能表示大小，方向由按键本身决定
            state.mapping_amount = Math.abs(mapping.amount);
        } else {
            state.mapping_amount = 1; // 重置为默认值
        }

        // 5. 恢复 触发阈值，如果存在
        if (typeof mapping.trigger_theshold === 'number') {
            // 使用绝对值，因为滑块只能表示大小，方向由按键本身决定
            state.triggerTheshold = Math.abs(mapping.trigger_theshold);
        } else {
            state.mapping_amount = 0.3; // 重置为默认值
        }

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
    state.mapping_amount = 1; // 重置为默认值
    state.triggerTheshold = 0.3
    state.rawKeyDisplayText = ''; // 清空上次的按键检测结果
    state.keyDisplayText = '';
    await openButtonMapModal("添加按键映射");
}


export function formatKeyDisplay(rawKey: string): string {
    if (!rawKey) return '';
    return rawKey.split('+').map(part => {
        const translated = translate(`keyMappings.${part}`);
        // 如果翻译结果等于原始的key，说明没有找到翻译，使用大写作为备选
        return translated === `keyMappings.${part}` ? part.toUpperCase() : translated;
    }).join(' + ');
}

// 更新按键显示
function updateKeyDisplay() {
    const parts: string[] = [];
    if (state.currentKeys.ctrl) parts.push('Control');
    if (state.currentKeys.shift) parts.push('Shift');
    if (state.currentKeys.alt) parts.push('Alt');
    if (state.currentKeys.meta) parts.push('Meta');
    if (state.currentKeys.key) parts.push(state.currentKeys.key);

    // 更新原始文本用于逻辑判断
    state.rawKeyDisplayText = parts.join('+');

    // 更新显示文本用于UI
    state.keyDisplayText = formatKeyDisplay(state.rawKeyDisplayText);
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

    // 如果检测结果不是滚轮, 重置 amount (使用原始值判断)
    if (!state.rawKeyDisplayText.toLowerCase().includes('mousewheel')) {
        state.mapping_amount = 1;
    }
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
    if (e.deltaY < 0 || e.deltaX < 0) {
        state.currentKeys.key = 'MouseWheelUp';
    } else if (e.deltaY > 0 || e.deltaX > 0) {
        state.currentKeys.key = 'MouseWheelDown';
    }

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
    state.rawKeyDisplayText = '';

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
        { value: 'A' },
        { value: 'B' },
        { value: 'X' },
        { value: 'Y' },
        { value: 'LB' },
        { value: 'RB' },
        { value: 'LT' },
        { value: 'RT' },
        { value: 'Back' },
        { value: 'Start' },
        { value: 'Guide' },
        { value: 'DPadUp' },
        { value: 'DPadDown' },
        { value: 'DPadLeft' },
        { value: 'DPadRight' },
    ],
    ps: [
        { value: 'Cross' },
        { value: 'Circle' },
        { value: 'Square' },
        { value: 'Triangle' },
        { value: 'L1' },
        { value: 'R1' },
        { value: 'LT' },
        { value: 'RT' },
        { value: 'Share' },
        { value: 'Options' },
        { value: 'PS' },
        { value: 'DPadUp' },
        { value: 'DPadDown' },
        { value: 'DPadLeft' },
        { value: 'DPadRight' },
    ],
    switch: [ // 新增 Switch 布局
        { value: 'A' },
        { value: 'B' },
        { value: 'X' },
        { value: 'Y' },
        { value: 'L' },
        { value: 'R' },
        { value: 'LT' },
        { value: 'RT' },
        { value: 'Minus' },
        { value: 'Plus' },
        { value: 'Home' },
        { value: 'DPadUp' },
        { value: 'DPadDown' },
        { value: 'DPadLeft' },
        { value: 'DPadRight' },
    ]
}

// 预加载所有按键图标 SVG
const buttonIconSvgs = import.meta.glob(
    [
        '/src/assets/controller/playstation/*.svg',
        '/src/assets/controller/xbox/*.svg',
        '/src/assets/controller/switch/*.svg',
        '/src/assets/controller/icon/*.svg'
    ],
    { eager: true, import: 'default' }
);

/**
 * 根据按键名称获取对应的 SVG 图标组件
 * @param buttonName - 按键名称 (例如 "Cross", "Back")
 * @returns 对应的 SVG 组件, 如果找不到则返回 null
 */
export function getButtonIcon(buttonName: string) {
    if (buttonName.length <= 2) {
        // 名称长度小于等于 2 的按键不显示图标
        return null;
    }

    // 尝试在各个可能的路径中查找
    const possiblePaths = [
        `/src/assets/controller/playstation/${buttonName.toLowerCase()}.svg`,
        `/src/assets/controller/xbox/${buttonName.toLowerCase()}.svg`,
        `/src/assets/controller/switch/${buttonName.toLowerCase()}.svg`,
        `/src/assets/controller/icon/${buttonName.toLowerCase()}.svg`
    ];

    for (const path of possiblePaths) {
        if (buttonIconSvgs[path]) {
            return buttonIconSvgs[path];
        }
    }

    // 如果在所有路径中都找不到，返回 null
    console.warn(`未找到按键 "${buttonName}" 对应的 SVG 图标`);
    return null;
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

        // 保存当前预设到设置中，以便下次启动时恢复
        await updateSettings();
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


export async function deletePreset() {
    if (!state.previousPreset) return;

    try {
        await invoke("delete_preset", { name: state.previousPreset });
        state.presets = state.presets.filter(preset => preset !== state.previousPreset);
        updateStatusMessage(`方案 "${state.previousPreset}" 删除成功`, false);
        state.previousPreset = "default";
        await switchPreset();
    } catch (error) {
        console.error("删除预设失败:", error);
        updateStatusMessage(`删除预设失败: ${error}`, true);
    }
}

export async function editPreset() {
    state.showPresetEditModal = true;
}


/**
 * 更新映射顺序
 * @param newOrder 新的映射顺序数组
 */
export async function updateMappingsOrder(newOrder: any[]) {
    try {
        await invoke("update_mappings_order", { mappings: newOrder });
        updateStatusMessage("映射顺序已更新", false);
    } catch (error) {
        console.error("更新映射顺序失败:", error);
        updateStatusMessage(`更新映射顺序失败: ${error}`, true);
        // 如果失败，从后端重新加载一次，以恢复到失败前的状态
        await queryMappings();
    }
}


export async function updateStickAsMouse() {
    try {
        await invoke("update_stick_as_mouse", {
            useStickAsMouse: state.current_preset.items.use_stick_as_mouse,
            stickAsMouseSimulation: state.current_preset.items.stick_as_mouse_simulation
        });
        // updateStatusMessage("摇杆模拟鼠标设置已保存", false);
    } catch (error) {
        console.error("保存摇杆模拟鼠标设置失败:", error);
        updateStatusMessage(`保存摇杆模拟鼠标设置失败: ${error}`, true);
    }
}

export async function updateStickRotationThreshold() {
    try {
        await invoke("update_stick_rotation_threshold", {
            threshold: state.current_preset.items.stick_rotate_trigger_threshold
        });
        // updateStatusMessage("摇杆旋转阈值已保存", false);
    } catch (error) {
        console.error("保存摇杆旋转阈值失败:", error);
        updateStatusMessage(`保存摇杆旋转阈值失败: ${error}`, true);
    }
}

export async function updateMouseMoveSpeed() {
    try {
        await invoke("update_mouse_move_speed", {
            moveSpeed: state.current_preset.items.move_speed
        });
    } catch (error) {
        console.error("保存鼠标移动速度失败:", error);
        updateStatusMessage(`保存鼠标移动速度失败: ${error}`, true);
    }
}