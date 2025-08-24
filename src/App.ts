import {updateStatusMessage} from "@/ts/LeftPanel.ts";
import {invoke} from "@tauri-apps/api/core";
import {appWindow, state} from "@/ts/global_states.ts";
import {locale} from "@tauri-apps/plugin-os";

export async function initApp() {
    await checkBuildEnv();
    await queryGlobalSettings();
    // loadPreset();

    await queryMappings();
    updateStatusMessage("请选择一个设备并点击连接按钮");

    await invoke("try_auto_connect_last_device");

    state.locale = await invoke("get_locale");
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
    }>("get_current_settings");

    console.log("queryGlobalSettings", settings);

    state.autoStart = settings.auto_start || false;
    state.minimizeToTray = settings.minimize_to_tray || false;
    state.rememberLastConnection = settings.remember_last_connection || false;
    state.lastConnectedDevice = settings.last_connected_device || null;
    state.theme = settings.theme || "light";
    state.pollingFrequency = settings.polling_frequency || 125;
    state.previousPreset = settings.previous_preset || "default";

    if (state.minimizeToTray) {
        appWindow?.hide();
    }

    // await invoke("set_frequency", { freq: state.pollingFrequency });
}

// 加载映射配置
export async function queryMappings() {
    if (!invoke) return;
    state.mappings = await invoke("get_mappings");
}

async function checkBuildEnv() {
    if (!invoke) return;
    state.is_release_env = await invoke("is_release_env");
    if (state.is_release_env) {
        document.addEventListener('contextmenu', e => e.preventDefault());
    }
}
