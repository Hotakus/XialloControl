import { updateStatusMessage } from "@/ts/LeftPanel.ts";
import { invoke } from "@tauri-apps/api/core";
import { appWindow, Preset, state } from "@/ts/global_states.ts";
import { locale } from "@tauri-apps/plugin-os";
import { setLanguage } from "@/ts/i18n.ts";

export async function initApp() {
    await queryLocale();
    await checkBuildEnv();
    await queryGlobalSettings();
    await queryPresetList();
    // await loadPreset();
    await queryMappings();
    updateStatusMessage("请选择一个设备并点击连接按钮");

    await invoke("try_auto_connect_last_device");
}

async function queryLocale() {
    if (!invoke) return;
    state.locale = await invoke("get_locale");
}

// 预设管理功能
export async function queryPresetList() {
    try {
        const presets: string[] = await invoke("check_presets_list");
        state.presets = presets;
        if (!state.presets.includes(state.previousPreset)) {
            state.previousPreset = "default";
        }

        const preset = await invoke<Preset>("load_preset", { name: state.previousPreset });
        state.current_preset = preset;

        console.log("预设列表:", preset);
    } catch (error) {
        console.error("加载预设列表失败:", error);
    }
}

export async function queryGlobalSettings() {
    if (!invoke) return;

    // camelCase 数据返回
    const settings = await invoke<{
        auto_start: boolean;
        minimize_to_tray: boolean;
        remember_last_connection: boolean;
        last_connected_device: { vid: number, pid: number, sub_pid: number } | null;
        theme: string;
        polling_frequency: number;
        previous_preset: string;
        calibration_mode: string;
        language: string;
    }>("get_current_settings");

    console.log("queryGlobalSettings", settings);

    state.autoStart = settings.auto_start || false;
    state.minimizeToTray = settings.minimize_to_tray || false;
    state.rememberLastConnection = settings.remember_last_connection || false;
    state.lastConnectedDevice = settings.last_connected_device || null;
    state.theme = settings.theme || "light";
    state.pollingFrequency = settings.polling_frequency || 125;
    state.previousPreset = settings.previous_preset || "default";
    state.calibration_mode = settings.calibration_mode || "square";
    state.language = settings.language || "system";

    // Init language
    let targetLocale = state.language;
    if (targetLocale === 'system') {
        targetLocale = state.locale; // e.g. 'zh-CN'
    }
    setLanguage(targetLocale);

    if (state.minimizeToTray) {
        appWindow?.hide();
    }

    // await invoke("set_frequency", { freq: state.pollingFrequency });
}

// 加载映射配置
export async function queryMappings() {
    if (!invoke) return;
    state.mappings = await invoke("get_mappings");
    console.log("Loaded mappings:", state.mappings);
}

export async function refreshMappings() {
    if (!invoke) return;
    state.mappings = await invoke("refresh_mappings");
    console.log("Refreshed mappings:", state.mappings);
}

async function checkBuildEnv() {
    if (!invoke) return;
    state.is_release_env = await invoke("is_release_env");
    if (state.is_release_env) {
        document.addEventListener('contextmenu', e => e.preventDefault());
    }
}
