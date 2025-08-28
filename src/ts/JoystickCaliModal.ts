import { nextTick, reactive, ref, watch } from 'vue';
import { state } from "@/ts/global_states.ts";
import { invoke } from "@tauri-apps/api/core";
import { updateStatusMessage } from "@/ts/LeftPanel.ts";

// --- 响应式状态定义 ---
export const calibratingStick = ref<'left' | 'right' | 'none'>('none');
export const currentStep = ref('Idle');
export const calibrationState = reactive({
    left_stick: { step: 'Idle', stick_center: { 0: 0, 1: 0 }, stick_range: { x_min: 0, x_max: 0, y_min: 0, y_max: 0 } },
    right_stick: { step: 'Idle', stick_center: { 0: 0, 1: 0 }, stick_range: { x_min: 0, x_max: 0, y_min: 0, y_max: 0 } },
});
export const calibrationHint = ref('请选择一个摇杆开始校准');
export const calibrationMode = ref<'circle' | 'square'>('square');

// --- 原有私有函数 ---
let pollInterval: number | null = null;
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
    } catch (e) { console.error("Failed to poll calibration state:", e); }
}
function updateHint(step: string) {
    switch (step) {
        case 'Idle': calibrationHint.value = '请选择一个摇杆开始校准'; break;
        case 'CenterCheck': calibrationHint.value = '第一步：请完全松开摇杆，不要触碰，然后点击“下一步”。'; break;
        case 'RangeDetection': calibrationHint.value = '第二步：请将摇杆推到边缘，并沿着边缘画几圈，然后点击“完成”。'; break;
        case 'Complete': calibrationHint.value = '校准完成！您可以点击“保存”来保存结果，或点击“取消”放弃更改。'; break;
        default: calibrationHint.value = '';
    }
}
function stopPolling() {
    if (pollInterval) { clearInterval(pollInterval); pollInterval = null; }
}
function updateJoystickAreaStyle() {
    const leftArea = document.getElementById('joystick-left');
    const rightArea = document.getElementById('joystick-right');
    if (leftArea && rightArea) {
        const borderRadius = calibrationMode.value === 'circle' ? '50%' : '15px';
        leftArea.style.borderRadius = borderRadius;
        rightArea.style.borderRadius = borderRadius;
    }
}

// --- 原有导出函数 ---
export function setCalibrationMode(mode: 'circle' | 'square') {
    calibrationMode.value = mode;
    state.calibration_mode = mode;
    invoke('set_calibration_mode', { mode: mode });
    updateJoystickAreaStyle();
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
        calibratingStick.value = 'none';
        currentStep.value = 'Idle';
        updateHint('Idle');
        stopPolling();
    } catch (e) { updateStatusMessage(`保存失败: ${e}`, true); }
}
export async function resetToDefault() {
    try {
        await invoke('reset_calibration_to_default');
        updateStatusMessage('校准已恢复为默认设置。');
        await pollCalibrationState();
    } catch (e) { updateStatusMessage(`恢复默认失败: ${e}`, true); }
}
export function openCaliModal() {
    state.showCaliModal = true;
    if (state.calibration_mode === 'square' || state.calibration_mode === 'circle') {
        calibrationMode.value = state.calibration_mode;
    }
    setTimeout(() => {
        updateJoystickAreaStyle();
    }, 100);
    closeCircularityTest();
}
export function closeCaliModal() {
    if (calibratingStick.value !== 'none') {
        cancelCalibration();
    }
    state.showCaliModal = false;
    closeCircularityTest();
}

// ==================================================================
// --- 圆度测试模块 V10.0 (基于固定理论圆的优化算法) ---
// ==================================================================

export const isCircularityTestEnabled = ref(false);
export const circularityErrorTextLeft = ref('圆度误差: --%');
export const circularityErrorTextRight = ref('圆度误差: --%');

// ==================================================================
// --- 摇杆圆度/轨迹绘制模块 (重构) ---
// ==================================================================

interface CircularityPoint {
    x: number;
    y: number;
}

// 增加一个带半径的类型，用于优化计算
interface CircularityPointWithRadius extends CircularityPoint {
    r: number; // 半径的平方
}

