import {state} from "@/ts/global_states.ts";
import {invoke} from "@tauri-apps/api/core";
import {updateStatusMessage} from "@/ts/LeftPanel.ts";
import {queryMappings} from "@/App.ts";


/**
 * 切换标签页
 * @param tabName 要切换到的标签页名称
 */
export function switchTab(tabName: string) {
    state.activeTab = tabName;
}


export async function setPollingFrequency() {
    await invoke("set_frequency", {freq: state.pollingFrequency});
    await updateSettings();
}

/**
 * 保存设置
 */
export async function updateSettings() {
    const newSettings = {
        auto_start: state.autoStart,
        minimize_to_tray: state.minimizeToTray,
        theme: state.theme,
        polling_frequency: state.pollingFrequency,
        previous_preset: state.previousPreset
    };

    try {
        await invoke("update_settings", {newSettings});
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
    const mapping = state.mappings.find(m => m.id === id);
    if (mapping) {
        // --- 新增转换和状态恢复逻辑 ---
        const raw_key: string = mapping.composed_shortcut_key; // 获取英文原始值

        // 1. 转换为中文显示值，用于弹窗UI
        const display_key = raw_key.split(' + ').map(part => keyDisplayNames[part] || part.toUpperCase()).join(' + ');

        // 2. 反向解析原始值，恢复 state.currentKeys 状态
        const parts = raw_key.split(' + ');
        state.currentKeys = {ctrl: false, shift: false, alt: false, meta: false, key: null}; // 重置
        state.currentKeys.ctrl = parts.includes('Control');
        state.currentKeys.shift = parts.includes('Shift');
        state.currentKeys.alt = parts.includes('Alt');
        state.currentKeys.meta = parts.includes('Meta');
        // 查找不是修饰键的部分作为主键
        state.currentKeys.key = parts.find(p => !['Control', 'Shift', 'Alt', 'Meta'].includes(p)) || null;

        // 使用转换后的中文值和恢复的状态打开模态窗口
        console.log("编辑按钮映射", id);
        await openButtonMapModal("编辑按键映射", mapping.composed_button, display_key, mapping.id);
        // --- 逻辑结束 ---
    }
}

export async function deleteButtonMap(id: number) {
    invoke('delete_mapping', {id: id})
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
    state.currentKeys = {ctrl: false, shift: false, alt: false, meta: false, key: null};

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
        {value: 'A', text: 'A 按钮'},
        {value: 'B', text: 'B 按钮'},
        {value: 'X', text: 'X 按钮'},
        {value: 'Y', text: 'Y 按钮'},
        {value: 'LB', text: '左肩键 (LB)'},
        {value: 'RB', text: '右肩键 (RB)'},
        {value: 'LT', text: '左扳机 (LT)'},
        {value: 'RT', text: '右扳机 (RT)'},
        {value: 'START', text: '开始按钮'},
        {value: 'SELECT', text: '选择按钮'}
    ],
    ps: [
        {value: 'CROSS', text: '叉按钮 (CROSS)'},
        {value: 'CIRCLE', text: '圆按钮 (CIRCLE)'},
        {value: 'SQUARE', text: '方按钮 (SQUARE)'},
        {value: 'TRIANGLE', text: '三角按钮 (TRIANGLE)'},
        {value: 'L1', text: '左肩键 (L1)'},
        {value: 'R1', text: '右肩键 (R1)'},
        {value: 'L2', text: '左扳机 (L2)'},
        {value: 'R2', text: '右扳机 (R2)'},
        {value: 'OPTIONS', text: '选项按钮'},
        {value: 'SHARE', text: '分享按钮'}
    ],
    switchpro: [
        {value: 'B', text: 'B 按钮'},
        {value: 'A', text: 'A 按钮'},
        {value: 'Y', text: 'Y 按钮'},
        {value: 'X', text: 'X 按钮'},
        {value: 'L', text: '左肩键 (L)'},
        {value: 'R', text: '右肩键 (R)'},
        {value: 'ZL', text: '左扳机 (ZL)'},
        {value: 'ZR', text: '右扳机 (ZR)'},
        {value: 'PLUS', text: '加号按钮'},
        {value: 'MINUS', text: '减号按钮'}
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
        case "Switch": {
            state.buttonsText = buttonTextMapLists.switchpro;
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
