// src/windowHeader.ts
import {appWindow, invoke, state} from "@/ts/global_states";

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
