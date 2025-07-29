// 等待DOM加载完成后执行初始化
document.addEventListener('DOMContentLoaded', () => {
    // ======================
    // 1. 初始化部分
    // ======================

    // 获取Tauri API对象
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
        statusMessage: document.getElementById('status-message')
    };

    // 应用状态变量
    let hasUserSelectedDevice = false;
    let currentDevices = [];
    let device_selected = {
        name: "",
        vendor_id: "",
        product_id: "",
        device_path: "",
        controller_type: ""
    };
    let minimize_to_tray = true;
    let isConnected = false; // 设备连接状态标志

    // 连接状态图标SVG
    const icon_connected = `<svg t="1753591538385" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="19479" width="200" height="200"><path d="M728.96 269.44a207.36 207.36 0 1 0-414.08 0v133.12a239.36 239.36 0 0 1 96-93.44v-39.68a111.36 111.36 0 1 1 222.08 0v200.96a111.36 111.36 0 0 1-111.36 111.36 110.08 110.08 0 0 1-69.76-25.6v108.8a203.52 203.52 0 0 0 69.76 12.8 207.36 207.36 0 0 0 207.36-207.36z" fill="#ffffff" p-id="19480"></path><path d="M632.96 680.32v58.88a111.36 111.36 0 1 1-222.08 0V520.32a111.36 111.36 0 0 1 110.72-111.36 110.08 110.08 0 0 1 42.88 8.96 112.64 112.64 0 0 1 26.88 16.64v-108.8a204.8 204.8 0 0 0-69.76-12.8 207.36 207.36 0 0 0-206.72 207.36v219.52a207.36 207.36 0 1 0 414.08 0V588.16a238.72 238.72 0 0 1-96 92.16z" fill="#ffffff" p-id="19481"></path></svg>`;
    const icon_disconnected = `<svg t="1753595424804" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="19662" width="200" height="200"><path d="M521.6 158.08a111.36 111.36 0 0 1 111.36 111.36V480l96 96V269.44a206.72 206.72 0 0 0-394.24-87.68L411.52 256a110.72 110.72 0 0 1 110.08-97.92zM864 846.08l-135.04-135.04-96-96-222.08-222.08-96-96-135.04-134.4L112 230.4l202.88 202.24v306.56a206.72 206.72 0 0 0 394.24 87.68l87.04 87.04z m-341.76 4.48a111.36 111.36 0 0 1-111.36-111.36V528.64l221.44 221.44a110.72 110.72 0 0 1-110.72 100.48z" fill="#ffffff" p-id="19663"></path></svg>`;

    // 初始状态设置
    toggleIndicator(false);
    updateStatusMessage("请选择一个设备并点击连接按钮");

    // ======================
    // 2. 映射配置功能
    // ======================

    // 映射相关变量
    let deviceType = 'xbox'; // 默认设备类型
    let mappings = [];       // 存储按键映射
    let editingMappingId = null; // 当前编辑的映射ID

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

    // 获取映射相关的DOM元素
    const addButton = document.getElementById('add-button-map');
    const modal = document.getElementById('mapping-modal');
    const closeModal = document.getElementById('close-modal');
    const cancelBtn = document.getElementById('cancel-btn');
    const keyDetector = document.getElementById('key-detector-area');
    const keyDisplay = document.getElementById('key-display');
    const controllerButtonSelect = document.getElementById('controller-button');
    const modalTitle = document.getElementById('modal-title');
    const mappingList = document.getElementById('button-mapping-list');

    // 当前按下的键状态
    let currentKeys = {
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
        key: null
    };
    let keyListenerActive = false; // 按键监听激活状态

    // 打开模态窗口（添加新映射）
    addButton.addEventListener('click', openModalForNew);

    // 打开模态窗口（编辑现有映射）
    function openModalForEdit(mapping) {
        modalTitle.textContent = "编辑按键映射";
        controllerButtonSelect.value = mapping.composed_button;
        keyDisplay.textContent = mapping.composed_shortcut_key;
        editingMappingId = mapping.id;
        modal.classList.add('active');
    }

    // 打开模态窗口（新增映射）
    function openModalForNew() {
        // 清除错误提示
        const errorElement = document.getElementById('modal-error');
        errorElement.style.display = 'none';
        errorElement.textContent = '';

        modalTitle.textContent = "添加按键映射";
        controllerButtonSelect.value = "";
        keyDisplay.textContent = "";
        editingMappingId = null;
        modal.classList.add('active');
        updateControllerButtons(); // 更新手柄按键选项
    }

    // 关闭模态窗口
    function closeModalFunc() {
        modal.classList.remove('active');
        stopKeyDetection(true); // 完全重置
    }

    // 关闭按钮事件
    closeModal.addEventListener('click', closeModalFunc);
    cancelBtn.addEventListener('click', closeModalFunc);
    function removeAllKeyListeners() {
        window.removeEventListener('keydown', handleKeyDown);
        window.removeEventListener('keyup', handleKeyUp);
        window.removeEventListener('mousedown', handleMouseDown);
        window.removeEventListener('mouseup', handleMouseUp);
        window.removeEventListener('wheel', handleWheel);
    }

    // 开始按键检测
    function startKeyDetection() {
        if (keyListenerActive) return;

        preventNextClick = false; // 添加这行

        keyListenerActive = true;
        currentKeys = {ctrl: false, shift: false, alt: false, meta: false, key: null};

        keyDetector.classList.add('active');
        keyDetector.textContent = '请按下键盘按键、鼠标按键或滚动滚轮...';
        keyDisplay.textContent = '';

        // 添加所有事件监听
        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('keyup', handleKeyUp);
        window.addEventListener('mousedown', handleMouseDown);
        window.addEventListener('mouseup', handleMouseUp);
        window.addEventListener('wheel', handleWheel);
    }

    // 停止按键检测
    function stopKeyDetection(resetText = true) {
        if (!keyListenerActive) return;

        keyListenerActive = false;
        keyDetector.classList.remove('active');

        // 可选：是否重置提示文本
        if (resetText) {
            keyDetector.textContent = '点击此处并按下键盘按键、鼠标按键或滚动滚轮';
        }

        // 移除所有事件监听器（保留 mouseup 的移除）
        removeAllKeyListeners();
    }

    function handleMouseUp(e) {
        // 当检测到鼠标释放时停止检测
        stopMouseDetection();
    }

    // 添加 handleWheel 函数来处理滚轮事件
    function handleWheel(e) {
        e.preventDefault();
        e.stopPropagation();

        currentKeys.ctrl = e.ctrlKey;
        currentKeys.shift = e.shiftKey;
        currentKeys.alt = e.altKey;
        currentKeys.meta = e.metaKey;
        currentKeys.key = e.deltaY < 0 ? 'MouseWheelUp' : 'MouseWheelDown';

        updateKeyDisplay();
        stopKeyDetection(false);
    }

    // 处理按键按下事件
    function handleKeyDown(e) {
        e.preventDefault();

        // 更新修饰键状态
        if (e.key === 'Control' || e.key === 'Ctrl') currentKeys.ctrl = true;
        else if (e.key === 'Shift') currentKeys.shift = true;
        else if (e.key === 'Alt') currentKeys.alt = true;
        else if (e.key === 'Meta') currentKeys.meta = true;
        else currentKeys.key = e.key; // 非修饰键

        updateKeyDisplay();
    }

    // 处理按键释放事件
    function handleKeyUp(e) {
        // 当非修饰键释放时停止检测
        if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
            stopKeyDetection();
        }
    }

    // 更新按键显示
    function updateKeyDisplay() {
        let displayText = '';
        let isMouseKey = false;

        // 构建修饰键显示文本
        if (currentKeys.ctrl) displayText += 'Ctrl + ';
        if (currentKeys.shift) displayText += 'Shift + ';
        if (currentKeys.alt) displayText += 'Alt + ';
        if (currentKeys.meta) displayText += 'Cmd + ';

        // 添加主键显示
        if (currentKeys.key) {
            const key = currentKeys.key;
            displayText += keyDisplayNames[key] || key.toUpperCase();

            // 检查是否是鼠标按键
            isMouseKey = key.startsWith('Mouse');
        }

        keyDisplay.textContent = displayText;

        // 如果是鼠标按键，添加特定样式
        if (isMouseKey) {
            keyDisplay.classList.add('mouse');
        } else {
            keyDisplay.classList.remove('mouse');
        }

        keyDisplay.textContent = displayText;
    }

    let preventNextClick = false;
    // 按键检测区域点击事件
    keyDetector.addEventListener('click', (e) => {
        if (preventNextClick) {
            preventNextClick = false;
            e.stopPropagation();
            e.preventDefault();
            return;
        }
        startKeyDetection();
    });

    // 确认按钮功能（保存映射）
    document.getElementById('confirm-btn').addEventListener('click', function () {
        const composed_button = controllerButtonSelect.value;
        const composed_shortcut_key = keyDisplay.textContent;
        const errorElement = document.getElementById('modal-error');
// 在确认后停止检测
        stopKeyDetection();
        // 清除之前的错误
        errorElement.style.display = 'none';
        errorElement.textContent = '';

        // 验证输入
        if (!composed_button) {
            errorElement.textContent = '请选择手柄按键';
            errorElement.style.display = 'block';
            return;
        }

        if (!composed_shortcut_key) {
            errorElement.textContent = '请设置键盘映射按键';
            errorElement.style.display = 'block';
            return;
        }

        if (editingMappingId) {
            // 编辑现有映射
            const mapping = mappings.find(m => m.id === editingMappingId);
            if (mapping) {
                mapping.composed_button = composed_button;
                mapping.composed_shortcut_key = composed_shortcut_key;
            }
        } else {
            // 添加新映射
            mappings.push({
                id: Date.now(), // 使用时间戳作为唯一ID
                composed_button: composed_button,
                composed_shortcut_key: composed_shortcut_key,
                mapping_type: 'keyboard'
            });
        }

        // 调用后端保存映射
        if (invoke) invoke('set_mapping', {mapping: mappings});

        // 更新UI并关闭模态窗口
        renderMappings();
        closeModalFunc();
    });

    // 根据设备类型更新手柄按键选项
    function updateControllerButtons() {
        // 清空现有选项（保留第一个提示选项）
        while (controllerButtonSelect.options.length > 1) {
            controllerButtonSelect.remove(1);
        }

        // 根据设备类型添加选项
        let buttons = [];
        if (deviceType === 'xbox') {
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
        } else if (deviceType === 'ps') {
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
        } else if (deviceType === 'switchpro') {
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

        // 添加新选项
        buttons.forEach(button => {
            const option = document.createElement('option');
            option.value = button.value;
            option.textContent = button.text;
            controllerButtonSelect.appendChild(option);
        });
    }

    // 渲染映射列表
    function renderMappings() {
        if (mappings.length === 0) {
            // 空状态显示
            mappingList.innerHTML = `
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

        mappingList.innerHTML = '';

        // 渲染每个映射项
        mappings.forEach(mapping => {
            const item = document.createElement('div');
            item.className = 'button-map-item';
            item.innerHTML = `
                <div class="button-icon">${mapping.composed_button}</div>
                <div class="key-text">映射到</div>
                <div class="key-value">${mapping.composed_shortcut_key}</div>
                <div class="item-actions">
                    <button class="item-action-btn edit" data-id="${mapping.id}">
                        <svg t="1753769162786" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="3801" width="200" height="200"><path d="M869.62198 290.936185c-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 501.688143c0 40.342824-32.8205 73.163323-73.163323 73.163323L252.963339 897.142777c-40.342824 0-73.163323-32.8205-73.163323-73.163323l0-606.206592c0-40.342824 32.8205-73.163323 73.163323-73.163323l407.621744 0c17.316387 0 31.355125-14.039761 31.355125-31.355125s-14.039761-31.355125-31.355125-31.355125L252.963339 81.899288c-74.92341 0-135.873574 60.950164-135.873574 135.873574l0 606.206592c0 74.92341 60.950164 135.873574 135.873574 135.873574l512.140193 0c74.92341 0 135.873574-60.950164 135.873574-135.873574L900.977106 322.292334C900.978129 304.975946 886.938368 290.936185 869.62198 290.936185z" fill="#707070" p-id="3802"></path><path d="M535.946388 467.382826c6.01704 5.496178 13.59053 8.205892 21.143553 8.205892 8.502651 0 16.97358-3.434216 23.159466-10.201339L898.602012 116.986411c11.682064-12.779048 10.783601-32.615838-1.995447-44.297902-12.784164-11.676947-32.615838-10.783601-44.303019 2.000564L533.950941 423.084924C522.269901 435.863972 523.167341 455.700763 535.946388 467.382826z" fill="#707070" p-id="3803"></path><path d="M355.315448 594.978876l0 30.589692c0 17.316387 14.039761 31.355125 31.355125 31.355125 17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125C369.355209 563.623751 355.315448 577.663512 355.315448 594.978876z" fill="#707070" p-id="3804"></path><path d="M631.396297 656.924717c17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 30.589692C600.041172 642.884956 614.07991 656.924717 631.396297 656.924717z" fill="#707070" p-id="3805"></path><path d="M589.507258 705.233979c-14.600533-9.344832-33.978882-5.082762-43.317575 9.492188-0.122797 0.199545-13.431917 19.525706-34.101679 19.525706-20.067035 0-32.549324-18.167778-33.401738-19.443841-9.211802-14.488992-28.395724-18.877953-43.001373-9.803273-14.713097 9.140171-19.22997 28.472472-10.089799 43.180452 11.253298 18.117636 42.138726 48.77896 86.49291 48.77896 44.129056 0 75.393107-30.432103 86.911442-48.411593C608.339162 733.966371 604.088348 714.572672 589.507258 705.233979z" fill="#707070" p-id="3806"></path></svg>
                    </button>
                    <button class="item-action-btn delete" data-id="${mapping.id}">
                        <svg t="1753765954234" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" p-id="2368" width="200" height="200"><path d="M840 288H688v-56c0-40-32-72-72-72h-208C368 160 336 192 336 232V288h-152c-12.8 0-24 11.2-24 24s11.2 24 24 24h656c12.8 0 24-11.2 24-24s-11.2-24-24-24zM384 288v-56c0-12.8 11.2-24 24-24h208c12.8 0 24 11.2 24 24V288H384zM758.4 384c-12.8 0-24 11.2-24 24v363.2c0 24-19.2 44.8-44.8 44.8H332.8c-24 0-44.8-19.2-44.8-44.8V408c0-12.8-11.2-24-24-24s-24 11.2-24 24v363.2c0 51.2 41.6 92.8 92.8 92.8h358.4c51.2 0 92.8-41.6 92.8-92.8V408c-1.6-12.8-12.8-24-25.6-24z" fill="#f57070" p-id="2369"></path><path d="M444.8 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24zM627.2 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24z" fill="#f57070" p-id="2370"></path></svg>
                    </button>
                </div>
            `;
            mappingList.appendChild(item);
        });

        // 添加编辑事件
        document.querySelectorAll('.item-action-btn.edit').forEach(btn => {
            btn.addEventListener('click', function () {
                const mappingId = parseInt(this.dataset.id);
                const mapping = mappings.find(m => m.id === mappingId);
                if (mapping) openModalForEdit(mapping);
            });
        });

        // 添加删除事件
        document.querySelectorAll('.item-action-btn.delete').forEach(btn => {
            btn.addEventListener('click', function () {
                const mappingId = parseInt(this.dataset.id);
                mappings = mappings.filter(m => m.id !== mappingId);
                renderMappings();
            });
        });
    }

    // 根据设备名称确定设备类型
    function detectDeviceType(deviceName) {
        if (deviceName.includes('Xbox')) return 'xbox';
        if (deviceName.includes('PS') || deviceName.includes('PlayStation')) return 'ps';
        if (deviceName.includes('Switch')) return 'switchpro';
        return 'xbox'; // 默认类型
    }

    // 设备连接后更新映射配置
    function updateMappingsForDevice(deviceName) {
        deviceType = detectDeviceType(deviceName);
        updateControllerButtons();
        renderMappings();
    }

    // ======================
    // 3. 设置管理函数
    // ======================

    // 加载应用设置
    async function loadSettings() {
        if (!invoke) return;
        try {
            const settings = await invoke("get_current_settings");
            // 更新UI元素
            uiElements.autoStart.checked = settings.auto_start;
            uiElements.minimizeToTray.checked = settings.minimize_to_tray;
            uiElements.theme.value = settings.theme;
            uiElements.pollingFrequency.value = settings.polling_frequency;
            uiElements.deadzone.value = settings.deadzone;
            uiElements.deadzoneValue.textContent = settings.deadzone + '%';

            // 设置后端频率
            invoke("set_frequency", {freq: settings.polling_frequency});
            minimize_to_tray = settings.minimize_to_tray;

            // 应用主题设置
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
        };
        try {
            await invoke("update_settings", {newSettings});
        } catch (error) {
            console.error("保存设置失败:", error);
        }
    }

    // ======================
    // 4. 窗口控制函数
    // ======================

    // 窗口操作命令
    async function windowControl(action) {
        if (window.__TAURI__) {
            await invoke(action);
        } else {
            console.log(`${action} simulated`);
        }
    }

    // ======================
    // 5. 设备管理函数
    // ======================

    // 更新设备列表
    function updateDeviceList(devices) {
        if (devices.length === 0) hasUserSelectedDevice = false;
        if (hasUserSelectedDevice) {
            toggleIndicator(devices.length > 0);
            return;
        }
        if (JSON.stringify(currentDevices) === JSON.stringify(devices)) return;

        currentDevices = devices;
        // 清空并重建设备下拉菜单
        uiElements.deviceSelect.innerHTML = '';
        const defaultOption = document.createElement('option');
        defaultOption.textContent = '请选择设备';
        defaultOption.disabled = true;
        defaultOption.selected = true;
        defaultOption.value = "null";
        uiElements.deviceSelect.appendChild(defaultOption);

        currentDevices.forEach((device, index) => {
            const option = document.createElement('option');
            option.value = index.toString();
            option.textContent = `${index}: ${device.name}`;
            uiElements.deviceSelect.appendChild(option);
        });
    }

    // 断开当前设备
    async function disconnectCurrentDevice() {
        if (!isConnected || !device_selected) return true;

        const deviceName = device_selected.name;
        updateStatusMessage(`正在断开设备: ${deviceName}...`);

        try {
            await invoke("disconnect_device", {deviceName});
            updateStatusMessage(`设备已断开`);
            uiElements.connectButton.classList.remove('connected');
            uiElements.connectButton.innerHTML = icon_disconnected;
            toggleIndicator(false);
            isConnected = false;
            return true;
        } catch (error) {
            console.error("断开连接失败:", error);
            updateStatusMessage(`断开失败`, true);
            return false;
        }
    }

    // 断开设备并查询新设备
    async function _close_and_query_device() {
        await disconnectCurrentDevice();
        hasUserSelectedDevice = false;
        device_selected = null;
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
    function stripDevicePrefixFromString(name) {
        const index = name.indexOf(":");
        return index !== -1 ? name.slice(index + 1).trim() : name;
    }

    // ======================
    // 6. UI交互函数
    // ======================

    // 切换选项卡
    function switchTab(tabElement) {
        const targetTab = tabElement.dataset.tab;
        // 移除所有激活状态
        document.querySelectorAll('.tab').forEach(t => {
            t.classList.remove('active');
            t.setAttribute('aria-selected', 'false');
        });
        document.querySelectorAll('.tab-content').forEach(c => {
            c.classList.remove('active');
        });
        // 激活当前选项卡
        tabElement.classList.add('active');
        tabElement.setAttribute('aria-selected', 'true');
        document.getElementById(targetTab).classList.add('active');
    }

    // 更新状态指示灯
    function toggleIndicator(isOn) {
        isOn ?
            uiElements.indicator.classList.add('on') :
            uiElements.indicator.classList.remove('on');
    }

    // 处理轮询频率变化
    function handlePollingFrequencyChange() {
        const min = 1, max = 8 * 1000;
        let value = parseInt(uiElements.pollingFrequency.value);
        value = Math.min(Math.max(value, min), max);
        uiElements.pollingFrequency.value = value;

        if (invoke) invoke("set_frequency", {freq: value});
        saveSettings();
    }

    // 更新状态消息
    function updateStatusMessage(message, isError = false) {
        uiElements.statusMessage.textContent = message;
        uiElements.statusMessage.className = 'status-message';
        if (isError) uiElements.statusMessage.classList.add('error');
        else if (message.includes('成功') || message.includes('连接')) {
            uiElements.statusMessage.classList.add('success');
        }
    }

    // ======================
    // 7. 事件监听器
    // ======================


    // 新增鼠标事件处理函数
    function handleMouseDown(e) {
        e.preventDefault();
        e.stopPropagation();

        preventNextClick = true; // 添加这行

        currentKeys.ctrl = e.ctrlKey;
        currentKeys.shift = e.shiftKey;
        currentKeys.alt = e.altKey;
        currentKeys.meta = e.metaKey;

        let mouseKey = null;
        switch (e.button) {
            case 0: mouseKey = 'MouseLeft'; break;
            case 1: mouseKey = 'MouseMiddle'; break;
            case 2: mouseKey = 'MouseRight'; break;
            case 3: mouseKey = 'MouseX1'; break;
            case 4: mouseKey = 'MouseX2'; break;
            default: return;
        }

        currentKeys.key = mouseKey;
        updateKeyDisplay();

        // 停止检测但不重置文本
        stopKeyDetection(false);

        // 新增：移除鼠标释放事件监听器，阻止后续事件影响
        window.removeEventListener('mouseup', handleMouseUp);
    }


    function stopMouseDetection() {
        if (!keyListenerActive) return;

        // 只移除鼠标相关监听器
        window.removeEventListener('mousedown', handleMouseDown);
        window.removeEventListener('mouseup', handleMouseUp);

        // 保持显示区域状态
        keyDetector.classList.remove('active');
        keyListenerActive = false;
    }

    // 窗口事件监听（物理连接状态）
    if (appWindow && invoke) {
        appWindow.listen('physical_connect_status', (event) => {
            if (event.payload === false) {
                console.log(`手柄 ${device_selected.name} 物理断开连接`);
                updateStatusMessage(`手柄 ${device_selected.name} 物理断开连接!`, true);
                _close_and_query_device();
            }
        });
    }

    // 加载初始设置
    loadSettings();

    // 选项卡切换事件
    if (uiElements.tabs) {
        uiElements.tabs.addEventListener('click', (e) => {
            const tab = e.target.closest('.tab');
            if (tab) {
                switchTab(tab);
                if (tab.dataset.tab === 'settingTab') {
                    uiElements.deadzoneValue.textContent = uiElements.deadzone.value + '%';
                }
            }
        });
    }

    // 窗口控制按钮事件
    uiElements.minimizeButton?.addEventListener('click', () => appWindow?.minimize());
    uiElements.closeButton?.addEventListener('click', () => {
        minimize_to_tray ? appWindow?.hide() : appWindow?.close();
    });

    // 设备选择事件
    uiElements.deviceSelect?.addEventListener('change', async function () {
        if (this.value === "null") {
            device_selected = null;
            updateStatusMessage("请选择一个设备");
            return;
        }

        const selectedIndex = parseInt(this.value);
        if (isNaN(selectedIndex)) {
            updateStatusMessage("无效的设备选择", true);
            return;
        }

        // 如果已有连接设备，先断开
        if (isConnected && device_selected) {
            const success = await disconnectCurrentDevice();
            if (!success) {
                // 断开失败时保持原选择
                this.value = currentDevices.indexOf(device_selected).toString();
                return;
            }
        }

        // 检查索引是否有效
        if (selectedIndex >= 0 && selectedIndex < currentDevices.length) {
            device_selected = currentDevices[selectedIndex];
            hasUserSelectedDevice = true;
            updateStatusMessage(`已选择设备: ${device_selected.name}`);
        } else {
            device_selected = null;
            updateStatusMessage(`无效的设备索引: ${selectedIndex}`, true);
        }
    });

    // 扫描设备按钮事件
    uiElements.scanButton?.addEventListener('click', async () => {
        uiElements.scanButton.classList.add('scanning');
        updateStatusMessage("正在扫描设备...");
        const icon = document.querySelector("#scan-button .icon");
        icon.classList.add("spin");

        await _close_and_query_device();

        // 扫描完成后移除动画
        icon.addEventListener("animationend", () => {
            icon.classList.remove("spin");
        }, {once: true});

        updateStatusMessage(`扫描完成！发现${currentDevices.length}个可用设备`);
        uiElements.scanButton.classList.remove('scanning');
    });

    // 连接/断开按钮事件
    uiElements.connectButton?.addEventListener('click', async function () {
        if (!device_selected || uiElements.deviceSelect.value === "null") {
            updateStatusMessage("请先选择一个设备", true);
            return;
        }

        isConnected = !isConnected;
        const deviceName = stripDevicePrefixFromString(device_selected.name);
        const controllerType = device_selected.controller_type;

        if (isConnected) {
            updateStatusMessage(`正在连接设备: ${deviceName}...`);
            if (await invoke("use_device", {deviceName})) {
                updateStatusMessage(`设备 ${deviceName} 已成功连接`);
                this.classList.add('connected');
                this.innerHTML = icon_connected;
                toggleIndicator(true);
                updateMappingsForDevice(controllerType); // 更新映射配置
            } else {
                updateStatusMessage(`连接失败`, true);
                this.classList.remove('connected');
                this.innerHTML = icon_disconnected;
                toggleIndicator(false);
                isConnected = false; // 回滚状态
            }
        } else {
            updateStatusMessage(`正在断开设备: ${deviceName}...`);
            if (await invoke("disconnect_device", {deviceName})) {
                updateStatusMessage(`设备已断开`);
                this.classList.remove('connected');
                this.innerHTML = icon_disconnected;
                toggleIndicator(false);
            } else {
                updateStatusMessage(`断开失败`, true);
                this.classList.add('connected');
                this.innerHTML = icon_disconnected;
                toggleIndicator(false);
                isConnected = true; // 回滚状态
            }
        }
    });

    // 预设管理事件
    uiElements.preset?.addEventListener('change', function () {
        updateStatusMessage(`已切换到预设方案: ${this.value}`);
    });
    uiElements.savePreset?.addEventListener('click', function () {
        updateStatusMessage('预设方案已保存');
    });
    uiElements.importPreset?.addEventListener('click', function () {
        updateStatusMessage('预设方案导入成功');
    });

    // 死区设置事件
    uiElements.deadzone?.addEventListener('input', function () {
        uiElements.deadzoneValue.textContent = this.value + '%';
    });
    uiElements.deadzone?.addEventListener('mouseup', saveSettings);

    // 设置变更监听
    uiElements.pollingFrequency?.addEventListener('change', handlePollingFrequencyChange);
    uiElements.autoStart?.addEventListener('change', function () {
        saveSettings();
        updateStatusMessage(`开机自启动已${this.checked ? '启用' : '禁用'}`);
    });
    uiElements.minimizeToTray?.addEventListener('change', function () {
        minimize_to_tray = this.checked;
        saveSettings();
        updateStatusMessage(`最小化到托盘已${this.checked ? '启用' : '禁用'}`);
    });

    // GitHub链接特殊处理
    uiElements.githubLink?.addEventListener('click', async (e) => {
        if (window.__TAURI__ && invoke) {
            e.preventDefault();
            invoke("open_url", {url: uiElements.githubLink.href});
        }
    });

    // 浏览器环境模拟警告
    if (!window.__TAURI__) {
        console.warn('Running in browser mode - Tauri API not available');
    }

    // 标题栏双击事件阻止（仅限Windows）
    document.getElementById("titlebar").addEventListener("dblclick", (e) => {
        e.preventDefault();
        e.stopPropagation();
    });

    // 应用主题设置函数
    function applyTheme(theme) {
        // 移除所有主题类
        document.body.classList.remove('theme-light', 'theme-dark', 'theme-system');

        // 应用新主题
        if (theme === 'system') {
            document.body.classList.add('theme-system');
            // 添加系统主题变化监听
            const systemThemeMedia = window.matchMedia('(prefers-color-scheme: dark)');
            const handleSystemThemeChange = () => applyTheme('system');
            systemThemeMedia.addEventListener('change', handleSystemThemeChange);
        } else {
            document.body.classList.add(`theme-${theme}`);
        }

        // 更新状态消息
        const themeTextMap = {
            light: '浅色模式',
            dark: '深色模式',
            system: '跟随系统'
        };
        updateStatusMessage(`已切换到${themeTextMap[theme]}主题`);
    }

    // 主题切换事件
    uiElements.theme?.addEventListener('change', function () {
        saveSettings();
        applyTheme(this.value);
    });

    // 加载映射配置
    async function loadMappings() {
        if (!invoke) return;
        mappings = await invoke("get_mappings");
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

    // 初始化映射功能和标题栏
    updateControllerButtons();
    loadMappings();
    titlebarLogic();

});