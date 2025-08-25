import { reactive, ref } from 'vue';
import { state } from "@/ts/global_states.ts";
import { invoke } from "@tauri-apps/api/core";
import { updateStatusMessage } from "@/ts/LeftPanel.ts";

// --- 响应式状态定义 ---

// 当前正在校准的摇杆 ('left', 'right', 或 'none')
export const calibratingStick = ref<'left' | 'right' | 'none'>('none');

// 当前校准步骤
export const currentStep = ref('Idle');

// 后端返回的完整校准状态
export const calibrationState = reactive({
    left_stick: { step: 'Idle', stick_center: { 0: 0, 1: 0 }, stick_range: { x_min: 0, x_max: 0, y_min: 0, y_max: 0 } },
    right_stick: { step: 'Idle', stick_center: { 0: 0, 1: 0 }, stick_range: { x_min: 0, x_max: 0, y_min: 0, y_max: 0 } },
});

// 用于显示在UI上的提示信息
export const calibrationHint = ref('请选择一个摇杆开始校准');

let pollInterval: number | null = null;

// --- 私有函数 ---

// 定期从后端获取最新校准状态
async function pollCalibrationState() {
    try {
        const newState = await invoke('get_calibration_state');
        Object.assign(calibrationState, newState);

        const stick = calibratingStick.value;
        if (stick !== 'none') {
            const step = calibrationState[stick === 'left' ? 'left_stick' : 'right_stick'].step;
            currentStep.value = step;
            updateHint(step);
        }
    } catch (e) {
        console.error("Failed to poll calibration state:", e);
    }
}

// 根据当前步骤更新提示文本
function updateHint(step: string) {
    switch (step) {
        case 'Idle':
            calibrationHint.value = '请选择一个摇杆开始校准';
            break;
        case 'CenterCheck':
            calibrationHint.value = '第一步：请完全松开摇杆，不要触碰，然后点击“下一步”。';
            break;
        case 'RangeDetection':
            calibrationHint.value = '第二步：请将摇杆推到边缘，并沿着边缘画几圈，然后点击“完成”。';
            break;
        case 'Complete':
            calibrationHint.value = '校准完成！您可以点击“保存”来保存结果，或点击“取消”放弃更改。';
            break;
        default:
            calibrationHint.value = '';
    }
}

// 停止状态轮询
function stopPolling() {
    if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
    }
}

// --- 导出的公共函数 (供Vue组件使用) ---

export function openCaliModal() {
    state.showCaliModal = true;
}

export function closeCaliModal() {
    if (calibratingStick.value !== 'none') {
        cancelCalibration();
    }
    state.showCaliModal = false;
}

export async function startCalibration(stick: 'left' | 'right') {
    if (calibratingStick.value !== 'none') return;

    calibratingStick.value = stick;
    await invoke('start_stick_calibration', { stickSide: stick });
    
    if (!pollInterval) {
        pollInterval = window.setInterval(pollCalibrationState, 100);
    }
    await pollCalibrationState();
}

export async function nextStep() {
    if (calibratingStick.value === 'none') return;
    await invoke('next_stick_calibration_step', { stickSide: calibratingStick.value });
    await pollCalibrationState();
}

export async function cancelCalibration() {
    if (calibratingStick.value === 'none') return;
    
    await invoke('cancel_stick_calibration', { stickSide: calibratingStick.value });
    calibratingStick.value = 'none';
    currentStep.value = 'Idle';
    updateHint('Idle');
    stopPolling();
}

export async function saveCalibration() {
    if (currentStep.value !== 'Complete') return;
    
    try {
        await invoke('save_current_calibration');
        updateStatusMessage('校准数据已成功保存！');
        console.log('校准数据已保存！');
        
        calibratingStick.value = 'none';
        currentStep.value = 'Idle';
        updateHint('Idle');
        stopPolling();
    } catch (e) {
        const errorMessage = `保存失败: ${e}`;
        updateStatusMessage(errorMessage, true);
        console.error("Failed to save calibration:", e);
    }
}

export async function resetToDefault() {
    try {
        await invoke('reset_calibration_to_default');
        updateStatusMessage('校准已恢复为默认设置。');
        console.log('校准已恢复为默认设置。');
        // 可以在这里选择性地重新加载校准状态以更新UI
        await pollCalibrationState(); 
    } catch (e) {
        const errorMessage = `恢复默认失败: ${e}`;
        updateStatusMessage(errorMessage, true);
        console.error("Failed to reset calibration:", e);
    }
}
