<template>
  <div class="right-panel">
    <div class="card preset-card">
      <label for="preset">{{ $t('rightPanel.preset') }}</label>
      <div class="preset-header">
        <!-- 正常模式：显示下拉框 -->
        <select v-if="!state.isCreatingNewPreset" id="preset" class="preset-select" v-model="state.previousPreset"
          @change="switchPreset()">
          <option disabled value="">{{ $t('rightPanel.selectPreset') }}</option>
          <option v-for="preset in state.presets" :key="preset" :value="preset">
            {{ preset }}
          </option>
        </select>

        <!-- 新建模式：显示输入框 -->
        <div v-else class="preset-input-container">
          <input type="text" v-model="state.newPresetName" :placeholder="$t('rightPanel.enterPresetName')" class="preset-input"
            @keyup.enter="confirmNewPreset()" @keyup.escape="cancelNewPreset()" ref="presetInput" />
          <button class="icon-button confirm-btn" @click="confirmNewPreset()" :title="$t('rightPanel.confirm')">
            <svg t="1756401635650" viewBox="0 0 1024 1024" width="16" height="16">
              <path
                d="M912 190h-69.9c-9.8 0-19.1 4.5-25.1 12.2L404.7 724.5 207 474c-6-7.7-15.3-12.2-25.1-12.2H112c-6.7 0-10.4 7.7-6.3 12.9l273.9 347c12.8 16.2 37.4 16.2 50.3 0l488.4-618.9c4.1-5.1.4-12.8-6.3-12.8z"
                fill="#ffffff" />
            </svg>
          </button>
          <button class="icon-button cancel-btn" @click="cancelNewPreset()" :title="$t('rightPanel.cancel')">
            <svg t="1756401635650" viewBox="0 0 1024 1024" width="16" height="16">
              <path
                d="M563.8 512l262.5-312.9c4.4-5.2.7-13.1-6.1-13.1h-79.8c-4.7 0-9.2 2.1-12.3 5.7L511.6 449.8 295.1 191.7c-3-3.6-7.5-5.7-12.3-5.7H203c-6.8 0-10.5 7.9-6.1 13.1L459.4 512 196.9 824.9A7.95 7.95 0 0 0 203 838h79.8c4.7 0 9.2-2.1 12.3-5.7l216.5-258.1 216.5 258.1c3 3.6 7.5 5.7 12.3 5.7h79.8c6.8 0 10.5-7.9 6.1-13.1L563.8 512z"
                fill="#ffffff" />
            </svg>
          </button>
        </div>

        <div class="preset-controls">
          <button v-if="!state.isCreatingNewPreset" id="create-preset" :title="$t('rightPanel.newPreset')"
            class="icon-button create-preset-btn" @click="newPreset()">
            <svg t="1756498807768" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="12468" width="200" height="200"><path d="M341.333333 341.333333a42.666667 42.666667 0 0 1 42.666667-42.666666h256a42.666667 42.666667 0 0 1 0 85.333333H384a42.666667 42.666667 0 0 1-42.666667-42.666667z m42.666667 128a42.666667 42.666667 0 0 0 0 85.333334h128a42.666667 42.666667 0 0 0 0-85.333334H384zM810.666667 682.666667a42.666667 42.666667 0 0 0-85.333334 0v85.333333h-85.333333a42.666667 42.666667 0 0 0 0 85.333333h85.333333v85.333334a42.666667 42.666667 0 0 0 85.333334 0v-85.333334h85.333333a42.666667 42.666667 0 0 0 0-85.333333h-85.333333v-85.333333z" p-id="12469" fill="#ffffff"></path><path d="M256 85.333333h512a85.333333 85.333333 0 0 1 85.333333 85.333334v341.333333a42.666667 42.666667 0 1 1-85.333333 0V170.666667H256v682.666666h170.666667a42.666667 42.666667 0 1 1 0 85.333334H256a85.333333 85.333333 0 0 1-85.333333-85.333334V170.666667a85.333333 85.333333 0 0 1 85.333333-85.333334z" p-id="12470" fill="#ffffff"></path></svg>
          </button>
          <button v-if="!state.isCreatingNewPreset" id="edit-preset" :title="$t('rightPanel.editPreset')" class="icon-button edit-preset-btn"
            @click="editPreset()">
            <svg t="1756498642082" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg"
              p-id="8837" width="200" height="200">
              <path
                d="M869.62198 290.936185c-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 501.688143c0 40.342824-32.8205 73.163323-73.163323 73.163323L252.963339 897.142777c-40.342824 0-73.163323-32.8205-73.163323-73.163323l0-606.206592c0-40.342824 32.8205-73.163323 73.163323-73.163323l407.621744 0c17.316387 0 31.355125-14.039761 31.355125-31.355125s-14.039761-31.355125-31.355125-31.355125L252.963339 81.899288c-74.92341 0-135.873574 60.950164-135.873574 135.873574l0 606.206592c0 74.92341 60.950164 135.873574 135.873574 135.873574l512.140193 0c74.92341 0 135.873574-60.950164 135.873574-135.873574L900.977106 322.292334C900.978129 304.975946 886.938368 290.936185 869.62198 290.936185zM535.946388 467.382826c6.01704 5.496178 13.59053 8.205892 21.143553 8.205892 8.502651 0 16.97358-3.434216 23.159466-10.201339L898.602012 116.986411c11.682064-12.779048 10.783601-32.615838-1.995447-44.297902-12.784164-11.676947-32.615838-10.783601-44.303019 2.000564L533.950941 423.084924C522.269901 435.863972 523.167341 455.700763 535.946388 467.382826zM355.315448 594.978876l0 30.589692c0 17.316387 14.039761 31.355125 31.355125 31.355125 17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125C369.355209 563.623751 355.315448 577.663512 355.315448 594.978876zM631.396297 656.924717c17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 30.589692C600.041172 642.884956 614.07991 656.924717 631.396297 656.924717zM589.507258 705.233979c-14.600533-9.344832-33.978882-5.082762-43.317575 9.492188-0.122797 0.199545-13.431917 19.525706-34.101679 19.525706-20.067035 0-32.549324-18.167778-33.401738-19.443841-9.211802-14.488992-28.395724-18.877953-43.001373-9.803273-14.713097 9.140171-19.22997 28.472472-10.089799 43.180452 11.253298 18.117636 42.138726 48.77896 86.49291 48.77896 44.129056 0 75.393107-30.432103 86.911442-48.411593C608.339162 733.966371 604.088348 714.572672 589.507258 705.233979z"
                fill="#ffffff" p-id="8838"></path>
            </svg>
          </button>
          <button v-if="!state.isCreatingNewPreset" id="delete-preset" :title="$t('rightPanel.deletePreset')"
            class="icon-button delete-preset-btn" @click="deletePreset()">
            <svg t="1756468673115" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg"
              p-id="4451" width="200" height="200">
              <path
                d="M820.579976 175.423128h-168.306498V119.284837c0-30.959148-25.179143-56.138291-56.138291-56.138291h-168.306499c-30.959148 0-56.138291 25.179143-56.138291 56.138291v56.138291h-168.342623c-30.959148 0-56.029916 25.070768-56.029916 56.029917v56.138291c0 30.959148 25.106893 56.138291 56.029916 56.138291v504.95562c0 61.918295 50.249912 112.168207 112.276582 112.168207h392.751288c61.918295 0 112.168207-50.249912 112.168207-112.168207V343.729627c30.959148 0 56.138291-25.179143 56.138291-56.138291V231.453045c0.036125-30.923023-25.143018-56.029916-56.102166-56.029917z m-392.787413-28.10527a28.033021 28.033021 0 0 1 28.033021-28.033021h112.276582c15.497636 0 28.033021 12.535384 28.033021 28.033021v28.10527h-168.306499v-28.10527z m336.649122 701.367389c0 30.959148-25.070768 56.066041-56.066041 56.066041H315.624356c-31.031398 0-56.138291-25.106893-56.138291-56.066041V343.729627h504.95562v504.95562z m28.10527-561.093911H231.453045c-15.497636 0-28.105271-12.535384-28.105271-28.033021 0-15.569886 12.643759-28.105271 28.105271-28.10527h561.09391c15.461511 0 28.033021 12.535384 28.033021 28.10527 0 15.497636-12.535384 28.033021-28.033021 28.033021z"
                fill="#ffffff" p-id="4452"></path>
              <path
                d="M371.654272 455.933959c15.569886 0 28.105271 12.535384 28.105271 28.03302v336.612997c0 15.569886-12.535384 28.105271-28.105271 28.105271-15.461511 0-28.033021-12.535384-28.03302-28.105271v-336.612997c0.036125-15.497636 12.571509-28.033021 28.03302-28.03302zM511.963875 455.933959c15.497636 0 28.033021 12.535384 28.033021 28.03302v336.612997c0 15.569886-12.535384 28.105271-28.033021 28.105271s-28.033021-12.535384-28.033021-28.105271v-336.612997a28.033021 28.033021 0 0 1 28.033021-28.03302zM652.273478 455.933959c15.497636 0 28.033021 12.535384 28.03302 28.03302v336.612997c0 15.569886-12.535384 28.105271-28.03302 28.105271-15.461511 0-28.033021-12.535384-28.033021-28.105271v-336.612997c0-15.497636 12.535384-28.033021 28.033021-28.03302z"
                fill="#ffffff" p-id="4453"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="tabs" role="tablist">
        <div class="tab" :class="{ active: state.activeTab === 'buttonMapTab' }" role="tab" aria-selected="true"
          data-tab="buttonMapTab" @click="switchTab('buttonMapTab')">{{ $t('rightPanel.buttonMap') }}</div>
        <div class="tab" :class="{ active: state.activeTab === 'stickMapTab' }" role="tab" aria-selected="false"
          data-tab="stickMapTab" @click="switchTab('stickMapTab')">{{ $t('rightPanel.pressureSetting') }}</div>
        <div class="tab" :class="{ active: state.activeTab === 'settingTab' }" role="tab" aria-selected="false"
          data-tab="settingTab" @click="switchTab('settingTab')">{{ $t('rightPanel.settings') }}</div>
      </div>

      <div id="buttonMapTab" class="tab-content" :class="{ active: state.activeTab === 'buttonMapTab' }"
        role="tabpanel">
        <div class="tab-content-container">
          <div class="button-map-header">
            <div class="button-map-title"><i class="fas fa-keyboard"></i> {{ $t('rightPanel.buttonMap') }}</div>
            <div class="button-map-controls">
              <button id="add-button-map" :title="$t('rightPanel.addMapping')" class="icon-button" @click="addButtonMap()">
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
                {{ $t('rightPanel.noMappings') }} <br>
                {{ $t('rightPanel.clickToAdd') }}
                <svg t="1753627455046" class="icon" viewBox="0 0 1024 1024" version="1.1"
                  xmlns="http://www.w3.org/2000/svg" p-id="4836" width="32" height="32">
                  <path
                    d="M508.9 926.4c-36.3-1.6-64.6-32.2-64.6-68.6V166.1c0-36.3 28.3-66.9 64.6-68.6 38.8-1.7 70.7 29.2 70.7 67.6v693.7c0 38.4-31.9 69.4-70.7 67.6z"
                    fill="#4C8BF5" p-id="4837"></path>
                  <path
                    d="M858.9 579.6H165.2c-37.4 0-67.6-30.3-67.6-67.6 0-37.4 30.3-67.6 67.6-67.6h693.7c37.4 0 67.6 30.3 67.6 67.6 0 37.4-30.3 67.6-67.6 67.6z"
                    fill="#4C8BF5" p-id="4838"></path>
                </svg>
                {{ $t('rightPanel.addButton') }}
              </p>
            </div>
            <!-- <template v-else>
              <div class="button-map-item" v-for="mapping in state.mappings" :key="mapping.id"
                @dblclick="editButtonMap(mapping.id)">
                <div class="button-icon">{{ mapping.composed_button }}</div>
                <div class="key-text">{{ $t('rightPanel.mapTo') }}</div>
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
            </template> -->
            <DraggableList />
          </div>

        </div>
      </div>

      <div id="stickMapTab" class="tab-content" :class="{ active: state.activeTab === 'stickMapTab' }" role="tabpanel">
        <div class="settings-container">
          <div class="setting-group">
            <h3>{{ $t('rightPanel.stickSettings') }}</h3>
            <div class="setting-item">
              <label for="deadzone">{{ $t('rightPanel.rightStickDeadzone') }}</label>
              <div class="slider-container">
                <input type="range" id="deadzone" min="0" max="30" v-model.number="state.current_preset.items.deadzone"
                  @change="saveDeadzoneSettings">
                <span id="deadzone-value"> {{ state.current_preset.items.deadzone }} %</span>
              </div>
            </div>
            <div class="setting-item">
              <label for="deadzone">{{ $t('rightPanel.leftStickDeadzone') }}</label>
              <div class="slider-container">
                <input type="range" id="deadzone-left" min="0" max="30"
                  v-model.number="state.current_preset.items.deadzone_left" @change="saveDeadzoneSettings">
                <span id="deadzone-left-value">{{ state.current_preset.items.deadzone_left }}%</span>
              </div>
            </div>

            <div class="setting-item">
              <label for="use-stick-as-mouse">{{ $t('rightPanel.useStickAsMouse') }}</label>
              <label class="switch">
                <input type="checkbox" id="use-stick-as-mouse" v-model="state.current_preset.items.use_stick_as_mouse"
                  @change="updateStickAsMouse()">
                <span class="slider round"></span>
              </label>
            </div>

            <div class="setting-item" v-if="state.current_preset.items.use_stick_as_mouse">
              <label for="stick-as-mouse-simulation">{{ $t('rightPanel.stickAsMouseSimulation') }}</label>
              <div class="btn-group">
                <button
                  :class="['btn-switch', { 'active': state.current_preset.items.stick_as_mouse_simulation === 'left' }]"
                  @click="state.current_preset.items.stick_as_mouse_simulation = 'left'; updateStickAsMouse()">
                  {{ $t('rightPanel.leftStick') }}
                </button>
                <button
                  :class="['btn-switch', { 'active': state.current_preset.items.stick_as_mouse_simulation === 'right' }]"
                  @click="state.current_preset.items.stick_as_mouse_simulation = 'right'; updateStickAsMouse()">
                  {{ $t('rightPanel.rightStick') }}
                </button>
              </div>
            </div>

            <div class="setting-item" v-if="state.current_preset.items.use_stick_as_mouse">
              <label>{{ $t('rightPanel.mouseMoveSpeed') }}：</label>
              <div class="polling-container">
                <input type="number" min="1" max="100" value="20"
                  v-model="state.current_preset.items.move_speed"
                  @change="updateMouseMoveSpeed()">
              </div>
            </div>
          </div>

          <div class="setting-group">
            <h3>{{ $t('rightPanel.stickRotationBehavior') }}</h3>
            <div class="setting-item">
              <label>{{ $t('rightPanel.triggerAngleThreshold') }}：</label>
              <div class="polling-container">
                <input type="number" id="stick_rotate_trigger_threshold_input" min="1" max="360" value="15"
                  v-model="state.current_preset.items.stick_rotate_trigger_threshold"
                  @change="updateStickRotationThreshold()">
                <span>{{ $t('rightPanel.degrees') }}</span>
              </div>
            </div>
          </div>

        </div>
      </div>

      <div id="settingTab" class="tab-content" :class="{ active: state.activeTab === 'settingTab' }" role="tabpanel">
        <div class="settings-container">
          <div class="setting-group">
            <h3>{{ $t('rightPanel.softwareSettings') }}</h3>

            <div class="setting-item">
              <label for="auto-start">{{ $t('rightPanel.autoStart') }}</label>
              <label class="switch">
                <input type="checkbox" id="auto-start" v-model="state.autoStart" @change="updateSettings()">
                <span class="slider round"></span>
              </label>
            </div>

            <div class="setting-item">
              <label for="remember-last-connection">{{ $t('rightPanel.rememberLastConnection') }}</label>
              <label class="switch">
                <input type="checkbox" id="remember-last-connection" v-model="state.rememberLastConnection"
                  @change="updateSettings()">
                <span class="slider round"></span>
              </label>
            </div>

            <div class="setting-item">
              <label for="minimize-to-tray">{{ $t('rightPanel.minimizeToTray') }}</label>
              <label class="switch">
                <input type="checkbox" id="minimize-to-tray" v-model="state.minimizeToTray" @change="updateSettings()">
                <span class="slider round"></span>
              </label>
            </div>

            <div class="setting-item">
              <label for="polling-frequency">{{ $t('rightPanel.pollingFrequency') }}</label>
              <div class="polling-container">
                <input type="number" id="polling-frequency" min="1" max="8000" value="125"
                  v-model="state.pollingFrequency" @change="setPollingFrequency()">
                <span>Hz</span>
              </div>
            </div>

            <div class="setting-item">
              <label for="theme">{{ $t('rightPanel.uiTheme') }}</label>
              <select id="theme" v-model="state.theme" @change="changeTheme()">
                <option value="light">{{ $t('rightPanel.lightMode') }}</option>
                <option value="dark">{{ $t('rightPanel.darkMode') }}</option>
                <option value="system">{{ $t('rightPanel.followSystem') }}</option>
              </select>
            </div>

            <div class="setting-item">
              <label for="language">{{ $t('rightPanel.uiLanguage') }}</label>
              <select id="language" v-model="state.language" @change="changeLanguage()">
                <option value="system">{{ $t('rightPanel.followSystem') }}</option>
                <option value="zh-CN">简体中文</option>
                <option value="en-US">English</option>
              </select>
            </div>
          </div>
          <div class="setting-group">
            <button id="reset-btn" class="btn btn-outline btn-settings" v-if="!state.is_release_env"
              @click="openDevTools()">
              {{ $t('rightPanel.openDevTools') }}
            </button>
            <button id="reset-btn" class="btn btn-outline btn-settings" @click="resetSettings()">
              {{ $t('rightPanel.resetSettings') }}
            </button>
            <button id="github-btn" class="btn btn-outline btn-settings" @click="openGithubLink()">
              {{ $t('rightPanel.githubProject') }}
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
  changeLanguage,
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
  cancelNewPreset,
  deletePreset,
  editPreset,
  updateStickAsMouse,
  updateStickRotationThreshold,
  updateMouseMoveSpeed,
} from "@/ts/RightPanel.ts";
import { state } from "@/ts/global_states.ts";
import { onMounted, ref } from "vue";
import DraggableList from "./DraggableList.vue";
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
