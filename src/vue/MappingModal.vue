<template>
  <transition name="modal-fade">
    <div class="modal-overlay" id="mapping-modal" :class="{ active: state.showMappingModal }"
      @click.self="closeButtonMapModal()">
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
                <label>选择输入源: </label>
                <select class="form-control" id="controller-button" v-model="state.selectedButton">
                  <option disabled value="">-- 请选择 --</option>
                  <optgroup label="按键">
                    <option v-for="btn in state.buttonsText" :key="btn.value" :value="btn.value">
                      {{ $t(`buttons.${btn.value}`) }}
                    </option>
                  </optgroup>
                  <optgroup label="摇杆">
                    <option v-for="stick in state.sticksText" :key="stick.value" :value="stick.value">
                      {{ $t(`sticks.${stick.value}`) }}
                    </option>
                  </optgroup>
                </select>
              </div>

              <div class="form-group check-mode">
                <label for="btn-check-mode">检测模式: </label>
                <div class="btn-group">
                  <button @click="state.checkMode = checkModeOptions[0]"
                    :class="['btn-switch', { 'active': state.checkMode === checkModeOptions[0] }]">
                    单击
                  </button>
                  <button @click="state.checkMode = checkModeOptions[1]"
                    :class="['btn-switch', { 'active': state.checkMode === checkModeOptions[1] }]">
                    双击
                  </button>
                  <button @click="state.checkMode = checkModeOptions[2]"
                    :class="['btn-switch', { 'active': state.checkMode === checkModeOptions[2] }]">
                    长按
                  </button>
                </div>
              </div>

              <div class="form-group check-mode check-mode-div"
                v-if="textInclude(state.checkMode, checkModeOptions.filter(e => e != 'single'), false)">
                <label v-if="state.checkMode === 'double'">└ 双击最大间隔</label>
                <label v-if="state.checkMode === 'long'">└ 长按最小间隔</label>
                <div class="check-mode-control">
                  <input type="number" class="form-control" v-model.number="state.checkModeParam">
                  <span>ms</span>
                </div>
              </div>

              <div class="form-group key-detector">
                <label>映射输出动作: </label>
                <div class="detector-area" :class="{ active: state.keyListenerActive }" id="key-detector-area"
                  @click="detectKey()">
                  {{ state.keyDetectorText }}
                </div>
                <div class="detector-hint">点击上方区域, 然后按下按键或滚动滚轮</div>
                <div class="key-display" id="key-display">{{ state.keyDisplayText }}</div>
              </div>
              <div id="modal-error" class="status-message error" v-show="state.modalErrorVisible">
                {{ state.modalErrorMessage }}
              </div>
            </div>

            <!-- Divider -->
            <div class="divider"></div>

            <!-- Right Panel: Conditional Inputs -->
            <div class="modal-middle-panel">
              <!-- Trigger State Settings -->
              <div>
                <div class="form-group continually-trigger">
                  <label>映射持续触发: </label>
                  <label class="switch">
                    <input type="checkbox" v-model="state.triggerState.continually_trigger">
                    <span class="slider round"></span>
                  </label>
                </div>
                <div class="form-group" v-if="state.triggerState.continually_trigger">
                  <label for="initial-interval">初始触发间隔 (ms)</label>
                  <input type="number" id="initial-interval" class="form-control"
                    v-model.number="state.triggerState.initial_interval">
                </div>
                <div class="form-group" v-if="state.triggerState.continually_trigger">
                  <label for="min-interval">最小触发间隔 (ms)</label>
                  <input type="number" id="min-interval" class="form-control"
                    v-model.number="state.triggerState.min_interval">
                </div>
                <div class="form-group" v-if="state.triggerState.continually_trigger">
                  <label for="acceleration">加速因子</label>
                  <div class="slider-container">
                    <input type="range" id="acceleration" min="0.1" max="2" step="0.1"
                      v-model.number="state.triggerState.acceleration">
                    <span>{{ state.triggerState.acceleration }}</span>
                  </div>
                </div>
              </div>

              <!-- Divider -->
              <div class="divider horizon"></div>

              <!-- Amount for Mouse Wheel -->
              <div v-if="textInclude(state.rawKeyDisplayText, mousewheel, false)">
                <div class="form-group">
                  <label for="mousewheel-amount">滚轮滚动量</label>
                  <div class="slider-container">
                    <input type="range" id="mousewheel-amount" min="1" max="20" step="1"
                      v-model.number="state.mapping_amount">
                    <span>{{ state.mapping_amount }}</span>
                  </div>
                </div>
              </div>

              <div v-if="textInclude(state.selectedButton, trigger_text, true)">
                <div class="form-group">
                  <label for="mousewheel-amount">扳机触发阈值</label>
                  <div class="slider-container">
                    <input type="range" id="trigger_threshold" min="0.01" max="1" step="0.01"
                      v-model.number="state.triggerTheshold">
                    <span>{{ (state.triggerTheshold * 100).toFixed(0) }}%</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- <div class="modal-right-panel">
            </div> -->

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
import { state } from "@/ts/global_states.ts";
import { closeButtonMapModal, detectKey, mappingsConfirm } from "@/ts/MappingModal.ts";

const checkModeOptions = ["single", "double", "long"];
const trigger_text = ["lt", "rt"];
const mousewheel = ["mousewheel"];

function textInclude(text: string, pattens: string[], strictMatch: boolean = false) {
  if (strictMatch) {
    return pattens.some(e => text.toLowerCase() === e.toLowerCase());
  } else {
    return pattens.some(e => text.toLowerCase().includes(e));
  }
}
</script>

<style scoped>
.modal {
  width: 70%;
  max-width: 900px;
}

.modal-left-panel {
  flex: 4;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.modal-middle-panel {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 0px;
}

.modal-middle-panel .form-group {
  margin-bottom: 10px;
}

.modal-right-panel {
  flex: 3;
  display: flex;
  flex-direction: column;
  gap: 0px;
}

.modal-content-wrapper {
  display: flex;
  gap: 15px;
}

.mapping-type-selector {
  display: flex;
  gap: 20px;
  margin-bottom: 20px;
  border-bottom: 1px solid #ccc;
  padding-bottom: 15px;
}

.mapping-type-selector label {
  display: flex;
  align-items: center;
  gap: 5px;
}

.continually-trigger {
  display: flex;
  justify-content: space-between;
  margin-top: 6px;
}

.check-mode {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
}

.check-mode label {
  margin-bottom: 0;
  flex-shrink: 0;
}

.status-message {
  margin-top: 0px;
}

.check-mode-control {
  width: 37%;
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 14px;
  color: #555;
}

.check-mode-div {
  margin-top: 5px;
  margin-bottom: 0px;
}

.divider.horizon {
  width: 100%;
  height: 1px;
  background-color: #e0e4eb;
  margin-top: 10px;
  margin-bottom: 6px;
}
</style>
