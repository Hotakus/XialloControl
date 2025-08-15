// src/windowHeader.ts
import {uiElements, state, invoke, appWindow} from "@/ts/global_states";

// ---------- titlebar 显示隐藏逻辑 ----------
export async function updateTitlebar() {
    if (!invoke) return;
    const platform = await invoke<string>("get_platform");
    const titlebar = uiElements['titlebar'];
    if (!titlebar) return;

    if (platform === "windows") {
        titlebar.classList.add("show");
        titlebar.classList.remove("hide");
    } else if (platform === "linux") {
        titlebar.classList.add("hide");
        titlebar.classList.remove("show");
    } else {
        console.warn(`未知平台: ${platform}`);
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
