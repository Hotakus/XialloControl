// src/global_states.ts
import {nextTick, reactive} from "vue";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {invoke} from "@tauri-apps/api/core";
import {DeviceInfo} from "@/ts/LeftPanel.ts";
import { locale } from "@tauri-apps/plugin-os";

let appWindow = getCurrentWindow();

export interface LastConnectedDevice {
    vid: number;
    pid: number;
    sub_pid: number;
}

export interface Preset {
    name: string;
    items: {
        deadzone: number;
        deadzone_left: number;
        mappings: any[];
    }
}

// ---------- 响应式应用状态 ----------
export const state = reactive({
    version: '0.0.0',
    is_release_env: false,
    locale: 'zh-CN',
    newVersionInfo: null as string | null,
    showNewVersionButton: false,

    titlebar_visible: true,

    statusMessage: '选择设备后点击连接按钮',
    statusMessageIsError: false,
    statusMessageIsSuccess: false,

    activeTab: 'buttonMapTab',

    autoStart: false,
    minimizeToTray: false,
    rememberLastConnection: false,
    lastConnectedDevice: null as LastConnectedDevice | null,
    theme: 'light',
    pollingFrequency: 125,
    previousPreset: "default",

    connectButtonDisabled: false,

    // 按键映射模态窗口相关
    showMappingModal: false,
    modalErrorVisible: false,
    modalErrorMessage: '',
    modalTitle: '',
    keyListenerActive: false,
    keyDetectorText: '点击此处并按下键盘按键、鼠标按键或滚动滚轮',
    keyDisplayText: '',
    selectedButton: "",
    isMouseKey: false,
    isScanning: false,

    showUpdateModal: false,
    showCaliModal: false,

    buttonsText: [{value: '', text: ''}],

    // main.js 原有的状态
    hasUserSelectedDevice: false,
    currentDevices: [] as DeviceInfo[],
    deviceSelected: null as DeviceInfo | null,
    deviceSelectedIndex: "null",
    connectIcon: "",

    isConnected: false,
    deviceType: 'xbox',
    mappings: [] as any[],
    editingMappingId: null as number | null,
    // 用于模态窗口中的触发状态绑定
    triggerState: {
        initial_interval: 300,
        min_interval: 100,
        acceleration: 0.8,
    },

    preventNextClick: false,
    currentKeys: {
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
        key: null as string | null
    },

    // 从 main.js 添加过来的新状态
    current_controller_datas: {
        buttons: 0,
        left_stick: {x: 0, y: 0, is_pressed: false},
        right_stick: {x: 0, y: 0, is_pressed: false},
        left_trigger: {value: 0, has_pressure: false, is_pressed: false},
        right_trigger: {value: 0, has_pressure: false, is_pressed: false},
        left_stick_center: [0, 0],
        right_stick_center: [0, 0],
        limits: {
            sticks_value_min: -0.0,
            sticks_value_max: 0.0,
            triggers_value_min: 0.0,
            triggers_value_max: 0.0,
        },
        is_acting: false
    },

    current_preset: {
        name: "",
        items: {
            deadzone: 0,
            deadzone_left: 0,
            mappings: []
        }
    } as Preset
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
        'close-modal',
        'cancel-btn',

        // 摇杆校准相关
        'open-joystick-cali-modal',
        'joystick-cali-modal',
        'close-joystick-cali-modal',
        'cancel-joystick-cali-btn',
        'joystick-left',
        'joystick-right',
        'handle-left',
        'handle-right',
        'deadzone-cali-left',
        'deadzone-cali-right',
        'progress-x-left',
        'progress-y-left',
        'progress-x-left-value',
        'progress-y-left-value',
        'progress-x-right',
        'progress-y-right',
        'progress-x-right-value',
        'progress-y-right-value',

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
