<template>
  <div class="joystick-cali-overlay" :class="{active: state.showCaliModal}" id="joystick-cali-modal">
    <div class="joystick-cali-modal">
      <div class="modal-header">
        摇杆校准
        <button class="modal-close" @click="closeCaliModal()">&times;</button>
      </div>

      <div class="modal-body">
        <!-- 引导信息 -->
        <div class="calibration-guide">
          <p>{{ calibrationHint }}</p>
        </div>

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
.calibration-guide {
  text-align: center;
  margin-bottom: 1rem;
  font-size: 1.1rem;
  min-height: 40px; /* 防止文字变化时布局跳动 */
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
