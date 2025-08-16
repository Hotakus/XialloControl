// src/windowHeader.ts
import {uiElements, state, invoke, appWindow} from "@/ts/global_states";

// ---------- titlebar 显示隐藏逻辑 ----------
export async function updateTitlebar() {
    if (!invoke) return;
    const platform = await invoke<string>("get_platform");
    if (platform === "windows") {
        state.titlebar_visible = true;
    } else if (platform === "linux") {
        state.titlebar_visible = false;
    } else {
        // TODO: macOS titlebar
        state.titlebar_visible = true;
    }
}

// ---------- 窗口按钮事件 ----------
export function setupWindowButtons() {
    let minimizeButton = uiElements['minimize-button'];
    let maximizeButton = uiElements['maximize-button'];
    let closeButton  = uiElements['close-button'];

    if (!minimizeButton || !closeButton || !maximizeButton) return;

    minimizeButton.addEventListener("click", async () => {
        console.log("minimize button clicked");
        await appWindow.minimize();
    });

    maximizeButton.addEventListener("click", async () => {
        console.log("maximize button clicked");
        await appWindow.toggleMaximize();
    });

    closeButton.addEventListener("click", async () => {
        console.log("close button clicked");
        if (state.minimizeToTray) await appWindow.hide();
        else await appWindow.close();
    });
}
