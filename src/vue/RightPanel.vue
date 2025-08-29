<template>
  <div class="right-panel">
    <div class="card preset-card">
      <label for="preset">预设方案:</label>
      <div class="preset-header">
        <!-- 正常模式：显示下拉框 -->
        <select v-if="!state.isCreatingNewPreset" id="preset" class="preset-select" v-model="state.previousPreset"
          @change="switchPreset()">
          <option disabled value="">-- 请选择预设方案 --</option>
          <option v-for="preset in state.presets" :key="preset" :value="preset">
            {{ preset }}
          </option>
        </select>

        <!-- 新建模式：显示输入框 -->
        <div v-else class="preset-input-container">
          <input type="text" v-model="state.newPresetName" placeholder="输入方案名称" class="preset-input"
            @keyup.enter="confirmNewPreset()" @keyup.escape="cancelNewPreset()" ref="presetInput" />
          <button class="icon-button confirm-btn" @click="confirmNewPreset()" title="确认">
            <svg t="1756401635650" viewBox="0 0 1024 1024" width="16" height="16">
              <path
                d="M912 190h-69.9c-9.8 0-19.1 4.5-25.1 12.2L404.7 724.5 207 474c-6-7.7-15.3-12.2-25.1-12.2H112c-6.7 0-10.4 7.7-6.3 12.9l273.9 347c12.8 16.2 37.4 16.2 50.3 0l488.4-618.9c4.1-5.1.4-12.8-6.3-12.8z"
                fill="#ffffff" />
            </svg>
          </button>
          <button class="icon-button cancel-btn" @click="cancelNewPreset()" title="取消">
            <svg t="1756401635650" viewBox="0 0 1024 1024" width="16" height="16">
              <path
                d="M563.8 512l262.5-312.9c4.4-5.2.7-13.1-6.1-13.1h-79.8c-4.7 0-9.2 2.1-12.3 5.7L511.6 449.8 295.1 191.7c-3-3.6-7.5-5.7-12.3-5.7H203c-6.8 0-10.5 7.9-6.1 13.1L459.4 512 196.9 824.9A7.95 7.95 0 0 0 203 838h79.8c4.7 0 9.2-2.1 12.3-5.7l216.5-258.1 216.5 258.1c3 3.6 7.5 5.7 12.3 5.7h79.8c6.8 0 10.5-7.9 6.1-13.1L563.8 512z"
                fill="#ffffff" />
            </svg>
          </button>
        </div>

        <div class="preset-controls">
          <button v-if="!state.isCreatingNewPreset" id="save-preset" title="新建方案" class="icon-button"
            @click="newPreset()">
            <svg t="1756401635650" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg"
              p-id="4604" width="200" height="200">
              <path
                d="M914.181742 251.621027L672.174208 10.295205A34.085568 34.085568 0 0 0 645.587465 0.069535H134.303944a34.085568 34.085568 0 0 0-34.085569 34.085568v954.395906a34.085568 34.085568 0 0 0 34.085569 34.085568h755.336188a34.085568 34.085568 0 0 0 34.085569-34.085568V272.754079a34.085568 34.085568 0 0 0-9.543959-21.133052z m-92.712746 3.408557H666.720517V100.962816zM168.389512 954.465441V68.240671h430.159869v220.874481a34.085568 34.085568 0 0 0 34.085568 34.085568h222.919615V954.465441z"
                fill="#ffffff" p-id="4605"></path>
              <path
                d="M713.758601 545.438624H548.10274V379.782763a34.085568 34.085568 0 0 0-68.171136 0V545.438624H304.731784a34.085568 34.085568 0 0 0-34.085568 34.085568 33.403857 33.403857 0 0 0 4.771979 16.361073 34.085568 34.085568 0 0 0 31.358723 21.133052h170.427841v170.42784a34.085568 34.085568 0 1 0 68.171136 0V618.38174h170.42784a34.085568 34.085568 0 0 0 34.085568-34.085568 33.403857 33.403857 0 0 0-4.771979-16.361073A34.085568 34.085568 0 0 0 713.758601 545.438624z"
                fill="#ffffff" p-id="4606"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="tabs" role="tablist">
        <div class="tab" :class="{ active: state.activeTab === 'buttonMapTab' }" role="tab" aria-selected="true"
          data-tab="buttonMapTab" @click="switchTab('buttonMapTab')">按键映射</div>
        <div class="tab" :class="{ active: state.activeTab === 'stickMapTab' }" role="tab" aria-selected="false"
          data-tab="stickMapTab" @click="switchTab('stickMapTab')">压力设置</div>
        <div class="tab" :class="{ active: state.activeTab === 'settingTab' }" role="tab" aria-selected="false"
          data-tab="settingTab" @click="switchTab('settingTab')">设置</div>
      </div>

      <div id="buttonMapTab" class="tab-content" :class="{ active: state.activeTab === 'buttonMapTab' }"
        role="tabpanel">
        <div class="tab-content-container">
          <div class="button-map-header">
            <div class="button-map-title"><i class="fas fa-keyboard"></i> 按键映射</div>
            <div class="button-map-controls">
              <button id="add-button-map" title="添加映射" class="icon-button" @click="addButtonMap()">
                <svg t="1753626247148" class="icon" viewBox="0 0 1024 1024" version="1.1"
                  xmlns="http://www.w3.org/2000/svg" p-id="2595" width="200" height="200">
                  <path
                    d="M508.9 926.4c-36.3-1.6-64.6-32.2-64.6-68.6V166.1c0-36.3 28.3-66.9 64.6-68.6 38.8-1.7 70.7 29.2 70.7 67.6v693.7c0 38.4-31.9 69.4-70.7 67.6z"
                    fill="#ffffff" p-id="2596"></path>
                  <path
                    d="M858.9 579.6H165.2c-37.4 0-67.6-30.3-67.6-67.6 0-37.4 30.3-67.6 67.6-67.6h693.7c37.4 0 67.6 30.3 67.6 67.6 0 37.4-30.3 67.6-67.6 67.6z"
                    fill="#ffffff" p-id="2597"></path>
                </svg>
              </button>
            </div>
          </div>

          <div class="button-map">
            <div class="empty-state" v-if="state.mappings.length === 0">
              <p>
                <svg t="1753627389377" class="icon" viewBox="0 0 1024 1024" version="1.1"
                  xmlns="http://www.w3.org/2000/svg" p-id="3769" width="32" height="32">
                  <path
                    d="M921.6 208.3c12.6 0 24.5 5 33.5 14s14 20.9 14 33.5v512.3c0 12.6-5 24.5-14 33.5s-20.9 14-33.5 14H102.4c-12.6 0-24.4-5-33.4-14s-14-20.9-14-33.5V255.9c0-12.6 5-24.5 14-33.5s20.9-14 33.4-14h819.2m0-55.1H102.4C46.1 153.3 0 199.4 0 255.9v512.3c0 56.4 46.1 102.5 102.4 102.5h819.1c56.4 0 102.4-46.1 102.4-102.5V255.9c0.1-56.5-45.9-102.6-102.3-102.6zM460.8 307h102.4v102.5H460.8V307z m0 153.7h102.4v102.5H460.8V460.7zM307.2 307h102.4v102.5H307.2V307z m0 153.7h102.4v102.5H307.2V460.7zM256 563.2H153.7V460.9H256v102.3z m0-153.6H153.7V307.2H256v102.4z m614.4 358.5H153.8V665.8h716.6v102.3zM716.8 563.2H614.5V460.9h102.3v102.3z m0-153.6H614.5V307.2h102.3v102.4z m153.6 153.6H768.1V460.9h102.3v102.3z m0-153.6H768.1V307.2h102.3v102.4z m0 0"
                    p-id="3770" fill="#3A7DE0"></path>
                </svg>
                <br>
                尚未添加任何按键映射 <br>
                点击右上角的
                <svg t="1753627455046" class="icon" viewBox="0 0 1024 1024" version="1.1"
                  xmlns="http://www.w3.org/2000/svg" p-id="4836" width="32" height="32">
                  <path
                    d="M508.9 926.4c-36.3-1.6-64.6-32.2-64.6-68.6V166.1c0-36.3 28.3-66.9 64.6-68.6 38.8-1.7 70.7 29.2 70.7 67.6v693.7c0 38.4-31.9 69.4-70.7 67.6z"
                    fill="#4C8BF5" p-id="4837"></path>
                  <path
                    d="M858.9 579.6H165.2c-37.4 0-67.6-30.3-67.6-67.6 0-37.4 30.3-67.6 67.6-67.6h693.7c37.4 0 67.6 30.3 67.6 67.6 0 37.4-30.3 67.6-67.6 67.6z"
                    fill="#4C8BF5" p-id="4838"></path>
                </svg>
                按钮添加映射
              </p>
            </div>
            <template v-else>
              <div class="button-map-item" v-for="mapping in state.mappings" :key="mapping.id"
                @dblclick="editButtonMap(mapping.id)">
                <div class="button-icon">{{ mapping.composed_button }}</div>
                <div class="key-text">映射到</div>
                <div class="key-value">{{ formatKeyDisplay(mapping.composed_shortcut_key) }}</div>
                <div class="item-actions">
                  <button class="item-action-btn edit" @click="editButtonMap(mapping.id)">
                    <svg t="1753769162786" class="icon" viewBox="0 0 1024 1024" version="1.1"
                      xmlns="http://www.w3.org/2000/svg" p-id="3801" width="200" height="200">
                      <path
                        d="M869.62198 290.936185c-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 501.688143c0 40.342824-32.8205 73.163323-73.163323 73.163323L252.963339 897.142777c-40.342824 0-73.163323-32.8205-73.163323-73.163323l0-606.206592c0-40.342824 32.8205-73.163323 73.163323-73.163323l407.621744 0c17.316387 0 31.355125-14.039761 31.355125-31.355125s-14.039761-31.355125-31.355125-31.355125L252.963339 81.899288c-74.92341 0-135.873574 60.950164-135.873574 135.873574l0 606.206592c0 74.92341 60.950164 135.873574 135.873574 135.873574l512.140193 0c74.92341 0 135.873574-60.950164 135.873574-135.873574L900.977106 322.292334C900.978129 304.975946 886.938368 290.936185 869.62198 290.936185z"
                        fill="#707070" p-id="3802"></path>
                      <path
                        d="M535.946388 467.382826c6.01704 5.496178 13.59053 8.205892 21.143553 8.205892 8.502651 0 16.97358-3.434216 23.159466-10.201339L898.602012 116.986411c11.682064-12.779048 10.783601-32.615838-1.995447-44.297902-12.784164-11.676947-32.615838-10.783601-44.303019 2.000564L533.950941 423.084924C522.269901 435.863972 523.167341 455.700763 535.946388 467.382826z"
                        fill="#707070" p-id="3803"></path>
                      <path
                        d="M355.315448 594.978876l0 30.589692c0 17.316387 14.039761 31.355125 31.355125 31.355125 17.316387 0 31.355125-14.039761-31.355125 31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125C369.355209 563.623751 355.315448 577.663512 355.315448 594.978876z"
                        fill="#707070" p-id="3804"></path>
                      <path
                        d="M631.396297 656.924717c17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 30.589692C600.041172 642.884956 614.07991 656.924717 631.396297 656.924717z"
                        fill="#707070" p-id="3805"></path>
                    </svg>
                  </button>
                  <button class="item-action-btn delete" @click="deleteButtonMap(mapping.id)">
                    <svg t="1753765954234" class="icon" viewBox="0 0 1024 1024" version="1.1"
                      xmlns="http://www.w3.org/2000/svg" p-id="2368" width="200" height="200">
                      <path
                        d="M840 288H688v-56c0-40-32-72-72-72h-208C368 160 336 192 336 232V288h-152c-12.8 0-24 11.2-24 24s11.2 24 24 24h656c12.8 0 24-11.2 24-24s-11.2-24-24-24zM384 288v-56c0-12.8 11.2-24 24-24h208c12.8 0 24 11.2 24 24V288H384zM758.4 384c-12.8 0-24 11.2-24 24v363.2c0 24-19.2 44.8-44.8 44.8H332.8c-24 0-44.8-19.2-44.8-44.8V408c0-12.8-11.2-24-24-24s-24 11.2-24 24v363.2c0 51.2 41.6 92.8 92.8 92.8h358.4c51.2 0 92.8-41.6 92.8-92.8V408c-1.6-12.8-12.8-24-25.6-24z"
                        fill="#f57070" p-id="2369"></path>
                      <path
                        d="M444.8 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24zM627.2 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24z"
                        fill="#f57070" p-id="2370"></path>
                    </svg>
                  </button>
                </div>
              </div>
            </template>
          </div>

        </div>
      </div>

      <div id="stickMapTab" class="tab-content" :class="{ active: state.activeTab === 'stickMapTab' }" role="tabpanel">
        <div class="settings-container">
          <div class="setting-group">
            <h3>摇杆设置</h3>
            <div class="setting-item">
              <label for="deadzone">右摇杆死区:</label>
              <div class="slider-container">
                <input type="range" id="deadzone" min="0" max="30" v-model.number="state.current_preset.items.deadzone"
                  @change="saveDeadzoneSettings">
                <span id="deadzone-value"> {{ state.current_preset.items.deadzone }} %</span>
              </div>
            </div>
            <div class="setting-item">
              <label for="deadzone">左摇杆死区:</label>
              <div class="slider-container">
                <input type="range" id="deadzone-left" min="0" max="30"
                  v-model.number="state.current_preset.items.deadzone_left" @change="saveDeadzoneSettings">
                <span id="deadzone-left-value">{{ state.current_preset.items.deadzone_left }}%</span>
              </div>
            </div>
          </div>
          <div class="setting-group">
            <h3>摇杆旋转行为</h3>
            <div class="setting-item">
              <label>右摇杆旋转:</label>
            </div>
          </div>

        </div>
      </div>

      <div id="settingTab" class="tab-content" :class="{ active: state.activeTab === 'settingTab' }" role="tabpanel">
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
              <label for="remember-last-connection">记住上次连接状态:</label>
              <label class="switch">
                <input type="checkbox" id="remember-last-connection" v-model="state.rememberLastConnection"
                  @change="updateSettings()">
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
                <input type="number" id="polling-frequency" min="1" max="8000" value="125"
                  v-model="state.pollingFrequency" @change="setPollingFrequency()">
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
          <div class="setting-group">
            <button id="reset-btn" class="btn btn-outline btn-settings" v-if="!state.is_release_env"
              @click="openDevTools()">
              打开开发者工具
            </button>
            <button id="reset-btn" class="btn btn-outline btn-settings" @click="resetSettings()">
              重置设置
            </button>
            <button id="github-btn" class="btn btn-outline btn-settings" @click="openGithubLink()">
              GitHub 项目
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  addButtonMap,
  changeTheme,
  deleteButtonMap,
  editButtonMap,
  formatKeyDisplay,
  openDevTools,
  openGithubLink,
  resetSettings,
  saveDeadzoneSettings,
  setPollingFrequency,
  switchTab,
  updateSettings,
  savePreset,
  importPreset,
  switchPreset,
  newPreset,
  confirmNewPreset,
  cancelNewPreset
} from "@/ts/RightPanel.ts";
import { state } from "@/ts/global_states.ts";
import { onMounted } from "vue";
</script>

<style scoped>
.preset-input-container {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  /* border: 1px solid #dce2eb; */
  /* border-radius: 8px; */
  padding-right: 5px;
  /* height: 48px; */
}

.preset-input {
  flex: 1;
  border: none;
  /* background: transparent; */
  border-radius: 8px;
  border: 1px solid #c5cad1;
  padding: 10px;
  margin-right: 10px;
  font-size: 14px;
  outline: none;
  color: #2f3542;
  background: #F8F9FA;
}

.preset-input::placeholder {
  color: #7a859e;
}

.preset-input-controls {
  display: flex;
  gap: 4px;
}

.icon-button.delete-preset-btn {
  background-color: var(--danger);
}

.cancel-btn:hover {
  color: #2f3542;
}
</style>
