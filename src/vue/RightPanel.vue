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

          <div class="button-map">
            <div class="empty-state" v-if="state.mappings.length === 0">
              <p>
                <svg t="1753627389377" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="3769" width="32" height="32">
                  <path
                      d="M921.6 208.3c12.6 0 24.5 5 33.5 14s14 20.9 14 33.5v512.3c0 12.6-5 24.5-14 33.5s-20.9 14-33.5 14H102.4c-12.6 0-24.4-5-33.4-14s-14-20.9-14-33.5V255.9c0-12.6 5-24.5 14-33.5s20.9-14 33.4-14h819.2m0-55.1H102.4C46.1 153.3 0 199.4 0 255.9v512.3c0 56.4 46.1 102.5 102.4 102.5h819.1c56.4 0 102.4-46.1 102.4-102.5V255.9c0.1-56.5-45.9-102.6-102.3-102.6zM460.8 307h102.4v102.5H460.8V307z m0 153.7h102.4v102.5H460.8V460.7zM307.2 307h102.4v102.5H307.2V307z m0 153.7h102.4v102.5H307.2V460.7zM256 563.2H153.7V460.9H256v102.3z m0-153.6H153.7V307.2H256v102.4z m614.4 358.5H153.8V665.8h716.6v102.3zM716.8 563.2H614.5V460.9h102.3v102.3z m0-153.6H614.5V307.2h102.3v102.4z m153.6 153.6H768.1V460.9h102.3v102.3z m0-153.6H768.1V307.2h102.3v102.4z m0 0"
                      p-id="3770" fill="#3A7DE0"></path>
                </svg>
                <br>
                尚未添加任何按键映射 <br>
                点击右上角的
                <svg t="1753627455046" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="4836" width="32" height="32">
                  <path d="M508.9 926.4c-36.3-1.6-64.6-32.2-64.6-68.6V166.1c0-36.3 28.3-66.9 64.6-68.6 38.8-1.7 70.7 29.2 70.7 67.6v693.7c0 38.4-31.9 69.4-70.7 67.6z" fill="#4C8BF5" p-id="4837"></path>
                  <path d="M858.9 579.6H165.2c-37.4 0-67.6-30.3-67.6-67.6 0-37.4 30.3-67.6 67.6-67.6h693.7c37.4 0 67.6 30.3 67.6 67.6 0 37.4-30.3 67.6-67.6 67.6z" fill="#4C8BF5" p-id="4838"></path>
                </svg>
                按钮添加映射
              </p>
            </div>

            <template v-else>
              <div class="button-map-item" v-for="mapping in state.mappings" :key="mapping.id" @dblclick="editButtonMap(mapping.id)">
                <div class="button-icon">{{ mapping.composed_button }}</div>
                <div class="key-text">映射到</div>
                <div class="key-value">{{ formatKeyDisplay(mapping.composed_shortcut_key) }}</div>
                <div class="item-actions">
                  <button class="item-action-btn edit" @click="editButtonMap(mapping.id)">
                    <svg t="1753769162786" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="3801" width="200" height="200"><path d="M869.62198 290.936185c-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 501.688143c0 40.342824-32.8205 73.163323-73.163323 73.163323L252.963339 897.142777c-40.342824 0-73.163323-32.8205-73.163323-73.163323l0-606.206592c0-40.342824 32.8205-73.163323 73.163323-73.163323l407.621744 0c17.316387 0 31.355125-14.039761 31.355125-31.355125s-14.039761-31.355125-31.355125-31.355125L252.963339 81.899288c-74.92341 0-135.873574 60.950164-135.873574 135.873574l0 606.206592c0 74.92341 60.950164 135.873574 135.873574 135.873574l512.140193 0c74.92341 0 135.873574-60.950164 135.873574-135.873574L900.977106 322.292334C900.978129 304.975946 886.938368 290.936185 869.62198 290.936185z" fill="#707070" p-id="3802"></path><path d="M535.946388 467.382826c6.01704 5.496178 13.59053 8.205892 21.143553 8.205892 8.502651 0 16.97358-3.434216 23.159466-10.201339L898.602012 116.986411c11.682064-12.779048 10.783601-32.615838-1.995447-44.297902-12.784164-11.676947-32.615838-10.783601-44.303019 2.000564L533.950941 423.084924C522.269901 435.863972 523.167341 455.700763 535.946388 467.382826z" fill="#707070" p-id="3803"></path><path d="M355.315448 594.978876l0 30.589692c0 17.316387 14.039761 31.355125 31.355125 31.355125 17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125C369.355209 563.623751 355.315448 577.663512 355.315448 594.978876z" fill="#707070" p-id="3804"></path><path d="M631.396297 656.924717c17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 30.589692C600.041172 642.884956 614.07991 656.924717 631.396297 656.924717z" fill="#707070" p-id="3805"></path></svg>
                  </button>
                  <button class="item-action-btn delete" @click="deleteButtonMap(mapping.id)">
                    <svg t="1753765954234" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="2368" width="200" height="200"><path d="M840 288H688v-56c0-40-32-72-72-72h-208C368 160 336 192 336 232V288h-152c-12.8 0-24 11.2-24 24s11.2 24 24 24h656c12.8 0 24-11.2 24-24s-11.2-24-24-24zM384 288v-56c0-12.8 11.2-24 24-24h208c12.8 0 24 11.2 24 24V288H384zM758.4 384c-12.8 0-24 11.2-24 24v363.2c0 24-19.2 44.8-44.8 44.8H332.8c-24 0-44.8-19.2-44.8-44.8V408c0-12.8-11.2-24-24-24s-24 11.2-24 24v363.2c0 51.2 41.6 92.8 92.8 92.8h358.4c51.2 0 92.8-41.6 92.8-92.8V408c-1.6-12.8-12.8-24-25.6-24z" fill="#f57070" p-id="2369"></path><path d="M444.8 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24zM627.2 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24z" fill="#f57070" p-id="2370"></path></svg>
                  </button>
                </div>
              </div>
            </template>
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
              <button id="open-joystick-cali-modal" class="icon-button" @click="openCaliModal()">
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
                <input type="number" id="polling-frequency" min="1" max="8000" value="125" v-model="state.pollingFrequency" @change="setPollingFrequency()">
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
            <button id="reset-btn" class="btn btn-outline btn-settings" v-if="!state.is_release_env" @click="openDevTools()">
              打开开发者工具
            </button>
            <button id="reset-btn" class="btn btn-outline btn-settings" @click="resetSettings()">
              重置设置
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {addButtonMap, changeTheme, deleteButtonMap, editButtonMap, formatKeyDisplay, openDevTools, resetSettings, setPollingFrequency, switchTab, updateSettings} from "@/ts/RightPanel.ts";
import {state} from "@/ts/global_states.ts";
import {openCaliModal} from "@/ts/JoystickCaliModal.ts";
</script>

<style scoped>

</style>
