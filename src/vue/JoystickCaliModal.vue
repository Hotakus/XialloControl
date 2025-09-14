n<template>
  <div class="joystick-cali-overlay" :class="{active: state.showCaliModal}" id="joystick-cali-modal" @click.self="closeCaliModal()">
    <div class="joystick-cali-modal">
      <div class="modal-header">
        摇杆校准
        <button class="modal-close" @click="closeCaliModal()">&times;</button>
      </div>

      <div class="modal-body">
        <div class="modal-main-content">
          <!-- 左侧：两个摇杆 -->
          <div class="joysticks-container">
            <div class="joystick-display-group">
              <div style="text-align:center;">
                <label>左摇杆</label>
                <div class="joystick-area" id="joystick-left">
                  <canvas v-if="isCircularityTestEnabled" id="canvas-left" class="joystick-canvas"></canvas>
                  <div class="crosshair"></div>
                  <svg class="joystick-line-svg">
                    <line id="line-left" x1="50%" y1="50%" x2="50%" y2="50%" stroke="#ccc" stroke-width="2"/>
                  </svg>
                  <div class="joystick-handle" id="handle-left"></div>
                </div>
                <div class="circularity-info" id="circularity-info-left">{{ circularityErrorTextLeft }}</div>
                <div class="cali-btn-group">
                  <button class="btn btn-primary"
                          @click="startCalibration('left')"
                          :disabled="calibratingStick !== 'none'">
                    校准左摇杆
                  </button>
                </div>
              </div>
              <div class="progress-bars-container">
                <div class="progress-item">
                  <div class="progress-bar-vertical centered-progress"><div class="progress-fill" id="progress-x-left"></div></div>
                  <span class="progress-value" id="progress-x-left-value">X: 0</span>
                  <span class="progress-label">X轴</span>
                </div>
                <div class="progress-item">
                  <div class="progress-bar-vertical centered-progress"><div class="progress-fill" id="progress-y-left"></div></div>
                  <span class="progress-value" id="progress-y-left-value">Y: 0</span>
                  <span class="progress-label">Y轴</span>
                </div>
              </div>
            </div>

            <div class="divider"></div>

            <div class="joystick-display-group">
              <div class="joystick-graph" style="text-align:center;">
                <label>右摇杆</label>
                <div class="joystick-area" id="joystick-right">
                  <canvas v-if="isCircularityTestEnabled" id="canvas-right" class="joystick-canvas"></canvas>
                  <div class="crosshair"></div>
                  <svg class="joystick-line-svg">
                    <line id="line-right" x1="50%" y1="50%" x2="50%" y2="50%" stroke="#ccc" stroke-width="2"/>
                  </svg>
                  <div class="joystick-handle" id="handle-right"></div>
                </div>
                <div class="circularity-info" id="circularity-info-right">{{ circularityErrorTextRight }}</div>
                <div class="cali-btn-group">
                   <button class="btn btn-primary"
                          @click="startCalibration('right')"
                          :disabled="calibratingStick !== 'none'">
                    校准右摇杆
                  </button>
                </div>
              </div>
              <div class="progress-bars-container">
                <div class="progress-item">
                  <div class="progress-bar-vertical centered-progress"><div class="progress-fill" id="progress-x-right"></div></div>
                  <span class="progress-value" id="progress-x-right-value">X: 0</span>
                  <span class="progress-label">X轴</span>
                </div>
                <div class="progress-item">
                  <div class="progress-bar-vertical centered-progress"><div class="progress-fill" id="progress-y-right"></div></div>
                  <span class="progress-value" id="progress-y-right-value">Y: 0</span>
                  <span class="progress-label">Y轴</span>
                </div>
              </div>
            </div>
          </div>

          <div class="divider"></div>

          <!-- 右侧：校准模式 -->
          <div class="calibration-controls">
            <div class="calibration-mode-selector">
              <label>校准模式:</label>
              <div class="btn-group">
                <button
                  :class="['btn-switch', { 'active': calibrationMode === 'circle' }]"
                  @click="setCalibrationMode('circle')"
                  :disabled="calibratingStick !== 'none'">
                  圆形
                </button>
                <button
                  :class="['btn-switch', { 'active': calibrationMode === 'square' }]"
                  @click="setCalibrationMode('square')"
                  :disabled="calibratingStick !== 'none'">
                  方形
                </button>
              </div>
            </div>
            <!-- 圆度测试开关 -->
            <div class="setting-item circularity-toggle">
              <label for="circularity-test">圆度测试:</label>
              <label class="switch">
                <input type="checkbox" id="circularity-test" v-model="isCircularityTestEnabled">
                <span class="slider round"></span>
              </label>
            </div>
          </div>
        </div>

        <!-- 下方：引导信息 -->
        <div class="calibration-guide">
          <p>{{ calibrationHint }}</p>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-danger" @click="resetToDefault()" v-if="calibratingStick === 'none'">恢复默认</button>

        <div>
            <button class="btn btn-outline" @click="cancelCalibration()" v-if="calibratingStick !== 'none'">取消校准</button>
            <button class="btn btn-outline" @click="closeCaliModal()" v-else>关闭</button>
            
            <button class="btn btn-primary" @click="nextStep()" v-if="currentStep === 'CenterCheck' || currentStep === 'RangeDetection'">
              {{ currentStep === 'RangeDetection' ? '完成' : '下一步' }}
            </button>

            <button class="btn btn-primary" @click="saveCalibration()" :disabled="currentStep !== 'Complete'">
              保存
            </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { state } from "@/ts/global_states.ts";