// 统一管理与绘制相关的状态
const drawingState = {
    left: {
        ctx: null as CanvasRenderingContext2D | null,
        bufferCtx: null as CanvasRenderingContext2D | null, // 蓝色缓冲
        redBufferCtx: null as CanvasRenderingContext2D | null, // 红色缓冲
        points: new Map<number, CircularityPointWithRadius>(), // 使用Map存储最远点
        animationFrameId: null as number | null,
        lastPos: null as CircularityPoint | null,
    },
    right: {
        ctx: null as CanvasRenderingContext2D | null,
        bufferCtx: null as CanvasRenderingContext2D | null, // 蓝色缓冲
        redBufferCtx: null as CanvasRenderingContext2D | null, // 红色缓冲
        points: new Map<number, CircularityPointWithRadius>(), // 使用Map存储最远点
        animationFrameId: null as number | null,
        lastPos: null as CircularityPoint | null,
    },
};

// 算法配置常量
const CONFIG = {
    THEORETICAL_RADIUS: 1.0,           // 固定理论圆半径 ([-1,1] 范围的标准圆)
    MIN_DETECTION_RADIUS: 0.85,        // 最小检测半径
    MIN_ERROR_THRESHOLD: 0.05,         // 红色警告区域的触发阈值 (5%)，可在此处调整
    MIN_DATA_POINTS: 24,               // 最少数据点要求（约15度一个点）
    ANGLE_PRECISION: 3,                // 角度精度（每3度一个桶）
};

// --- 绘制模块核心函数 ---

/**
 * [重构] 初始化指定摇杆的绘制环境
 */
function setupDrawing(stick: 'left' | 'right') {
    const canvas = document.getElementById(`canvas-${stick}`) as HTMLCanvasElement;
    const state = drawingState[stick];

    if (canvas && !state.ctx) {
        const area = canvas.parentElement!;
        const rect = area.getBoundingClientRect();
        canvas.width = rect.width;
        canvas.height = rect.height;
        state.ctx = canvas.getContext('2d');

        // 创建并设置蓝色离屏缓冲画布
        const blueBuffer = document.createElement('canvas');
        blueBuffer.width = canvas.width;
        blueBuffer.height = canvas.height;
        state.bufferCtx = blueBuffer.getContext('2d');

        // 创建并设置红色离屏缓冲画布
        const redBuffer = document.createElement('canvas');
        redBuffer.width = canvas.width;
        redBuffer.height = canvas.height;
        state.redBufferCtx = redBuffer.getContext('2d');

        // 清空状态
        state.points.clear();
        state.lastPos = null;

        // 启动渲染循环
        if (state.animationFrameId === null) {
            renderLoop(stick);
        }
    }
}

/**
 * [重构] 销毁指定摇杆的绘制环境
 */
function teardownDrawing(stick: 'left' | 'right') {
    const state = drawingState[stick];
    if (state.animationFrameId !== null) {
        cancelAnimationFrame(state.animationFrameId);
        state.animationFrameId = null;
    }
    if (state.ctx) {
        state.ctx.clearRect(0, 0, state.ctx.canvas.width, state.ctx.canvas.height);
    }
    state.points.clear();
    state.lastPos = null;
    state.ctx = null;
    state.bufferCtx = null;
    state.redBufferCtx = null;
}

/**
 * [最终方案] 统一的渲染循环，采用双缓冲绘制方案
 */
