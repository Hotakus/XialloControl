// src/global_states.ts
import {reactive, nextTick} from "vue";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {invoke} from "@tauri-apps/api/core";

let appWindow = getCurrentWindow();


// ---------- 响应式应用状态 ----------
export const state = reactive({
    hasUserSelectedDevice: false,
    currentDevices: [] as string[],
    deviceSelected: null as string | null,
    minimizeToTray: false,
    isConnected: false,
    deviceType: 'xbox',
    mappings: [] as any[],
    editingMappingId: null as number | null,
    keyListenerActive: false,
    preventNextClick: false,
    currentKeys: {
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
        key: null as string | null
    }
});

// ---------- 延迟获取 DOM 元素 ----------
export const uiElements: Record<string, HTMLElement | null> = {};

export async function initUIElements() {
    await nextTick(); // 等 Vue 渲染完成再获取
    const ids = [
        // 窗口控制相关
        'titlebar',
        'minimize-button',
        "maximize-button",
        'close-button',

        // 设备连接相关
        'device',
        'scan-button',
        'connect-button',
        'status-message',
        'status-indicator',

        // 预设管理相关
        'preset',
        'save-preset',
        'import-preset',

        // 按键映射相关
        'add-button-map',
        'button-mapping-list',
        'controller-button',
        'key-detector-area',
        'key-display',

        // 摇杆设置相关
        'deadzone',
        'deadzone-value',
        'deadzone-left',
        'deadzone-left-value',

        // 软件设置相关
        'auto-start',
        'minimize-to-tray',
        'polling-frequency',
        'theme',

        // 模态窗口相关
        'mapping-modal',
        'modal-title',
        'modal-error',

        // 其他
        'github-link'
    ];
    ids.forEach(id => {
        uiElements[id] = document.getElementById(id);
    });
    uiElements['tabs'] = document.querySelector('.tabs');
}

// ---------- Tauri API ----------
export {appWindow, invoke};
