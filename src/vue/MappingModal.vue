<template>
  <transition name="modal-fade">
    <div
        class="modal-overlay"
        id="mapping-modal" :class="{active: state.showMappingModal}" @click.self="closeButtonMapModal()">
      <div class="modal">
        <div class="modal-header">
          <span id="modal-title">
            {{ state.modalTitle }}
          </span>
          <button class="modal-close" id="close-modal" @click="closeButtonMapModal()">&times;</button>
        </div>
        <div class="modal-body">
          <div class="modal-content-wrapper">
            <!-- Left Panel -->
            <div class="modal-left-panel">
              <div class="form-group">
                <label><i class="fas fa-gamepad"></i> 选择手柄按键</label>
                <select class="form-control" id="controller-button" v-model="state.selectedButton">
                  <option disabled value="">-- 请选择按键 --</option>
                  <option v-for="btnText in state.buttonsText"
                          :key="btnText.value"
                          :value="btnText.value">
                    {{ btnText.text }}
                  </option>
                </select>
              </div>

              <div class="form-group key-detector">
                <label><i class="fas fa-keyboard"></i> 映射到：</label>
                <div class="detector-area" :class="{active: state.keyListenerActive}" id="key-detector-area"
                     @click="detectKey()">
                  {{ state.keyDetectorText }}
                </div>
                <div class="detector-hint">支持单键或组合键（如 Ctrl+C、Alt+F4）</div>
                <div class="key-display" id="key-display">{{ state.keyDisplayText }}</div>
              </div>
            </div>

            <!-- Divider -->
            <div class="divider"></div>

            <!-- Right Panel -->
            <div class="modal-right-panel">
              <div class="form-group">
                <label for="initial-interval">初始触发间隔 (ms)</label>
                <input type="number" id="initial-interval" class="form-control" v-model.number="state.triggerState.initial_interval">
              </div>
              <div class="form-group">
                <label for="min-interval">最小触发间隔 (ms)</label>
                <input type="number" id="min-interval" class="form-control" v-model.number="state.triggerState.min_interval">
              </div>
              <div class="form-group">
                <label for="acceleration">加速因子</label>
                <div class="slider-container">
                  <input type="range" id="acceleration" min="0.1" max="2" step="0.1" v-model.number="state.triggerState.acceleration">
                  <span>{{ state.triggerState.acceleration }}</span>
                </div>
              </div>
            </div>
          </div>
          <div id="modal-error" class="status-message error" style="margin-top: 15px;" v-show="state.modalErrorVisible">
            {{ state.modalErrorMessage }}
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-outline" id="cancel-btn" @click="closeButtonMapModal()">取消</button>
          <button class="btn btn-primary" id="confirm-btn" @click="mappingsConfirm()">确认</button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
// 可以写组件逻辑
import {state} from "@/ts/global_states.ts";
import {closeButtonMapModal, detectKey, mappingsConfirm} from "@/ts/MappingModal.ts";
</script>

<style scoped>
/* 如果只作用于这个组件，可以写 scoped 样式 */
</style>
