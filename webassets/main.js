document.addEventListener('DOMContentLoaded', () => {
    // ======================
    // 1. 初始化部分
    // ======================
    const appWindow = window.__TAURI__?.window.getCurrentWindow();
    const invoke = window.__TAURI__?.core.invoke;

    // 缓存常用UI元素引用
    const uiElements = {
        autoStart: document.getElementById('auto-start'),
        minimizeToTray: document.getElementById('minimize-to-tray'),
        theme: document.getElementById('theme'),
        pollingFrequency: document.getElementById('polling-frequency'),
        deadzone: document.getElementById('deadzone'),
        deadzoneValue: document.getElementById('deadzone-value'),
        deadzoneLeft: document.getElementById('deadzone-left'),
        deadzoneLeftValue: document.getElementById('deadzone-left-value'),
        deviceSelect: document.getElementById('device'),
        minimizeButton: document.getElementById('minimize-button'),
        closeButton: document.getElementById('close-button'),
        scanButton: document.getElementById('scan-button'),
        connectButton: document.getElementById('connect-button'),
        preset: document.getElementById('preset'),
        savePreset: document.getElementById('save-preset'),
        importPreset: document.getElementById('import-preset'),
        addButtonMap: document.getElementById('add-button-map'),
        indicator: document.getElementById('status-indicator'),
        githubLink: document.getElementById('github-link'),
        tabs: document.querySelector('.tabs'),
        statusMessage: document.getElementById('status-message'),
        mappingList: document.getElementById('button-mapping-list'),
        keyDetector: document.getElementById('key-detector-area'),
        keyDisplay: document.getElementById('key-display'),
        controllerButtonSelect: document.getElementById('controller-button'),
        modal: document.getElementById('mapping-modal'),
        modalTitle: document.getElementById('modal-title'),
        modalError: document.getElementById('modal-error')
    };

    // 应用状态变量
    const state = {
        hasUserSelectedDevice: false,
        currentDevices: [],
        deviceSelected: null,
        minimizeToTray: true,
        isConnected: false,
        deviceType: 'xbox',
        mappings: [],
        editingMappingId: null,
        keyListenerActive: false,
        preventNextClick: false,
        currentKeys: {ctrl: false, shift: false, alt: false, meta: false, key: null}
    };

    // 连接状态图标SVG
    const icons = {
        connected: `<svg t="1753591538385" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="19479" width="200" height="200"><path d="M728.96 269.44a207.36 207.36 0 1 0-414.08 0v133.12a239.36 239.36 0 0 1 96-93.44v-39.68a111.36 111.36 0 1 1 222.08 0v200.96a111.36 111.36 0 0 1-111.36 111.36 110.08 110.08 0 0 1-69.76-25.6v108.8a203.52 203.52 0 0 0 69.76 12.8 207.36 207.36 0 0 0 207.36-207.36z" fill="#ffffff" p-id="19480"></path><path d="M632.96 680.32v58.88a111.36 111.36 0 1 1-222.08 0V520.32a111.36 111.36 0 0 1 110.72-111.36 110.08 110.08 0 0 1 42.88 8.96 112.64 112.64 0 0 1 26.88 16.64v-108.8a204.8 204.8 0 0 0-69.76-12.8 207.36 207.36 0 0 0-206.72 207.36v219.52a207.36 207.36 0 1 0 414.08 0V588.16a238.72 238.72 0 0 1-96 92.16z" fill="#ffffff" p-id="19481"></path></svg>`,
        disconnected: `<svg t="1753595424804" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="19662" width="200" height="200"><path d="M521.6 158.08a111.36 111.36 0 0 1 111.36 111.36V480l96 96V269.44a206.72 206.72 0 0 0-394.24-87.68L411.52 256a110.72 110.72 0 0 1 110.08-97.92zM864 846.08l-135.04-135.04-96-96-222.08-222.08-96-96-135.04-134.4L112 230.4l202.88 202.24v306.56a206.72 206.72 0 0 0 394.24 87.68l87.04 87.04z m-341.76 4.48a111.36 111.36 0 0 1-111.36-111.36V528.64l221.44 221.44a110.72 110.72 0 0 1-110.72 100.48z" fill="#ffffff" p-id="19663"></path></svg>`
    };

    // 特殊键的显示名称映射
    const keyDisplayNames = {
        ' ': '空格键',
        'Control': 'Ctrl',
        'Shift': 'Shift',
        'Alt': 'Alt',
        'Meta': 'Cmd',
        'ArrowUp': '↑',
        'ArrowDown': '↓',
        'ArrowLeft': '←',
        'ArrowRight': '→',
        'Escape': 'Esc',
        'Tab': 'Tab',
        'CapsLock': 'Caps Lock',
        'Enter': 'Enter',
        'Backspace': 'Backspace',
        'Delete': 'Delete',
        'Insert': 'Insert',
        'Home': 'Home',
        'End': 'End',
        'PageUp': 'Page Up',
        'PageDown': 'Page Down',
        'ContextMenu': '菜单键',
        'F1': 'F1',
        'F2': 'F2',
        'F3': 'F3',
        'F4': 'F4',
        'F5': 'F5',
        'F6': 'F6',
        'F7': 'F7',
        'F8': 'F8',
        'F9': 'F9',
        'F10': 'F10',
        'F11': 'F11',
        'F12': 'F12',
        'MouseLeft': '鼠标左键',
        'MouseRight': '鼠标右键',
        'MouseMiddle': '鼠标中键',
        'MouseX1': '鼠标侧键1',
        'MouseX2': '鼠标侧键2',
        'MouseWheelUp': '滚轮上',
        'MouseWheelDown': '滚轮下'
    };

    // ======================
    // 2. 通用功能函数
    // ======================

    // 更新状态消息
    function updateStatusMessage(message, isError = false) {
        uiElements.statusMessage.textContent = message;
        uiElements.statusMessage.className = 'status-message';
        if (isError) uiElements.statusMessage.classList.add('error');
        else if (message.includes('成功') || message.includes('连接')) {
            uiElements.statusMessage.classList.add('success');
        }
    }

    // 切换指示灯
    function toggleIndicator(isOn) {
        isOn ?
            uiElements.indicator.classList.add('on') :
            uiElements.indicator.classList.remove('on');
    }

    // 切换选项卡
    function switchTab(tabElement) {
        const targetTab = tabElement.dataset.tab;
        document.querySelectorAll('.tab').forEach(t => {
            t.classList.remove('active');
            t.setAttribute('aria-selected', 'false');
        });
        document.querySelectorAll('.tab-content').forEach(c => {
            c.classList.remove('active');
        });
        tabElement.classList.add('active');
        tabElement.setAttribute('aria-selected', 'true');
        document.getElementById(targetTab).classList.add('active');
    }

    // 应用主题设置
    function applyTheme(theme) {
        document.body.classList.remove('theme-light', 'theme-dark', 'theme-system');
        document.body.classList.add(`theme-${theme}`);

        const themeTextMap = {
            light: '浅色模式',
            dark: '深色模式',
            system: '跟随系统'
        };
        updateStatusMessage(`已切换到${themeTextMap[theme]}主题`);
    }

    // ======================
    // 3. 映射配置功能
    // ======================

    // 更新按键显示
    function updateKeyDisplay() {
        let displayText = '';
        let isMouseKey = false;

        if (state.currentKeys.ctrl) displayText += 'Ctrl + ';
        if (state.currentKeys.shift) displayText += 'Shift + ';
        if (state.currentKeys.alt) displayText += 'Alt + ';
        if (state.currentKeys.meta) displayText += 'Cmd + ';

        if (state.currentKeys.key) {
            const key = state.currentKeys.key;
            displayText += keyDisplayNames[key] || key.toUpperCase();
            isMouseKey = key.startsWith('Mouse');
        }

        uiElements.keyDisplay.textContent = displayText;
        uiElements.keyDisplay.classList.toggle('mouse', isMouseKey);
    }

    // 处理按键事件
    function handleKeyDown(e) {
        e.preventDefault();
        if (e.key === 'Control' || e.key === 'Ctrl') state.currentKeys.ctrl = true;
        else if (e.key === 'Shift') state.currentKeys.shift = true;
        else if (e.key === 'Alt') state.currentKeys.alt = true;
        else if (e.key === 'Meta') state.currentKeys.meta = true;
        else state.currentKeys.key = e.key;
        updateKeyDisplay();
    }

    function handleKeyUp(e) {
        if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
            stopKeyDetection();
        }
    }

    // 处理鼠标事件
    function handleMouseDown(e) {
        e.preventDefault();
        e.stopPropagation();
        state.preventNextClick = true;

        state.currentKeys.ctrl = e.ctrlKey;
        state.currentKeys.shift = e.shiftKey;
        state.currentKeys.alt = e.altKey;
        state.currentKeys.meta = e.metaKey;

        const mouseKeys = ['MouseLeft', 'MouseMiddle', 'MouseRight', 'MouseX1', 'MouseX2'];
        state.currentKeys.key = mouseKeys[e.button] || null;

        if (state.currentKeys.key) {
            updateKeyDisplay();
            stopKeyDetection(false);
            window.removeEventListener('mouseup', handleMouseUp);
        }
    }

    function handleMouseUp(e) {
        stopMouseDetection();
    }

    // 处理滚轮事件
    function handleWheel(e) {
        e.preventDefault();
        e.stopPropagation();

        state.currentKeys.ctrl = e.ctrlKey;
        state.currentKeys.shift = e.shiftKey;
        state.currentKeys.alt = e.altKey;
        state.currentKeys.meta = e.metaKey;
        state.currentKeys.key = e.deltaY < 0 ? 'MouseWheelUp' : 'MouseWheelDown';

        updateKeyDisplay();
        stopKeyDetection(false);
    }

    // 开始按键检测
    function startKeyDetection() {
        if (state.keyListenerActive) return;
        state.preventNextClick = false;
        state.keyListenerActive = true;
        state.currentKeys = {ctrl: false, shift: false, alt: false, meta: false, key: null};

        uiElements.keyDetector.classList.add('active');
        uiElements.keyDetector.textContent = '请按下键盘按键、鼠标按键或滚动滚轮...';
        uiElements.keyDisplay.textContent = '';

        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('keyup', handleKeyUp);
        window.addEventListener('mousedown', handleMouseDown);
        window.addEventListener('mouseup', handleMouseUp);
        window.addEventListener('wheel', handleWheel);
    }

    // 停止按键检测
    function stopKeyDetection(resetText = true) {
        if (!state.keyListenerActive) return;
        state.keyListenerActive = false;
        uiElements.keyDetector.classList.remove('active');

        if (resetText) {
            uiElements.keyDetector.textContent = '点击此处并按下键盘按键、鼠标按键或滚动滚轮';
        }

        removeKeyListeners();
    }

    function stopMouseDetection() {
        if (!state.keyListenerActive) return;
        window.removeEventListener('mousedown', handleMouseDown);
        window.removeEventListener('mouseup', handleMouseUp);
        uiElements.keyDetector.classList.remove('active');
        state.keyListenerActive = false;
    }

    // 移除按键监听器
    function removeKeyListeners() {
        window.removeEventListener('keydown', handleKeyDown);
        window.removeEventListener('keyup', handleKeyUp);
        window.removeEventListener('mousedown', handleMouseDown);
        window.removeEventListener('mouseup', handleMouseUp);
        window.removeEventListener('wheel', handleWheel);
    }

    // 打开模态窗口
    function openModal(title, controllerValue, keyDisplayText, mappingId = null) {
        uiElements.modalError.style.display = 'none';
        uiElements.modalError.textContent = '';
        uiElements.modalTitle.textContent = title;
        uiElements.controllerButtonSelect.value = controllerValue || "";
        uiElements.keyDisplay.textContent = keyDisplayText || "";
        state.editingMappingId = mappingId;
        updateControllerButtons();
        uiElements.modal.classList.add('active');
    }

    // 关闭模态窗口
    function closeModalFunc() {
        uiElements.modal.classList.remove('active');
        stopKeyDetection(true);
    }

    // 根据设备类型更新手柄按键选项
    function updateControllerButtons() {
        while (uiElements.controllerButtonSelect.options.length > 1) {
            uiElements.controllerButtonSelect.remove(1);
        }

        let buttons = [];
        if (state.deviceType === 'xbox') {
            buttons = [
                {value: 'A', text: 'A 按钮'},
                {value: 'B', text: 'B 按钮'},
                {value: 'X', text: 'X 按钮'},
                {value: 'Y', text: 'Y 按钮'},
                {value: 'LB', text: '左肩键 (LB)'},
                {value: 'RB', text: '右肩键 (RB)'},
                {value: 'LT', text: '左扳机 (LT)'},
                {value: 'RT', text: '右扳机 (RT)'},
                {value: 'START', text: '开始按钮'},
                {value: 'SELECT', text: '选择按钮'}
            ];
        } else if (state.deviceType === 'ps') {
            buttons = [
                {value: 'CROSS', text: '叉按钮 (CROSS)'},
                {value: 'CIRCLE', text: '圆按钮 (CIRCLE)'},
                {value: 'SQUARE', text: '方按钮 (SQUARE)'},
                {value: 'TRIANGLE', text: '三角按钮 (TRIANGLE)'},
                {value: 'L1', text: '左肩键 (L1)'},
                {value: 'R1', text: '右肩键 (R1)'},
                {value: 'L2', text: '左扳机 (L2)'},
                {value: 'R2', text: '右扳机 (R2)'},
                {value: 'OPTIONS', text: '选项按钮'},
                {value: 'SHARE', text: '分享按钮'}
            ];
        } else if (state.deviceType === 'switchpro') {
            buttons = [
                {value: 'B', text: 'B 按钮'},
                {value: 'A', text: 'A 按钮'},
                {value: 'Y', text: 'Y 按钮'},
                {value: 'X', text: 'X 按钮'},
                {value: 'L', text: '左肩键 (L)'},
                {value: 'R', text: '右肩键 (R)'},
                {value: 'ZL', text: '左扳机 (ZL)'},
                {value: 'ZR', text: '右扳机 (ZR)'},
                {value: 'PLUS', text: '加号按钮'},
                {value: 'MINUS', text: '减号按钮'}
            ];
        }

        buttons.forEach(button => {
            const option = document.createElement('option');
            option.value = button.value;
            option.textContent = button.text;
            uiElements.controllerButtonSelect.appendChild(option);
        });
    }

    // 渲染映射列表
    function renderMappings() {
        if (state.mappings.length === 0) {
            uiElements.mappingList.innerHTML = `
                <div class="empty-state">
                    <svg t="1753627389377" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="3769" width="32" height="32"><path d="M921.6 208.3c12.6 0 24.5 5 33.5 14s14 20.9 14 33.5v512.3c0 12.6-5 24.5-14 33.5s-20.9 14-33.5 14H102.4c-12.6 0-24.4-5-33.4-14s-14-20.9-14-33.5V255.9c0-12.6 5-24.5 14-33.5s20.9-14 33.4-14h819.2m0-55.1H102.4C46.1 153.3 0 199.4 0 255.9v512.3c0 56.4 46.1 102.5 102.4 102.5h819.1c56.4 0 102.4-46.1 102.4-102.5V255.9c0.1-56.5-45.9-102.6-102.3-102.6zM460.8 307h102.4v102.5H460.8V307z m0 153.7h102.4v102.5H460.8V460.7zM307.2 307h102.4v102.5H307.2V307z m0 153.7h102.4v102.5H307.2V460.7zM256 563.2H153.7V460.9H256v102.3z m0-153.6H153.7V307.2H256v102.4z m614.4 358.5H153.8V665.8h716.6v102.3zM716.8 563.2H614.5V460.9h102.3v102.3z m0-153.6H614.5V307.2h102.3v102.4z m153.6 153.6H768.1V460.9h102.3v102.3z m0-153.6H768.1V307.2h102.3v102.4z m0 0" p-id="3770" fill="#3A7DE0"></path></svg>
                    <p>尚未添加任何按键映射</p>
                    <p>点击右上角的
                      <svg t="1753627455046" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="4836" width="32" height="32"><path d="M508.9 926.4c-36.3-1.6-64.6-32.2-64.6-68.6V166.1c0-36.3 28.3-66.9 64.6-68.6 38.8-1.7 70.7 29.2 70.7 67.6v693.7c0 38.4-31.9 69.4-70.7 67.6z" fill="#4C8BF5" p-id="4837"></path><path d="M858.9 579.6H165.2c-37.4 0-67.6-30.3-67.6-67.6 0-37.4 30.3-67.6 67.6-67.6h693.7c37.4 0 67.6 30.3 67.6 67.6 0 37.4-30.3 67.6-67.6 67.6z" fill="#4C8BF5" p-id="4838"></path></svg>
                      按钮添加映射
                    </p>
                </div>
            `;
            return;
        }

        uiElements.mappingList.innerHTML = '';

        state.mappings.forEach(mapping => {
            const item = document.createElement('div');
            item.className = 'button-map-item';
            item.innerHTML = `
                <div class="button-icon">${mapping.composed_button}</div>
                <div class="key-text">映射到</div>
                <div class="key-value">${mapping.composed_shortcut_key}</div>
                <div class="item-actions">
                    <button class="item-action-btn edit" data-id="${mapping.id}">
                        <svg t="1753769162786" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="3801" width="200" height="200"><path d="M869.62198 290.936185c-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 501.688143c0 40.342824-32.8205 73.163323-73.163323 73.163323L252.963339 897.142777c-40.342824 0-73.163323-32.8205-73.163323-73.163323l0-606.206592c0-40.342824 32.8205-73.163323 73.163323-73.163323l407.621744 0c17.316387 0 31.355125-14.039761 31.355125-31.355125s-14.039761-31.355125-31.355125-31.355125L252.963339 81.899288c-74.92341 0-135.873574 60.950164-135.873574 135.873574l0 606.206592c0 74.92341 60.950164 135.873574 135.873574 135.873574l512.140193 0c74.92341 0 135.873574-60.950164 135.873574-135.873574L900.977106 322.292334C900.978129 304.975946 886.938368 290.936185 869.62198 290.936185z" fill="#707070" p-id="3802"></path><path d="M535.946388 467.382826c6.01704 5.496178 13.59053 8.205892 21.143553 8.205892 8.502651 0 16.97358-3.434216 23.159466-10.201339L898.602012 116.986411c11.682064-12.779048 10.783601-32.615838-1.995447-44.297902-12.784164-11.676947-32.615838-10.783601-44.303019 2.000564L533.950941 423.084924C522.269901 435.863972 523.167341 455.700763 535.946388 467.382826z" fill="#707070" p-id="3803"></path><path d="M355.315448 594.978876l0 30.589692c0 17.316387 14.039761 31.355125 31.355125 31.355125 17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125C369.355209 563.623751 355.315448 577.663512 355.315448 594.978876z" fill="#707070" p-id="3804"></path><path d="M631.396297 656.924717c17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 30.589692C600.041172 642.884956 614.07991 656.924717 631.396297 656.924717z" fill="#707070" p-id="3805"></path></svg>
                    </button>
                    <button class="item-action-btn delete" data-id="${mapping.id}">
                        <svg t="1753765954234" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="2368" width="200" height="200"><path d="M840 288H688v-56c0-40-32-72-72-72h-208C368 160 336 192 336 232V288h-152c-12.8 0-24 11.2-24 24s11.2 24 24 24h656c12.8 0 24-11.2 24-24s-11.2-24-24-24zM384 288v-56c0-12.8 11.2-24 24-24h208c12.8 0 24 11.2 24 24V288H384zM758.4 384c-12.8 0-24 11.2-24 24v363.2c0 24-19.2 44.8-44.8 44.8H332.8c-24 0-44.8-19.2-44.8-44.8V408c0-12.8-11.2-24-24-24s-24 11.2-24 24v363.2c0 51.2 41.6 92.8 92.8 92.8h358.4c51.2 0 92.8-41.6 92.8-92.8V408c-1.6-12.8-12.8-24-25.6-24z" fill="#f57070" p-id="2369"></path><path d="M444.8 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24zM627.2 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24z" fill="#f57070" p-id="2370"></path></svg>
                    </button>
                </div>
            `;
            uiElements.mappingList.appendChild(item);
        });
    }

    // 根据设备名称确定设备类型
    function detectDeviceType(deviceName) {
        if (deviceName.includes('Xbox')) return 'xbox';
        if (deviceName.includes('PS') || deviceName.includes('PlayStation')) return 'ps';
        if (deviceName.includes('Switch')) return 'switchpro';
        return 'xbox';
    }

    // 设备连接后更新映射配置
    function updateMappingsForDevice(deviceName) {
        state.deviceType = detectDeviceType(deviceName);
        updateControllerButtons();
        renderMappings();
    }

    // ======================
    // 4. 设置管理函数
    // ======================

    // 加载应用设置
    async function loadSettings() {
        if (!invoke) return;
        try {
            const settings = await invoke("get_current_settings");
            uiElements.autoStart.checked = settings.auto_start;
            uiElements.minimizeToTray.checked = settings.minimize_to_tray;
            uiElements.theme.value = settings.theme;
            uiElements.pollingFrequency.value = settings.polling_frequency;
            uiElements.deadzone.value = settings.deadzone;
            uiElements.deadzoneValue.textContent = settings.deadzone + '%';
            uiElements.deadzoneLeft.value = settings.deadzone_left || 10;
            uiElements.deadzoneLeftValue.textContent = (settings.deadzone_left || 10) + '%';

            state.minimizeToTray = settings.minimize_to_tray;
            invoke("set_frequency", {freq: settings.polling_frequency});
            applyTheme(settings.theme);
        } catch (error) {
            console.error("加载设置失败:", error);
        }
    }

    // 保存应用设置
    async function saveSettings() {
        if (!invoke) return;
        const newSettings = {
            auto_start: uiElements.autoStart.checked,
            minimize_to_tray: uiElements.minimizeToTray.checked,
            theme: uiElements.theme.value,
            polling_frequency: parseInt(uiElements.pollingFrequency.value),
            deadzone: parseInt(uiElements.deadzone.value),
            deadzone_left: parseInt(uiElements.deadzoneLeft.value)
        };
        try {
            await invoke("update_settings", {newSettings});
        } catch (error) {
            console.error("保存设置失败:", error);
        }
    }

    // ======================
    // 5. 设备管理函数
    // ======================

    // 更新设备列表
    function updateDeviceList(devices) {
        if (devices.length === 0) state.hasUserSelectedDevice = false;
        if (state.hasUserSelectedDevice) {
            toggleIndicator(devices.length > 0);
            return;
        }
        if (JSON.stringify(state.currentDevices) === JSON.stringify(devices)) return;

        state.currentDevices = devices;
        uiElements.deviceSelect.innerHTML = '';
        const defaultOption = document.createElement('option');
        defaultOption.textContent = '请选择设备';
        defaultOption.disabled = true;
        defaultOption.selected = true;
        defaultOption.value = "null";
        uiElements.deviceSelect.appendChild(defaultOption);

        state.currentDevices.forEach((device, index) => {
            const option = document.createElement('option');
            option.value = index.toString();
            option.textContent = `${index}: ${device.name}`;
            uiElements.deviceSelect.appendChild(option);
        });
    }

    // 断开当前设备
    async function disconnectCurrentDevice() {
        if (!state.isConnected || !state.deviceSelected) return true;

        const deviceName = state.deviceSelected.name;
        updateStatusMessage(`正在断开设备: ${deviceName}...`);

        try {
            await invoke("disconnect_device", {deviceName});
            updateStatusMessage(`设备已断开`);
            uiElements.connectButton.classList.remove('connected');
            uiElements.connectButton.innerHTML = icons.disconnected;
            toggleIndicator(false);
            state.isConnected = false;
            return true;
        } catch (error) {
            console.error("断开连接失败:", error);
            updateStatusMessage(`断开失败`, true);
            return false;
        }
    }

    // 断开设备并查询新设备
    async function closeAndQueryDevice() {
        await disconnectCurrentDevice();
        state.hasUserSelectedDevice = false;
        state.deviceSelected = null;
        uiElements.deviceSelect.selectedIndex = 0;

        try {
            const devices = await invoke("query_devices");
            updateDeviceList(devices);
        } catch (error) {
            console.error("查询设备失败:", error);
            updateStatusMessage("设备查询失败", true);
        }
    }

    // 去除设备名前缀
    function stripDevicePrefix(name) {
        const index = name.indexOf(":");
        return index !== -1 ? name.slice(index + 1).trim() : name;
    }

    // 切换设备连接状态
    async function toggleDeviceConnection() {
        if (!state.deviceSelected || uiElements.deviceSelect.value === "null") {
            updateStatusMessage("请先选择一个设备", true);
            return;
        }

        state.isConnected = !state.isConnected;
        const deviceName = stripDevicePrefix(state.deviceSelected.name);
        const controllerType = state.deviceSelected.controller_type;

        if (state.isConnected) {
            updateStatusMessage(`正在连接设备: ${deviceName}...`);
            if (await invoke("use_device", {deviceName})) {
                updateStatusMessage(`设备 ${deviceName} 已成功连接`);
                uiElements.connectButton.classList.add('connected');
                uiElements.connectButton.innerHTML = icons.connected;
                toggleIndicator(true);
                updateMappingsForDevice(controllerType);
            } else {
                updateStatusMessage(`连接失败`, true);
                uiElements.connectButton.classList.remove('connected');
                uiElements.connectButton.innerHTML = icons.disconnected;
                toggleIndicator(false);
                state.isConnected = false;
            }
        } else {
            updateStatusMessage(`正在断开设备: ${deviceName}...`);
            if (await invoke("disconnect_device", {deviceName})) {
                updateStatusMessage(`设备已断开`);
                uiElements.connectButton.classList.remove('connected');
                uiElements.connectButton.innerHTML = icons.disconnected;
                toggleIndicator(false);
            } else {
                updateStatusMessage(`断开失败`, true);
                uiElements.connectButton.classList.add('connected');
                uiElements.connectButton.innerHTML = icons.disconnected;
                toggleIndicator(false);
                state.isConnected = true;
            }
        }
    }

    // ======================
    // 6. 初始化函数
    // ======================

    // 加载映射配置
    async function loadMappings() {
        if (!invoke) return;
        state.mappings = await invoke("get_mappings");
        renderMappings();
    }

    // 获取平台信息并调整标题栏显示
    async function titlebarLogic() {
        if (!invoke) return;
        const platform = await invoke("get_platform");
        const titlebar = document.getElementById("titlebar");
        if (platform === "windows") {
            titlebar.classList.add("show");
            titlebar.classList.remove("hide");
        } else if (platform === "linux") {
            titlebar.classList.add("hide");
            titlebar.classList.remove("show");
        } else {
            console.warn(`未知平台: ${platform}`);
        }
    }

    // 处理轮询频率变化
    function handlePollingFrequencyChange() {
        const min = 1, max = 8000;
        let value = parseInt(uiElements.pollingFrequency.value);
        value = Math.min(Math.max(value, min), max);
        uiElements.pollingFrequency.value = value;

        if (invoke) invoke("set_frequency", {freq: value});
        saveSettings();
    }

    // ======================
    // 7. 事件监听器
    // ======================

    function setupEventListeners() {
        // 窗口控制按钮
        uiElements.minimizeButton?.addEventListener('click', () => appWindow?.minimize());
        uiElements.closeButton?.addEventListener('click', () => {
            state.minimizeToTray ? appWindow?.hide() : appWindow?.close();
        });

        // 设备选择
        uiElements.deviceSelect?.addEventListener('change', async function () {
            if (this.value === "null") {
                state.deviceSelected = null;
                updateStatusMessage("请选择一个设备");
                return;
            }

            const selectedIndex = parseInt(this.value);
            if (isNaN(selectedIndex)) {
                updateStatusMessage("无效的设备选择", true);
                return;
            }

            if (state.isConnected && state.deviceSelected) {
                const success = await disconnectCurrentDevice();
                if (!success) {
                    this.value = state.currentDevices.indexOf(state.deviceSelected).toString();
                    return;
                }
            }

            if (selectedIndex >= 0 && selectedIndex < state.currentDevices.length) {
                state.deviceSelected = state.currentDevices[selectedIndex];
                state.hasUserSelectedDevice = true;
                updateStatusMessage(`已选择设备: ${state.deviceSelected.name}`);
            } else {
                state.deviceSelected = null;
                updateStatusMessage(`无效的设备索引: ${selectedIndex}`, true);
            }
        });

        // 扫描设备
        uiElements.scanButton?.addEventListener('click', async () => {
            uiElements.scanButton.classList.add('scanning');
            updateStatusMessage("正在扫描设备...");
            const icon = document.querySelector("#scan-button .icon");
            icon.classList.add("spin");

            await closeAndQueryDevice();

            icon.addEventListener("animationend", () => {
                icon.classList.remove("spin");
            }, {once: true});

            updateStatusMessage(`扫描完成！发现${state.currentDevices.length}个可用设备`);
            uiElements.scanButton.classList.remove('scanning');
        });

        // 连接/断开设备
        uiElements.connectButton?.addEventListener('click', toggleDeviceConnection);

        // 预设管理
        uiElements.preset?.addEventListener('change', function () {
            updateStatusMessage(`已切换到预设方案: ${this.value}`);
        });
        uiElements.savePreset?.addEventListener('click', function () {
            updateStatusMessage('预设方案已保存');
        });
        uiElements.importPreset?.addEventListener('click', function () {
            updateStatusMessage('预设方案导入成功');
        });

        // 死区设置
        uiElements.deadzone?.addEventListener('input', function () {
            uiElements.deadzoneValue.textContent = this.value + '%';
        });
        uiElements.deadzone?.addEventListener('mouseup', saveSettings);

        uiElements.deadzoneLeft?.addEventListener('input', function () {
            uiElements.deadzoneLeftValue.textContent = this.value + '%';
        });
        uiElements.deadzoneLeft?.addEventListener('mouseup', saveSettings);

        // 设置变更
        uiElements.pollingFrequency?.addEventListener('change', handlePollingFrequencyChange);
        uiElements.autoStart?.addEventListener('change', function () {
            saveSettings();
            updateStatusMessage(`开机自启动已${this.checked ? '启用' : '禁用'}`);
        });
        uiElements.minimizeToTray?.addEventListener('change', function () {
            state.minimizeToTray = this.checked;
            saveSettings();
            updateStatusMessage(`最小化到托盘已${this.checked ? '启用' : '禁用'}`);
        });

        // GitHub链接
        uiElements.githubLink?.addEventListener('click', async (e) => {
            if (window.__TAURI__ && invoke) {
                e.preventDefault();
                invoke("open_url", {url: uiElements.githubLink.href});
            }
        });

        // 主题切换
        uiElements.theme?.addEventListener('change', function () {
            saveSettings();
            applyTheme(this.value);
        });

        // 映射功能
        uiElements.addButtonMap?.addEventListener('click', () => openModal("添加按键映射", "", ""));
        uiElements.keyDetector?.addEventListener('click', (e) => {
            if (state.preventNextClick) {
                state.preventNextClick = false;
                e.stopPropagation();
                e.preventDefault();
                return;
            }
            startKeyDetection();
        });
        document.getElementById('close-modal')?.addEventListener('click', closeModalFunc);
        document.getElementById('cancel-btn')?.addEventListener('click', closeModalFunc);

        // 确认映射按钮
        document.getElementById('confirm-btn')?.addEventListener('click', function () {
            const composed_button = uiElements.controllerButtonSelect.value;
            const composed_shortcut_key = uiElements.keyDisplay.textContent;

            stopKeyDetection();
            uiElements.modalError.style.display = 'none';
            uiElements.modalError.textContent = '';

            if (!composed_button) {
                uiElements.modalError.textContent = '请选择手柄按键';
                uiElements.modalError.style.display = 'block';
                return;
            }

            if (!composed_shortcut_key) {
                uiElements.modalError.textContent = '请设置键盘映射按键';
                uiElements.modalError.style.display = 'block';
                return;
            }

            if (state.editingMappingId) {
                const mapping = state.mappings.find(m => m.id === state.editingMappingId);
                if (mapping) {
                    mapping.composed_button = composed_button;
                    mapping.composed_shortcut_key = composed_shortcut_key;
                }
            } else {
                state.mappings.push({
                    id: Date.now(),
                    composed_button: composed_button,
                    composed_shortcut_key: composed_shortcut_key,
                    mapping_type: 'keyboard'
                });
            }

            if (invoke) invoke('set_mapping', {mapping: state.mappings});
            renderMappings();
            closeModalFunc();
        });

        // 选项卡切换
        if (uiElements.tabs) {
            uiElements.tabs.addEventListener('click', (e) => {
                const tab = e.target.closest('.tab');
                if (tab) {
                    switchTab(tab);
                    if (tab.dataset.tab === 'settingTab') {
                        uiElements.deadzoneValue.textContent = uiElements.deadzone.value + '%';
                        uiElements.deadzoneLeftValue.textContent = uiElements.deadzoneLeft.value + '%';
                    }
                }
            });
        }

        // 映射列表事件委托
        uiElements.mappingList?.addEventListener('click', (e) => {
            const btn = e.target.closest('.item-action-btn');
            if (!btn) return;

            const mappingId = parseInt(btn.dataset.id);
            if (btn.classList.contains('edit')) {
                const mapping = state.mappings.find(m => m.id === mappingId);
                if (mapping) openModal("编辑按键映射", mapping.composed_button, mapping.composed_shortcut_key, mapping.id);
            } else if (btn.classList.contains('delete')) {
                state.mappings = state.mappings.filter(m => m.id !== mappingId);
                if (invoke) invoke('set_mapping', {mapping: state.mappings});
                renderMappings();
            }
        });

        // 标题栏双击事件阻止
        document.getElementById("titlebar")?.addEventListener("dblclick", (e) => {
            e.preventDefault();
            e.stopPropagation();
        });

        // 物理连接状态监听
        if (appWindow && invoke) {
            appWindow.listen('physical_connect_status', (event) => {
                if (event.payload === false) {
                    console.log(`手柄 ${state.deviceSelected.name} 物理断开连接`);
                    updateStatusMessage(`手柄 ${state.deviceSelected.name} 物理断开连接!`, true);
                    closeAndQueryDevice();
                }
            });
        }
    }

    // ======================
    // 8. 初始化应用
    // ======================
    function initApp() {
        toggleIndicator(false);
        updateStatusMessage("请选择一个设备并点击连接按钮");
        loadSettings();
        updateControllerButtons();
        loadMappings();
        titlebarLogic();
        setupEventListeners();
    }

    initApp();
});