// src/ts/ControllerGraph.ts

import {appWindow, state} from "@/ts/global_states.ts";
import {computed, nextTick, watch} from "vue";

// 这个操作只需要在模块加载时执行一次，所以直接放在顶层作用域
const controllerSvgs = import.meta.glob('/src/assets/controller/*.svg', {eager: true, import: 'default'});

/**
 * 根据名称获取预加载的SVG组件
 * @param name - SVG文件的名字 (例如 "xbox")
 */
function getSvgComponent(name: string = "xbox") {
    // 构造在 import.meta.glob 结果中的 key
    const key = `/src/assets/controller/${name.toLowerCase()}.svg`;

    // 尝试获取对应的SVG，如果找不到，就返回一个默认的 (比如xbox)
    return controllerSvgs[key] || controllerSvgs['/src/assets/controller/xbox.svg'];
}

// ✨ 这就是魔法发生的地方！✨
// 创建一个计算属性，它会依赖 state.isConnected 和 state.deviceSelected
export const currentControllerSvg = computed(() => {
    // 1. 如果设备未连接，或者没有设备类型信息
    if (!state.isConnected || !state.deviceSelected?.controller_type) {
        console.log("未连接，显示默认Xbox SVG");
        return getSvgComponent(); // 返回默认的 Xbox SVG
    }

    // 2. 如果已连接，根据设备类型动态返回对应的SVG
    const controllerType = state.deviceSelected.controller_type;
    console.log(`已连接，设备类型: ${controllerType}，正在查找对应SVG...`);

    return getSvgComponent(controllerType);
});


export enum ControllerButtons {
    // Face buttons
    South = 0,
    East = 1,
    West = 2,
    North = 3,

    // Shoulder buttons
    LB = 4,
    RB = 5,

    // Trigger
    LT,
    RT,

    // Thumb buttons
    LStick,
    RStick,

    // Back button
    Back,

    // Start button
    Start,

    // Guide button
    Guide,

    // D-pad
    Left,
    Right,
    Up,
    Down
}

const controllerSvgsBtnElements = {
    'svg-gamepad-btn-west-frame': ControllerButtons.West,
    'svg-gamepad-btn-north-frame': ControllerButtons.North,
    'svg-gamepad-btn-east-frame': ControllerButtons.East,
    'svg-gamepad-btn-south-frame': ControllerButtons.South,
    'svg-gamepad-btn-select': ControllerButtons.Start,
    'svg-gamepad-btn-share': ControllerButtons.Back,
    'svg-gamepad-btn-leftstick': ControllerButtons.LStick,
    'svg-gamepad-btn-rightstick': ControllerButtons.RStick,
    'svg-gamepad-btn-lb': ControllerButtons.LB,
    'svg-gamepad-btn-rb': ControllerButtons.RB,
    'svg-gamepad-btn-guide': ControllerButtons.Guide,
    'svg-gamepad-dpad-up': ControllerButtons.Up,
    'svg-gamepad-dpad-right': ControllerButtons.Right,
    'svg-gamepad-dpad-down': ControllerButtons.Down,
    'svg-gamepad-dpad-left': ControllerButtons.Left
}

const controllerSvgsPressureElements = {
    'svg-gamepad-leftstick': ControllerButtons.LStick,
    'svg-gamepad-rightstick': ControllerButtons.RStick,
    'svg-gamepad-lefttrigger': ControllerButtons.LB,
    'svg-gamepad-righttrigger': ControllerButtons.RB
}

export function initControllerGraph() {
    // TODO: 初始化
    for (let [id, btn] of Object.entries(controllerSvgsBtnElements)) {
        const element = document.querySelector<HTMLElement>(`#${id}`);
        if (element)
            element.style.fillOpacity = '0';
    }

    for (let [id, btn] of Object.entries(controllerSvgsPressureElements)) {
        const element = document.querySelector<HTMLElement>(`#${id}`);
        if (element)
            element.style.fillOpacity = '0';
    }
}


/**
 * 更新扳机的视觉效果
 * @param triggerId
 * @param value 范围从0 到 1
 */
function updateTriggerVisual(triggerId: string, value: number) {
    const element = document.querySelector<SVGGElement>(`#${triggerId}`);
    if (!element) {
        return;
    }

    const opacity = Math.min(Math.max(value, 0), 1) * 0.6;
    element.style.fillOpacity = opacity.toString();

    let angle = value * 40; // 最大旋转10度 (可以根据喜好调整)

    let rect = element.getBBox();
    let pivotX = 0;
    let pivotY = rect.y / 2 + rect.height * 3.2 / 3; // 底部中心点

    switch (triggerId) {
        case 'svg-gamepad-lefttrigger': {
            pivotX = rect.x + rect.width / 2;
            angle = -angle; // 左扳机向内旋转
            break;
        }
        case 'svg-gamepad-righttrigger': {
            pivotX = rect.x + rect.width / 2;
            break;
        }
    }

    element.style.transformStyle = 'preserve-3d';
    element.style.transformOrigin = `${pivotX}px ${pivotY}px`; // 以底部中心为旋转点
    // element.style.transform = `rotate(${angle}deg)`;
    element.style.transform = `perspective(1200px) rotateX(${angle}deg)`;
}


