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


/**
 * 保存设置
 */
export async function updateSettings() {
    const newSettings = {
        auto_start: state.autoStart,
        minimize_to_tray: state.minimizeToTray,
        theme: state.theme,
        polling_frequency: state.pollingFrequency,
        previous_preset: state.previous_preset_name
    };

    console.log("保存设置:", newSettings);

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


export async function openButtonMapModal(title = "添加按键映射", selectedButton = "", keyDisplayText = "", mappingId = null) {
    state.modalErrorVisible = false;
    state.modalErrorMessage = '';

    state.modalTitle = title;

    state.keyDisplayText = keyDisplayText;
    state.selectedButton = selectedButton;

    state.editingMappingId = mappingId;

    // TODO: 显示当前设备按钮映射
    // updateControllerButtons();
    state.showMappingModal = true;
}

export async function editButtonMap() {
    // TODO: 编辑按钮映射

}

export async function closeButtonMapModal() {
    stopKeyDetection(true);
    state.showMappingModal = false;
}


export async function saveButtonMap() {
    // TODO: 保存按钮映射
}


export async function addButtonMap() {
    // TODO: 添加按钮映射
    await openButtonMapModal();
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

    // TODO: test code
    // uiElements.keyDisplay.classList.toggle('mouse', isMouseKey);
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
function stopKeyDetection(resetText = true) {
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


function startKeyDetection() {
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


