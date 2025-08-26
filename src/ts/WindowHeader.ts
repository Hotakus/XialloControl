// src/windowHeader.ts
import {appWindow, invoke, state} from "@/ts/global_states";
import {updateStatusMessage} from "@/ts/LeftPanel";

// ---------- titlebar 显示隐藏逻辑 ----------
export async function updateTitlebar() {
    if (!invoke) return;
    const platform = await invoke<string>("get_platform");
    if (platform === "windows") {
        state.titlebar_visible = true;
    } else if (platform === "linux") {
        state.titlebar_visible = true; // TODO: 调整窗口平台差异 false
    } else {
        // TODO: macOS titlebar
        state.titlebar_visible = true;
    }
}

// ---------- 窗口按钮事件 ----------
export async function minimize() {
    console.log("minimize button clicked");
    await appWindow.minimize();
}

export async function maximize() {
    console.log("maximize button clicked");
    await appWindow.toggleMaximize();
}

export async function close() {
    console.log("close button clicked");
    if (state.minimizeToTray) await appWindow.hide();
    else await appWindow.close();
}

// ---------- 更新逻辑 ----------
export async function checkUpdate() {
    if (!invoke) return;
    try {
        const newVersion = await invoke<string | null>('check_update');
        if (newVersion) {
            console.log(`New version available: ${newVersion}`);
            state.newVersionInfo = newVersion;
            state.showNewVersionButton = true;
        } else {
            console.log('No new version available.');
            state.showNewVersionButton = false;
        }
    } catch (error) {
        console.error('Check update failed:', error);
        state.showNewVersionButton = false;
    }
}

export async function performUpdate() {
    if (!invoke) return;
    try {
        updateStatusMessage('正在下载并安装更新...');
        await invoke('perform_update');
        // 后端会在更新成功后自动重启
        updateStatusMessage('更新成功，正在重启应用...');
    } catch (error) {
        console.error('Perform update failed:', error);
        updateStatusMessage(`更新失败: ${error}`, true);
    }
}
