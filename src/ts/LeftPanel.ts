import {appWindow, state} from "@/ts/global_states.ts";
import {invoke} from "@tauri-apps/api/core";
import {updateControllerButtons} from "@/ts/RightPanel.ts";

// 更新状态消息
export function updateStatusMessage(message: string, isError = false) {
    state.statusMessage = message;

    // if (isError) {
    //     state.statusMessageIsError = isError;
    // } else if (!isError) {
    //     state.statusMessageIsSuccess = message.includes('成功');
    // }

    state.statusMessageIsError = isError;
    if (!isError) {
        state.statusMessageIsSuccess = message.includes('成功');
    } else {
        state.statusMessageIsSuccess = false;
    }
}

// 连接状态图标SVG
export const connectStatusIcons = {
    connected: `<svg t="1753591538385" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="19479" width="200" height="200"><path d="M728.96 269.44a207.36 207.36 0 1 0-414.08 0v133.12a239.36 239.36 0 0 1 96-93.44v-39.68a111.36 111.36 0 1 1 222.08 0v200.96a111.36 111.36 0 0 1-111.36 111.36 110.08 110.08 0 0 1-69.76-25.6v108.8a203.52 203.52 0 0 0 69.76 12.8 207.36 207.36 0 0 0 207.36-207.36z" fill="#ffffff" p-id="19480"></path><path d="M632.96 680.32v58.88a111.36 111.36 0 1 1-222.08 0V520.32a111.36 111.36 0 0 1 110.72-111.36 110.08 110.08 0 0 1 42.88 8.96 112.64 112.64 0 0 1 26.88 16.64v-108.8a204.8 204.8 0 0 0-69.76-12.8 207.36 207.36 0 0 0-206.72 207.36v219.52a207.36 207.36 0 1 0 414.08 0V588.16a238.72 238.72 0 0 1-96 92.16z" fill="#ffffff" p-id="19481"></path></svg>`,
    disconnected: `<svg t="1753595424804" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="19662" width="200" height="200"><path d="M521.6 158.08a111.36 111.36 0 0 1 111.36 111.36V480l96 96V269.44a206.72 206.72 0 0 0-394.24-87.68L411.52 256a110.72 110.72 0 0 1 110.08-97.92zM864 846.08l-135.04-135.04-96-96-222.08-222.08-96-96-135.04-134.4L112 230.4l202.88 202.24v306.56a206.72 206.72 0 0 0 394.24 87.68l87.04 87.04z m-341.76 4.48a111.36 111.36 0 0 1-111.36-111.36V528.64l221.44 221.44a110.72 110.72 0 0 1-110.72 100.48z" fill="#ffffff" p-id="19663"></path></svg>`
};

// 断开当前设备
export async function disconnectCurrentDevice() {
    if (!state.isConnected || !state.deviceSelected) return true;

    const deviceName = state.deviceSelected.name;
    updateStatusMessage(`正在断开设备: ${deviceName}...`);

    try {
        let res = await invoke("disconnect_device", {deviceName});
        if (res) {
            updateStatusMessage(`设备已断开`);
            state.isConnected = false;
            state.connectIcon = connectStatusIcons.disconnected;
            return true;
        } else {
            updateStatusMessage(`断开失败`, true);
            return false;
        }
    } catch (error) {
        console.log("断开连接失败:", error);
        updateStatusMessage(`断开失败`, true);
        return false;
    }
}


export async function onDeviceSelected() {
    console.log("onDeviceSelected " + state.deviceSelectedIndex);
    if (state.deviceSelectedIndex === "null") {
        state.deviceSelected = null;
        updateStatusMessage("请选择一个设备");
        return;
    }

    const selectedIndex = parseInt(state.deviceSelectedIndex);
    if (isNaN(selectedIndex)) {
        updateStatusMessage("无效的设备选择", true);
        return;
    }

    if (state.isConnected && state.deviceSelected) {
        const success = await disconnectCurrentDevice();
        if (!success) {
            state.deviceSelectedIndex = state.currentDevices.indexOf(state.deviceSelected).toString();
            return;
        }
    }

    if (selectedIndex >= 0 && selectedIndex < state.currentDevices.length) {
        state.deviceSelected = state.currentDevices[selectedIndex];
        state.hasUserSelectedDevice = true;
        updateStatusMessage(`已选择设备: ${state.deviceSelected.name}`);
    } else {
        state.deviceSelected = null;
        updateStatusMessage(`无效的设备索引: ${selectedIndex}`, true);
    }
}


// 去除设备名前缀
function stripDevicePrefix(name: string) {
    const index = name.indexOf(":");
    return index !== -1 ? name.slice(index + 1).trim() : name;
}

// Rust 的 enum => TypeScript 的 string union
export type ControllerType =
    | "Xbox"
    | "PlayStation"
    | "Switch"
    | "Betop"
    | "Other";