function renderLoop(stick: 'left' | 'right') {
    const stickState = drawingState[stick];
    const ctx = stickState.ctx;
    const bufferCtx = stickState.bufferCtx;
    const redBufferCtx = stickState.redBufferCtx;
    const currentData = state.current_controller_datas;
    const stickData = stick === 'left' ? currentData.left_stick : currentData.right_stick;

    if (!ctx || !bufferCtx || !redBufferCtx) return;

    const centerX = ctx.canvas.width / 2;
    const centerY = ctx.canvas.height / 2;
    const currentPos = { x: stickData.x, y: stickData.y };
    const lastPos = stickState.lastPos;

    // 1. 在各自的缓冲层上累积绘制
    if (lastPos && (currentPos.x !== lastPos.x || currentPos.y !== lastPos.y)) {
        const p1 = { x: centerX, y: centerY };
        const p2 = { x: centerX + lastPos.x * centerX, y: centerY - lastPos.y * centerY };
        const p3 = { x: centerX + currentPos.x * centerX, y: centerY - currentPos.y * centerY };

        // 在蓝色缓冲上绘制所有轨迹
        bufferCtx.fillStyle = 'rgb(76, 139, 245)';
        bufferCtx.beginPath();
        bufferCtx.moveTo(p1.x, p1.y);
        bufferCtx.lineTo(p2.x, p2.y);
        bufferCtx.lineTo(p3.x, p3.y);
        bufferCtx.closePath();
        bufferCtx.fill();

        // 仅在超出范围时，在红色缓冲上绘制
        const radius = Math.sqrt(currentPos.x * currentPos.x + currentPos.y * currentPos.y);
        if (radius > CONFIG.THEORETICAL_RADIUS + CONFIG.MIN_ERROR_THRESHOLD) {
            const excess = radius - (CONFIG.THEORETICAL_RADIUS + CONFIG.MIN_ERROR_THRESHOLD);
            // 使用平方函数使颜色变化更剧烈
            const excessRatio = Math.min(excess / 0.4, 1.0);
            const squaredRatio = excessRatio * excessRatio;
            // 颜色从非常浅的粉红(255,200,200)到纯红(255,0,0)
            const greenBlue = Math.floor(200 * (1 - squaredRatio));
            redBufferCtx.fillStyle = `rgb(255, ${greenBlue}, ${greenBlue})`;
            redBufferCtx.beginPath();
            redBufferCtx.moveTo(p1.x, p1.y);
            redBufferCtx.lineTo(p2.x, p2.y);
            redBufferCtx.lineTo(p3.x, p3.y);
            redBufferCtx.closePath();
            redBufferCtx.fill();
        }
    }

    // 2. 清空可见画布
    ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);

    // 3. 按顺序将缓冲层绘制到可见画布上
    ctx.globalAlpha = 0.3;
    ctx.drawImage(bufferCtx.canvas, 0, 0); // 先画蓝色
    ctx.drawImage(redBufferCtx.canvas, 0, 0); // 再叠加上红色
    ctx.globalAlpha = 1.0;

    // 4. 更新位置并请求下一帧
    stickState.lastPos = currentPos;
    stickState.animationFrameId = requestAnimationFrame(() => renderLoop(stick));
}


/**
 * 计算并返回指定摇杆的圆度误差文本
 */
function calculateCircularityError(stick: 'left' | 'right'): string {
    const pointsMap = drawingState[stick].points;
    const points = Array.from(pointsMap.values());

    if (points.length < CONFIG.MIN_DATA_POINTS) {
        return '数据不足';
    }

    // 为了简化，我们只计算外圈点的误差
    const outerPoints = points.filter(p => Math.sqrt(p.x * p.x + p.y * p.y) > CONFIG.MIN_DETECTION_RADIUS);
    if (outerPoints.length < CONFIG.MIN_DATA_POINTS) {
        return '外圈数据不足';
    }

    let totalDeviation = 0;
    let significantErrorCount = 0;

    // 仅统计超出标准圆范围的点的误差
    for (const point of outerPoints) {
        const distance = Math.sqrt(point.x * point.x + point.y * point.y);
        const deviation = Math.abs(distance - CONFIG.THEORETICAL_RADIUS);

        if (deviation > CONFIG.MIN_ERROR_THRESHOLD) {
            totalDeviation += deviation;
            significantErrorCount++;
        }
    }

    // 优化：计算平均偏差时，仅考虑超出阈值的点
    const avgDeviation = significantErrorCount > 0 ? totalDeviation / significantErrorCount : 0;
    const errorPercentage = avgDeviation * 100;

    return `圆度误差: ${errorPercentage.toFixed(2)}%`;
}


function closeCircularityTest() {
    if (isCircularityTestEnabled.value) {
        isCircularityTestEnabled.value = false;
    }
}

// --- 生命周期管理 ---

let errorUpdateInterval: number | null = null;

