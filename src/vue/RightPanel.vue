<template>
  <div class="right-panel">
    <div class="card preset-card">
      <label for="preset">📂 预设方案:</label>
      <div class="preset-header">
        <select id="preset" class="preset-select">
          <option>默认</option>
        </select>
        <div class="preset-controls">
          <button id="save-preset" title="保存方案" class="icon-button">
            <svg t="1753597230725" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="20689" width="200" height="200">
              <path
                  d="M72.902194 16.295914h800.074322a23.563011 23.563011 0 0 1 18.773333 8.874667l117.561807 142.974623c3.721634 4.525419 5.527398 9.568344 5.527398 15.437076v764.080172c0 40.145204-32.745978 72.902194-72.902194 72.902193H72.913204C32.745978 1020.564645 0 987.807656 0 947.662452V89.198108C0 49.041892 32.745978 16.295914 72.902194 16.295914z m37.854967 516.217118a4.866753 4.866753 0 0 0-4.855742 4.877764v363.795269c0 2.664602 2.180129 4.844731 4.855742 4.844731h795.449807a4.844731 4.844731 0 0 0 4.855742-4.844731v-363.795269a4.877763 4.877763 0 0 0-4.855742-4.877764H110.757161zM228.616258 124.64172a4.877763 4.877763 0 0 0-4.855742 4.866753V354.105806c0 2.686624 2.180129 4.866753 4.855742 4.866753h557.606538a4.866753 4.866753 0 0 0 4.855742-4.866753V129.508473a4.877763 4.877763 0 0 0-4.855742-4.866753H228.616258z"
                  fill="#ffffff" p-id="20690"></path>
            </svg>
          </button>
          <button id="import-preset" title="导入方案" class="icon-button">
            <svg t="1753597486869" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="22279" width="64" height="64">
              <path
                  d="M601.152 708.288 400 568 344 568l0 112-224 0 0 112 224 0 0 112L400 904l201.152-140.288C624 749.504 624 722.496 601.152 708.288L601.152 708.288zM891.264 331.2 638.656 76.864C630.528 68.608 619.456 64 607.936 64L232 64C196.032 64 176 83.712 176 120L176 512 288 512 288 176l280 0 0 168c0 24.192 32 56 56 56l168 0 0.768 448L624 848 624 960l224 0c35.968 0 56-19.712 56-56L904 362.176C904 350.528 899.392 339.392 891.264 331.2L891.264 331.2z"
                  p-id="22280" fill="#ffffff"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="tabs" role="tablist">
        <div class="tab" :class="{active: state.activeTab === 'buttonMapTab'}" role="tab" aria-selected="true" data-tab="buttonMapTab" @click="switchTab('buttonMapTab')">按键映射</div>
        <div class="tab" :class="{active: state.activeTab ==='stickMapTab'}" role="tab" aria-selected="false" data-tab="stickMapTab" @click="switchTab('stickMapTab')">摇杆映射</div>
        <div class="tab" :class="{active: state.activeTab ==='settingTab'}" role="tab" aria-selected="false" data-tab="settingTab" @click="switchTab('settingTab')">设置</div>
      </div>

      <div id="buttonMapTab" class="tab-content" :class="{active: state.activeTab === 'buttonMapTab'}" role="tabpanel">
        <div class="tab-content-container">
          <div class="button-map-header">
            <div class="button-map-title"><i class="fas fa-keyboard"></i> 按键映射</div>
            <div class="button-map-controls">
              <button id="add-button-map" title="添加映射" class="icon-button" @click="addButtonMap()">
                <svg t="1753626247148" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="2595" width="200" height="200">
                  <path d="M508.9 926.4c-36.3-1.6-64.6-32.2-64.6-68.6V166.1c0-36.3 28.3-66.9 64.6-68.6 38.8-1.7 70.7 29.2 70.7 67.6v693.7c0 38.4-31.9 69.4-70.7 67.6z" fill="#ffffff" p-id="2596"></path>
                  <path d="M858.9 579.6H165.2c-37.4 0-67.6-30.3-67.6-67.6 0-37.4 30.3-67.6 67.6-67.6h693.7c37.4 0 67.6 30.3 67.6 67.6 0 37.4-30.3 67.6-67.6 67.6z" fill="#ffffff" p-id="2597"></path>
                </svg>
              </button>
            </div>
          </div>
          <div class="button-map" id="button-mapping-list">
            <div class="empty-state">
              <p>尚未添加任何按键映射 <br>
                点击右上角的按钮添加映射
              </p>
            </div>
          </div>
        </div>
      </div>

      <div id="stickMapTab" class="tab-content" :class="{active: state.activeTab ==='stickMapTab'}" role="tabpanel">
        <div class="settings-container">
          <div class="setting-group">
            <h3>摇杆设置</h3>
            <div class="setting-item">
              <label for="deadzone">右摇杆死区:</label>
              <div class="slider-container">
                <input type="range" id="deadzone" min="0" max="30" value="10">
                <span id="deadzone-value">10%</span>
              </div>
            </div>
            <div class="setting-item">
              <label for="deadzone">左摇杆死区:</label>
              <div class="slider-container">
                <input type="range" id="deadzone-left" min="0" max="30" value="10">
                <span id="deadzone-left-value">10%</span>
              </div>
            </div>
            <div class="stick-buttons-row">
              <button id="open-joystick-cali-modal" class="icon-button">
                <svg t="1754895524804" viewBox="0 0 24 24" width="20" height="20">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13.5v6l5.25 3.15.75-1.23-4.5-2.67v-5.25z" fill="currentColor"/>
                </svg>
                校准
              </button>
            </div>
          </div>
          <div class="setting-group">
            <h3>摇杆行为</h3>
            <div class="setting-item">
              <label>右摇杆旋转:</label>
            </div>
          </div>

        </div>
      </div>

      <div id="settingTab" class="tab-content" :class="{active: state.activeTab ==='settingTab'}" role="tabpanel">
        <div class="settings-container">
          <div class="setting-group">
            <h3><i class="fas fa-cog"></i>软件设置</h3>

            <div class="setting-item">
              <label for="auto-start">开机自启动:</label>
              <label class="switch">
                <input type="checkbox" id="auto-start" v-model="state.autoStart" @change="updateSettings()">
                <span class="slider round"></span>
              </label>
            </div>

            <div class="setting-item">
              <label for="minimize-to-tray">最小化到托盘:</label>
              <label class="switch">
                <input type="checkbox" id="minimize-to-tray" v-model="state.minimizeToTray" @change="updateSettings()">
                <span class="slider round"></span>
              </label>
            </div>

            <div class="setting-item">
              <label for="polling-frequency">轮询频率:</label>
              <div class="polling-container">
                <input type="number" id="polling-frequency" min="1" max="8000" value="125" v-model="state.pollingFrequency" @change="updateSettings()">
                <span>Hz</span>
              </div>
            </div>

            <div class="setting-item">
              <label for="theme">界面主题:</label>
              <select id="theme" v-model="state.theme" @change="changeTheme()">
                <option value="light">浅色模式</option>
                <option value="dark">深色模式</option>
                <option value="system">跟随系统</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>


  <transition name="modal-fade">
    <div
        class="modal-overlay"
        id="mapping-modal" :class="{active: state.showMappingModal}">
      <div class="modal">
        <div class="modal-header">
          <span id="modal-title">
            {{ state.modalTitle }}
          </span>
          <button class="modal-close" id="close-modal" @click="closeButtonMapModal()">&times;</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label><i class="fas fa-gamepad"></i> 选择手柄按键</label>
            <select class="form-control" id="controller-button" v-model="state.selectedButton">
              <option value="">-- 请选择按键 --</option>
              <option value="A">A 按钮</option>
              <option value="B">B 按钮</option>
              <option value="X">X 按钮</option>
              <option value="Y">Y 按钮</option>
              <option value="LB">左肩键 (LB)</option>
              <option value="RB">右肩键 (RB)</option>
            </select>
          </div>

          <div class="form-group key-detector">
            <label><i class="fas fa-keyboard"></i> 映射到：</label>
            <div class="detector-area" :class="{active: state.keyListenerActive}" id="key-detector-area" @click="detectKey()">
              {{ state.keyDetectorText }}
            </div>
            <div class="detector-hint">支持单键或组合键（如 Ctrl+C、Alt+F4）</div>
            <div class="key-display" id="key-display">{{ state.keyDisplayText }}</div>
          </div>
          <div id="modal-error" class="status-message error" style="margin-top: 15px;" v-show="state.modalErrorVisible">
            {{ state.modalErrorMessage }}
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-outline" id="cancel-btn" @click="closeButtonMapModal()">取消</button>
          <button class="btn btn-primary" id="confirm-btn" @click="saveButtonMap()">确认</button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import {
  updateSettings,
  switchTab,
  changeTheme,
  saveButtonMap,
  openButtonMapModal,
  closeButtonMapModal,
  detectKey, addButtonMap
} from "@/ts/RightPanel.ts";
import {state} from "@/ts/global_states.ts";
</script>

<style scoped>

</style>