import {
  closeCaliModal,
  calibratingStick,
  currentStep,
  calibrationHint,
  startCalibration,
  nextStep,
  cancelCalibration,
  saveCalibration,
  resetToDefault,
  calibrationMode,
  setCalibrationMode,
  isCircularityTestEnabled,
  circularityErrorTextLeft,
  circularityErrorTextRight
} from "@/ts/JoystickCaliModal.ts";

// 此组件现在只负责流程控制，不负责UI的实时更新
// 摇杆位置和进度条的更新由 ControllerGraph.ts 全局处理
</script>

<style scoped>
.joystick-area {
  position: relative; /* 确保 SVG 可以正确定位 */
}

.joystick-line-svg {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none; /* 确保 SVG 不会干扰鼠标事件 */
  z-index: 1; /* 确保线在十字线之上，但在摇杆之下 */
}

.joystick-handle {
  z-index: 2; /* 确保摇杆在最上层 */
  transition: none !important; /* 移除任何可能的动画延迟 */
}

.joystick-canvas {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0; /* 确保 Canvas 在背景之上，但在十字线之下 */
}

.joystick-area .crosshair {
  z-index: 1; /* 确保十字线在 Canvas 之上 */
}

/* 新增样式，用于控制中心对称的进度条 */
.centered-progress .progress-fill {
  bottom: auto; /* 移除固定的 bottom: 0，交由JS控制 */
  transition: none; /* 移除动画，确保与摇杆位置实时同步 */
}

.modal-body {
  /* 让 modal-body 垂直排列它的子元素 (joysticks-container 和 calibration-guide) */
  flex-direction: column;
  align-items: stretch;
  gap: 10px;
}

.modal-main-content {
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: flex-start;
  gap: 15px;
}

.joysticks-container {
  display: flex;
  flex-direction: row;
  gap: 15px;
  justify-content: center;
  align-items: flex-start;
}

.calibration-controls {
  display: flex;
  flex-direction: column;
  justify-content: flex-start; /* 置顶 */
  align-items: flex-start; /* 改为左对齐 */
  flex-shrink: 0;
  /* padding: 0 20px; */
}

.divider {
  width: 1px;
  background-color: #e0e4eb;
  align-self: stretch;
}

.calibration-guide {
  text-align: center;
  /* margin-top: 1rem; */ /* 移除外边距，使用父元素的 gap 控制 */
  padding: 8px;
  border-radius: 6px;
  font-size: 13px;
  background: #f0f7ff;
  color: #4c8bf5;
  min-height: auto; /* 移除最小高度限制，让其自适应内容 */
  flex-shrink: 0; /* 防止被 flex item 拉伸 */
}

.calibration-guide p {
  margin: 0; /* 移除 p 标签的默认 margin */
}
.modal-footer {
  justify-content: space-between;
  align-items: center;
}
.modal-footer > div {
    display: flex;
    gap: 0.5rem;
}

/* 新增样式 */
.calibration-mode-selector {
  display: flex;
  flex-direction: row; /* 水平排列 */
  justify-content: space-between; /* 改为两端对齐 */
  align-items: center;
  width: 100%; /* 撑满容器宽度 */
  font-size: 13px;
  padding: 0 10px; /* 新增：与下方对齐 */
  box-sizing: border-box; /* 新增：确保 padding 不会影响宽度计算 */
}

.circularity-info {
  text-align: center;
  font-size: 12px;
  color: #666;
  margin-top: 5px;
  min-height: 18px; /* 避免内容出现时布局跳动 */
}

.setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
}

.circularity-toggle {
  margin-top: 20px; /* 与上方元素隔开一些距离 */
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  justify-content: space-between; /* 改为两端对齐 */
  width: 100%; /* 撑满容器宽度 */
  padding: 0 10px;
}

.switch {
    position: relative;
    display: inline-block;
    width: 38px;
    height: 22px;
}

.switch input {
    opacity: 0;
    width: 0;
    height: 0;
}

.slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #ccc;
    transition: .4s;
}

.slider:before {
    position: absolute;
    content: "";
    height: 16px;
    width: 16px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: .4s;
}

input:checked + .slider {
    background-color: var(--button-base-bg);
}

input:checked + .slider:before {
    transform: translateX(16px);
}

.slider.round {
    border-radius: 22px;
}

.slider.round:before {
    border-radius: 50%;
}
</style>
