import { state, UpdateInfo } from "./global_states";
import { invoke } from "@tauri-apps/api/core";

export function closeUpdateModal() {
    state.showUpdateModal = false;
    console.log("Update modal closed");
};

export function openUpdateModal() {
    state.showUpdateModal = true;
    console.log("Update modal opened");
}

export async function checkUpdate() {
    try {
        const newVersion = await invoke("check_update") as UpdateInfo | null;
        if (newVersion) {
            state.updateInfo = newVersion;
            console.log(`New version found: ${newVersion.version}`);
        } else {
            state.updateInfo = null; // 确保没有更新时状态为空
            console.log("No new version found");
        }
    } catch (error) {
        state.updateInfo = null;
        console.error("Failed to check for updates:", error);
    }
}