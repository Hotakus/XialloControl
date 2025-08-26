<template>
  <div class="joystick-cali-overlay" :class="{active: state.showCaliModal}" id="joystick-cali-modal" @click.self="closeCaliModal()">
    <div class="joystick-cali-modal">
      <div class="modal-header">
        摇杆校准
        <button class="modal-close" @click="closeCaliModal()">&times;</button>
      </div>

      <div class="modal-body">
        <div class="joysticks-container">
          <div class="joystick-display-group">
            <div style="text-align:center;">
              <label>左摇杆</label>
              <div class="joystick-area" id="joystick-left">
                <div class="crosshair"></div>
                <div class="joystick-handle" id="handle-left"></div>
              </div>
              <div class="cali-btn-group">
                <button class="btn btn-primary"
                        @click="startCalibration('left')"
                        :disabled="calibratingStick !== 'none'">
                  校准左摇杆
                </button>
              </div>
            </div>
            <!-- 进度条的HTML结构保持不变，由外部逻辑更新 -->
            <div class="progress-bars-container">
              <div class="progress-item">
                <div class="progress-bar-vertical"><div class="progress-fill" id="progress-x-left"></div></div>
                <span class="progress-value" id="progress-x-left-value">X: 0</span>
                <span class="progress-label">X轴</span>
              </div>
              <div class="progress-item">
                <div class="progress-bar-vertical"><div class="progress-fill" id="progress-y-left"></div></div>
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
                <div class="crosshair"></div>
                <div class="joystick-handle" id="handle-right"></div>
              </div>
              <div class="cali-btn-group">
                 <button class="btn btn-primary"
                        @click="startCalibration('right')"
                        :disabled="calibratingStick !== 'none'">
                  校准右摇杆
                </button>
              </div>
            </div>
            <!-- 进度条的HTML结构保持不变，由外部逻辑更新 -->
            <div class="progress-bars-container">
              <div class="progress-item">
                <div class="progress-bar-vertical"><div class="progress-fill" id="progress-x-right"></div></div>
                <span class="progress-value" id="progress-x-right-value">X: 0</span>
                <span class="progress-label">X轴</span>
              </div>
              <div class="progress-item">
                <div class="progress-bar-vertical"><div class="progress-fill" id="progress-y-right"></div></div>
                <span class="progress-value" id="progress-y-right-value">Y: 0</span>
                <span class="progress-label">Y轴</span>
              </div>
            </div>
          </div>
        </div>
        <!-- 引导信息 -->
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
  resetToDefault
} from "@/ts/JoystickCaliModal.ts";

// 此组件现在只负责流程控制，不负责UI的实时更新
// 摇杆位置和进度条的更新由 ControllerGraph.ts 全局处理
</script>

<style scoped>
.modal-body {
  /* 让 modal-body 垂直排列它的子元素 (joysticks-container 和 calibration-guide) */
  flex-direction: column;
  align-items: stretch; /* 覆盖全局的 align-items: center */
  gap: 10px; /* 减小垂直间距 */
}

.joysticks-container {
  /* 这个容器负责水平排列两个摇杆 */
  display: flex;
  flex-direction: row;
  gap: 15px;
  justify-content: center;
  align-items: center;
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
</style>