function updateStickVisual(stickId: string, x: number, y: number) {
    const element = document.querySelector<SVGGElement>(`#${stickId}`);
    if (!element) {
        return;
    }
    const opacity = Math.min(Math.max(Math.sqrt(x * x + y * y), 0), 1) * 0.55;
    element.style.fillOpacity = opacity.toString();

    let groupId = stickId + '-g';
    let groupElement = document.querySelector<SVGGElement>(`#${groupId}`);
    if (!groupElement) {
        return;
    }

    let groupRect = groupElement.getBBox();
    let groupCenterX = groupRect.x + groupRect.width / 2;
    let groupCenterY = groupRect.y + groupRect.height / 2;
    let groupRadius = Math.min(groupRect.width, groupRect.height) / 2;

    let elementRect = element.getBBox();
    let elementCenterX = elementRect.x + elementRect.width / 2;
    let elementCenterY = elementRect.y + elementRect.height / 2;
    let elementRadius = Math.min(elementRect.width, elementRect.height) / 2;

    let maxOffset = groupRadius - elementRadius;

    let offsetX = x * maxOffset;
    let offsetY = -y * maxOffset; // Y轴反向
    element.style.transform = `translate(${offsetX}px, ${offsetY}px)`;
    element.style.transformOrigin = `${elementCenterX}px ${elementCenterY}px`;
}

export function checkBit(num: number, bit: number): boolean {
    return (num & (1 << bit)) !== 0;
}

watch(() => state.current_controller_datas, async (newVal) => {
    await nextTick();

    // 确保 SVG 容器和它的子元素已经渲染完成
    const container = document.querySelector('.controller-svg') as HTMLElement;
    if (!container || !container.firstChild) {
        console.log("容器或SVG还没准备好，等待下一次更新...");
        return;
    }

    // --- 循环这个映射，更新DOM ---
    for (const [id, btnBit] of Object.entries(controllerSvgsBtnElements)) {
        let isPressed = checkBit(newVal.buttons, btnBit);
        const element = container.querySelector<HTMLElement>(`#${id}`);
        if (element) {
            element.style.transition = 'fill-opacity 0.13s ease';
            element.style.fillOpacity = isPressed ? '0.5' : '0';
        }
    }

    // 更新摇杆和扳机的压力
    // 设计一个扳机触发函数
    updateTriggerVisual('svg-gamepad-lefttrigger', newVal.left_trigger.value);
    updateTriggerVisual('svg-gamepad-righttrigger', newVal.right_trigger.value);

    updateStickVisual('svg-gamepad-leftstick', newVal.left_stick.x, newVal.left_stick.y);
    updateStickVisual('svg-gamepad-rightstick', newVal.right_stick.x, newVal.right_stick.y);
}, {deep: true, immediate: true});


watch(currentControllerSvg, async (newVal, oldVal) => {
    // 等待下一个 DOM 更新周期
    await nextTick();
    // 初始化控制器图形
    initControllerGraph();
}, {immediate: true});


// CompactPressureDatas 接口
export interface CompactPressureDatas {
    left_stick_x: number;
    left_stick_y: number;
    right_stick_x: number;
    right_stick_y: number;
    left_trigger: number;
    right_trigger: number;
}

// CompactControllerDatas 接口
export interface CompactControllerDatas {
    buttons: number;
    pressure: CompactPressureDatas;
}

function update_with_compact_datas(datas: CompactControllerDatas) {
    state.current_controller_datas.buttons = datas.buttons;
    state.current_controller_datas.left_stick.x = datas.pressure.left_stick_x;
    state.current_controller_datas.left_stick.y = datas.pressure.left_stick_y;
    state.current_controller_datas.right_stick.x = datas.pressure.right_stick_x;
    state.current_controller_datas.right_stick.y = datas.pressure.right_stick_y;
    state.current_controller_datas.left_trigger.value = datas.pressure.left_trigger;
    state.current_controller_datas.right_trigger.value = datas.pressure.right_trigger;
}

appWindow.listen('update_controller_compact_datas', (event) => {
    let datas = event.payload as CompactControllerDatas;
    // console.log(`update_controller_compact_datas: ${datas}`);
    update_with_compact_datas(datas);
});

// appWindow.listen('update_controller_pressure_datas', (event) => {
//     let datas = event.payload;
//     console.log(`update_controller_pressure_datas: ${datas}`);
// });
