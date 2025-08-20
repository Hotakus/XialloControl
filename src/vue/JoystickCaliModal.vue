<template>
  <div class="joystick-cali-overlay" :class="{active: state.showCaliModal}" id="joystick-cali-modal">
    <div class="joystick-cali-modal">
      <div class="modal-header">
        摇杆校准
        <button class="modal-close" id="close-joystick-cali-modal">&times;</button>
      </div>
      <div class="modal-body">
        <div class="joystick-display-group">
          <div style="text-align:center;">
            <label>左摇杆</label>
            <div class="joystick-area" id="joystick-left">
              <div class="crosshair"></div>
              <div class="joystick-handle" id="handle-left"></div>
            </div>
            <div class="cali-btn-group">
              <button class="btn btn-primary" id="calibrate-left-stick">校准左摇杆</button>
<!--              <div>推荐死区: <span id="deadzone-cali-left">0</span>%</div>-->
            </div>
          </div>
          <div class="progress-bars-container">
            <div class="progress-item">
              <div class="progress-bar-vertical">
                <div class="progress-fill" id="progress-x-left"></div>
              </div>
              <span class="progress-value" id="progress-x-left-value">X: 0</span>
              <span class="progress-label">X轴</span>
            </div>
            <div class="progress-item">
              <div class="progress-bar-vertical">
                <div class="progress-fill" id="progress-y-left"></div>
              </div>
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
              <button class="btn btn-primary" id="calibrate-right-stick">校准右摇杆</button>
<!--              <div>推荐死区: <span id="deadzone-cali-right">0</span>%</div>-->
            </div>
          </div>
          <div class="progress-bars-container">
            <div class="progress-item">
              <div class="progress-bar-vertical">
                <div class="progress-fill" id="progress-x-right"></div>
              </div>
              <span class="progress-value" id="progress-x-right-value">X: 0</span>
              <span class="progress-label">X轴</span>
            </div>
            <div class="progress-item">
              <div class="progress-bar-vertical">
                <div class="progress-fill" id="progress-y-right"></div>
              </div>
              <span class="progress-value" id="progress-y-right-value">Y: 0</span>
              <span class="progress-label">Y轴</span>
            </div>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-outline" id="cancel-joystick-cali-btn" @click="closeCaliModal()">取消</button>
        <button class="btn btn-primary" id="save-joystick-cali-btn">保存</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 可以写组件逻辑
import {state} from "@/ts/global_states.ts";
import {closeCaliModal} from "@/ts/JoystickCaliModal.ts";
</script>

<style scoped>
/* 如果只作用于这个组件，可以写 scoped 样式 */
</style>