watch(isCircularityTestEnabled, async (newValue) => {
    if (newValue) {
        await nextTick();
        setupDrawing('left');
        setupDrawing('right');

        // 启动独立的误差计算定时器
        if (errorUpdateInterval) clearInterval(errorUpdateInterval);
        errorUpdateInterval = window.setInterval(() => {
            circularityErrorTextLeft.value = calculateCircularityError('left');
            circularityErrorTextRight.value = calculateCircularityError('right');
        }, 300);

    } else {
        teardownDrawing('left');
        teardownDrawing('right');

        if (errorUpdateInterval) {
            clearInterval(errorUpdateInterval);
            errorUpdateInterval = null;
        }
        // 保留最终结果
        circularityErrorTextLeft.value = calculateCircularityError('left');
        circularityErrorTextRight.value = calculateCircularityError('right');
    }
});

function updateJoystickVisualsInModal(stick: 'left' | 'right', x: number, y: number) {
    const handle = document.getElementById(`handle-${stick}`);
    const area = document.getElementById(`joystick-${stick}`);
    const line = document.getElementById(`line-${stick}`);

    if (handle && area) {
        const areaRect = area.getBoundingClientRect();
        const handleRect = handle.getBoundingClientRect();
        const maxOffset = (areaRect.width / 2) - (handleRect.width / 2);
        const clampedX = Math.max(-1.0, Math.min(1.0, x));
        const clampedY = Math.max(-1.0, Math.min(1.0, y));
        const offsetX = clampedX * maxOffset;
        const offsetY = -clampedY * maxOffset;

        handle.style.transform = `translate(${offsetX}px, ${offsetY}px)`;

        if (line) {
            const centerX = areaRect.width / 2;
            const centerY = areaRect.height / 2;
            line.setAttribute('x2', `${centerX + offsetX}`);
            line.setAttribute('y2', `${centerY + offsetY}`);
        }
    }

    // 更新进度条
    const progressX = document.getElementById(`progress-x-${stick}`);
    const progressY = document.getElementById(`progress-y-${stick}`);
    const valueX = document.getElementById(`progress-x-${stick}-value`);
    const valueY = document.getElementById(`progress-y-${stick}-value`);

    if (progressX && valueX) {
        const value = x;
        if (value >= 0) {
            progressX.style.height = `${value * 50}%`;
            progressX.style.bottom = '50%';
            progressX.style.backgroundColor = '#4c8bf5';
        } else {
            const height = -value * 50;
            progressX.style.height = `${height}%`;
            progressX.style.bottom = `${50 - height}%`;
            progressX.style.backgroundColor = '#e74c3c';
        }
        valueX.textContent = `X: ${value.toFixed(3)}`;
    }
    if (progressY && valueY) {
        const value = y;
        if (value >= 0) {
            progressY.style.height = `${value * 50}%`;
            progressY.style.bottom = '50%';
            progressY.style.backgroundColor = '#4c8bf5';
        } else {
            const height = -value * 50;
            progressY.style.height = `${height}%`;
            progressY.style.bottom = `${50 - height}%`;
            progressY.style.backgroundColor = '#e74c3c';
        }
        valueY.textContent = `Y: ${value.toFixed(3)}`;
    }
}

// --- 主监听器 ---
watch(() => state.current_controller_datas, (newData) => {
    if (!state.showCaliModal) return;

    // 原有功能
    updateJoystickVisualsInModal('left', newData.left_stick.x, newData.left_stick.y);
    updateJoystickVisualsInModal('right', newData.right_stick.x, newData.right_stick.y);

    // 新增功能挂载点
    if (isCircularityTestEnabled.value) {
        // [新逻辑] 为误差计算采集最远点数据
        const sticks: ('left' | 'right')[] = ['left', 'right'];
        for (const stick of sticks) {
            const stickData = stick === 'left' ? newData.left_stick : newData.right_stick;
            const pointsMap = drawingState[stick].points;

            const radiusSq = stickData.x * stickData.x + stickData.y * stickData.y;
            // 只记录外圈的点，避免中心点干扰
            if (radiusSq > CONFIG.MIN_DETECTION_RADIUS * CONFIG.MIN_DETECTION_RADIUS) {
                const angle = Math.atan2(stickData.y, stickData.x);
                const bucketKey = Math.round((angle * 180 / Math.PI) / CONFIG.ANGLE_PRECISION);

                const existing = pointsMap.get(bucketKey);
                if (!existing || radiusSq > existing.r) {
                    pointsMap.set(bucketKey, { x: stickData.x, y: stickData.y, r: radiusSq });
                }
            }
        }
    }

}, { deep: true });
