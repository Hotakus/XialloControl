import {state} from "@/ts/global_states.ts";

let current_controller_datas = {
    buttons: 0,
    left_stick: {x: 0, y: 0, is_pressed: false},
    right_stick: {x: 0, y: 0, is_pressed: false},
    left_trigger: {value: 0, has_pressure: false, is_pressed: false},
    right_trigger: {value: 0, has_pressure: false, is_pressed: false},
    left_stick_center: [0, 0],
    right_stick_center: [0, 0],
    limits: {
        sticks_value_min: -0.0,
        sticks_value_max: 0.0,
        triggers_value_min: 0.0,
        triggers_value_max: 0.0,
    },
    is_acting: false
};

// let leftStickDeadzone = document.querySelector('#deadzone-cali-left');
// let rightStickDeadzone = document.querySelector('#deadzone-cali-right');
// let leftStick = document.querySelector('#handle-left');
// let rightStick = document.querySelector('#handle-right');
// let leftStickArea = document.querySelector('#joystick-left');
// let rightStickArea = document.querySelector('#joystick-right');
//
// let leftStickCenter = parseFloat(window.getComputedStyle(leftStick).width) / 2;
// let rightStickCenter = parseFloat(window.getComputedStyle(rightStick).width) / 2;
// let leftStickAreaCenter = parseFloat(window.getComputedStyle(leftStickArea).width) / 2 - leftStickCenter;
// let rightStickAreaCenter = parseFloat(window.getComputedStyle(rightStickArea).width) / 2 - rightStickCenter;
// let cali_ui_is_show = false;
//
// // 添加进度条更新函数
// function updateProgressBar(value, progressId, valueId, axis) {
//     const progressFill = document.getElementById(progressId);
//     const progressValue = document.getElementById(valueId);
//
//     // 计算填充高度（0-100%）
//     const fillHeight = Math.abs(value) * 100;
//
//     // 关键点：根据正负值调整 transform-origin 和 scaleY 实现正值向上填充，负值向下填充
//     if (value >= 0) {
//         progressFill.style.transformOrigin = 'bottom';
//         progressFill.style.transform = `scaleY(${fillHeight / 100})`;
//     } else {
//         progressFill.style.transformOrigin = 'top';
//         progressFill.style.transform = `scaleY(${fillHeight / 100})`;
//     }
//
//     // 高度固定100%，用 scaleY 控制填充比例
//     progressFill.style.height = '100%';
//
//     // 更新数值显示
//     progressValue.textContent = `${axis}: ${value.toFixed(2)}`;
// }
//
//
// appWindow.listen('update_controller_data', async (event) => {
//     current_controller_datas = event.payload;
//
//     if (cali_ui_is_show) {
//         leftStick.style.left = (leftStickAreaCenter + current_controller_datas.left_stick.x * leftStickAreaCenter) + 'px';
//         leftStick.style.top = (leftStickAreaCenter - current_controller_datas.left_stick.y * leftStickAreaCenter) + 'px';
//         rightStick.style.left = (rightStickAreaCenter + current_controller_datas.right_stick.x * rightStickAreaCenter) + 'px';
//         rightStick.style.top = (rightStickAreaCenter - current_controller_datas.right_stick.y * rightStickAreaCenter) + 'px';
//
//         // 更新左摇杆进度条
//         updateProgressBar(
//             current_controller_datas.left_stick.x,
//             'progress-x-left',
//             'progress-x-left-value',
//             'X'
//         );
//         updateProgressBar(
//             current_controller_datas.left_stick.y,
//             'progress-y-left',
//             'progress-y-left-value',
//             'Y'
//         );
//
//         // 更新右摇杆进度条
//         updateProgressBar(
//             current_controller_datas.right_stick.x,
//             'progress-x-right',
//             'progress-x-right-value',
//             'X'
//         );
//         updateProgressBar(
//             current_controller_datas.right_stick.y,
//             'progress-y-right',
//             'progress-y-right-value',
//             'Y'
//         );
//
//         // console.log(current_controller_datas);
//         // console.log("---", leftStickCenterPX, rightStickCenterPX);
//         let controller_deadzone = await invoke("check_controller_deadzone");
//         leftStickDeadzone.textContent = (controller_deadzone[0] * 100).toFixed(1);
//         rightStickDeadzone.textContent = (controller_deadzone[1] * 100).toFixed(1);
//         // console.log("---", a);
//     }
// });
//
// // 打开模态窗口按钮事件绑定
// document.getElementById('open-joystick-cali-modal').addEventListener('click', () => {
//     document.getElementById('joystick-cali-modal').classList.add('active');
//     cali_ui_is_show = true;
// });
// // 关闭模态窗口按钮
// document.getElementById('close-joystick-cali-modal').addEventListener('click', () => {
//     document.getElementById('joystick-cali-modal').classList.remove('active');
//     cali_ui_is_show = false;
// });
// document.getElementById('cancel-joystick-cali-btn').addEventListener('click', () => {
//     document.getElementById('joystick-cali-modal').classList.remove('active');
//     cali_ui_is_show = false;
// });


export function openCaliModal(){
    state.showCaliModal = true;
}

export function closeCaliModal(){
    state.showCaliModal = false;
}