// Rust 的 struct => TypeScript 的 interface
export interface DeviceInfo {
    name: string;
    vendor_id: string;
    product_id?: string;
    sub_product_id?: string;
    uuid_is_invalid: boolean;
    device_path?: string;
    controller_type: ControllerType;
}


// 更新设备列表
// function updateDeviceList(devices: DeviceInfo[]) {
//     if (devices.length === 0) state.hasUserSelectedDevice = false;
//     if (state.hasUserSelectedDevice) {
//         state.isConnected = devices.length > 0;
//         return;
//     }
//     if (JSON.stringify(state.currentDevices) === JSON.stringify(devices)) return;
//
//     state.currentDevices = devices;
//     uiElements.deviceSelect.innerHTML = '';
//     const defaultOption = document.createElement('option');
//     defaultOption.textContent = '请选择设备';
//     defaultOption.disabled = true;
//     defaultOption.selected = true;
//     defaultOption.value = "null";
//     uiElements.deviceSelect.appendChild(defaultOption);
//
//     state.currentDevices.forEach((device, index) => {
//         const option = document.createElement('option');
//         option.value = index.toString();
//         option.textContent = `${index}: ${device.name}`;
//         uiElements.deviceSelect.appendChild(option);
//     });
// }


async function queryDevice() {
    const devices = await invoke<DeviceInfo[]>("query_devices");
    // updateDeviceList(devices);
    console.log(devices);
    if (devices.length === 0) state.hasUserSelectedDevice = false;
    if (state.hasUserSelectedDevice) {
        state.isConnected = devices.length > 0;
        return;
    }
    state.currentDevices = devices;
}

function resetDevice() {
    state.hasUserSelectedDevice = false;
    state.deviceSelectedIndex = "null";
    state.deviceSelected = null;
}

// 断开设备并查询新设备
export async function closeAndQueryDevice(deferReset = false, deferQuery = false) {
    await disconnectCurrentDevice();

    if (!deferReset) {
        resetDevice();
    }

    try {
        if (!deferQuery) {
            await queryDevice();
        }
    } catch (error) {
        console.error("查询设备失败:", error);
        updateStatusMessage("设备查询失败", true);
    }
}


export async function scanDevices() {
    state.isScanning = true;
    updateStatusMessage("正在扫描设备...");
    const icon = document.querySelector("#scan-button .icon");
    icon?.classList.add("spin");

    await closeAndQueryDevice();

    icon?.addEventListener("animationend", () => {
        icon.classList.remove("spin");
    }, {once: true});

    updateStatusMessage(`扫描完成！发现${state.currentDevices.length}个可用设备`);
    state.isScanning = false;
}


// 设备连接后更新映射配置
function updateMappingsForDevice(deviceName: string) {
    // state.deviceType = detectDeviceType(deviceName);
    updateControllerButtons();
    // renderMappings();
}

// 切换设备连接状态
export async function toggleDeviceConnection() {

    state.connectButtonDisabled = true;

    if (!state.deviceSelected || state.deviceSelectedIndex === "null") {
        updateStatusMessage("请先从下拉列表选择一个设备", true);
        // 恢复按钮状态
        state.connectButtonDisabled = false;
        return;
    }

    state.isConnected = !state.isConnected;
    const deviceName = stripDevicePrefix(state.deviceSelected.name);
    const controllerType = state.deviceSelected.controller_type;

    try {
        // 如果当前设备已连接，则断开连接
        if (state.isConnected) {
            updateStatusMessage(`正在连接设备: ${deviceName}...`);
            if (await invoke("use_device", {deviceName})) {
                updateStatusMessage(`设备 ${deviceName} 已成功连接`);
                state.isConnected = true;
                state.connectIcon = connectStatusIcons.connected;
                updateMappingsForDevice(controllerType);
            } else {
                updateStatusMessage(`连接失败`, true);
                state.isConnected = false;
                state.connectIcon = connectStatusIcons.disconnected;
            }
        } else {
            // 如果当前设备未连接，则连接设备
            updateStatusMessage(`正在断开设备: ${deviceName}...`);
            if (await invoke("disconnect_device", {deviceName})) {
                updateStatusMessage(`设备已断开`);
                state.isConnected = false;
                state.connectIcon = connectStatusIcons.disconnected;
            } else {
                updateStatusMessage(`断开失败--`, true);
                state.isConnected = true;
                state.connectIcon = connectStatusIcons.disconnected;
            }
        }
    } catch (error) {
        console.error("连接操作出错:", error);
        updateStatusMessage(`操作失败: ${error}`, true);
    } finally {
        // 无论成功失败都恢复按钮状态
        state.connectButtonDisabled = false;
    }
}

appWindow.listen("physical_connect_status", async (event) => {
    let name = event.payload;

    state.isConnected = false;
    state.connectIcon = connectStatusIcons.disconnected;

    console.log(`手柄 ${name} 物理断开连接`);
    updateStatusMessage(`手柄 ${name} 物理断开连接!`, true);

    resetDevice();
    await queryDevice();
})
