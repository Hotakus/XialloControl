# XialloControl 项目结构分析 (AI 提示优化版)

## 1. 核心技术栈

- **框架**: Tauri (使用 Rust 构建的桌面应用框架)
- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust

## 2. 如何运行项目

1.  安装依赖: `pnpm install`
2.  启动开发环境: `pnpm tauri dev`
    -   该命令会自动执行 `pnpm dev` (`vite`) 启动前端服务，并启动 Tauri 应用外壳加载前端页面。
3.  构建应用: `pnpm tauri build`

## 3. 项目结构概览

这是一个典型的 Tauri 项目，代码分为前端 (`src`) 和后端 (`src-tauri`) 两部分。

### 3.1. 前端 (路径: `src`)

- **入口**: [`src/main.ts`](./src/main.ts) 创建 Vue 实例并挂载根组件 [`App.vue`](./src/App.vue)。
- **核心目录**:
    - `src/vue`: 存放所有 `.vue` 单文件组件，是 UI 的核心。
    - `src/ts`: 存放主要的 TypeScript 逻辑，用于分离视图和逻辑。每个 `.ts` 文件可能对应一个或多个 `.vue` 组件的逻辑。
    - `src/assets`: 存放静态资源，如 CSS、SVG 图像等。
- **与后端通信**: 前端通过 `@tauri-apps/api` 的 `invoke` 函数调用在后端 Rust 中注册的 `#[tauri::command]`。

### 3.2. 后端 (路径: `src-tauri`)

- **入口**: [`src-tauri/src/main.rs`](./src-tauri/src/main.rs) 是启动器，它只调用了 `xiallocontrol_lib::run()`。
- **核心逻辑**: **[`src-tauri/src/lib.rs`](./src-tauri/src/lib.rs) 是整个后端的神经中枢**。
    - **`run()` 函数**: 使用 `tauri::Builder` 构建应用，集成插件 (日志、更新、自启等)，设置 `setup` 钩子和 `invoke_handler`。
    - **`invoke_handler`**: 这是 **关键中的关键**。它通过 `tauri::generate_handler![]` 宏注册了所有可供前端调用的 Rust 函数 (`command`)。所有后端功能都通过这里暴露给前端。
    - **`setup` 钩子**: 负责应用启动时的初始化工作，包括动态创建主窗口、初始化系统托盘和手柄控制器。
- **模块化**: 后端逻辑按功能划分在 `src-tauri/src` 下的不同模块中：
    - `src-tauri/src/controller`: 游戏手柄的核心逻辑，包括设备发现、数据读取、校准等。**非常重要**。
    - `src-tauri/src/mapping`: 按键映射管理。
    - `src-tauri/src/preset`: 预设方案管理。
    - `src-tauri/src/setting`: 应用设置管理。
    - `src-tauri/src/tray`: 系统托盘图标和菜单。
    - `src-tauri/src/xeno_utils`: 通用工具函数。
- **配置文件**: [`src-tauri/tauri.conf.json5`](./src-tauri/tauri.conf.json5) 定义了应用ID、版本、构建命令和更新服务器等元数据。**注意：窗口是在 `lib.rs` 中动态创建的，而非在此静态配置**。

## 4. 核心状态管理与关键文件

- **核心状态管理模式**: 本项目 **最重要的架构特点** 是后端的 **有状态服务**。后端并非无状态的请求-响应模式，而是在内存中维护了应用的 **核心实时状态**。这些状态可能通过 `Mutex` 包裹的静态变量或 `Tauri State` 来管理。
- **核心状态包括**:
    - 当前连接的手柄设备信息及其实时输入数据。
    - 用户的全局设置 (通过 `src-tauri/src/setting` 模块管理)。
    - 按键映射规则列表 (通过 `src-tauri/src/mapping` 模块管理)。
    - 用户的预设方案 (通过 `src-tauri/src/preset` 模块管理)。
- **状态交互**: 前端通过 `invoke` 调用 `lib.rs` 中注册的各种 `command` (如 `get_current_settings`, `update_mapping`, `query_devices`) 来读取或修改这些位于 Rust 后端的内存状态。

- **关键文件**:
    - **[`src-tauri/src/lib.rs`](./src-tauri/src/lib.rs)**: **必读文件**。理解该文件中的 `invoke_handler` 列表，就能掌握后端暴露给前端的所有状态读写接口。
    - **[`src-tauri/src/controller`](./src-tauri/src/controller)**: 手柄数据处理的核心逻辑所在地，是理解数据流的关键。
    - **[`src/vue`](./src/vue) & [`src/ts`](./src/ts)**: 前端UI和逻辑的主要实现目录，展示了如何调用后端 `command` 并响应状态变化。

## 5. 功能模块详细分析

### 5.1. 控制器模块 (`src-tauri/src/controller`)

该模块是整个应用的核心，负责与游戏手柄进行交互。

#### 5.1.1. 核心数据结构

- **`DeviceInfo`** (`controller.rs`): 存储设备信息，如名称、厂商ID、产品ID、设备路径、控制器类型等。
- **`ControllerDatas`** (`datas.rs`): 存储控制器的实时数据，包括按钮状态、摇杆数据、触发器数据等。
- **`ControllerButtons`** (`datas.rs`): 枚举类型，定义了所有可能的按钮。
- **`JoystickRotation`** (`datas.rs`): 枚举类型，定义了摇杆旋转状态（无旋转、顺时针、逆时针）。
- **`StickCalibration`** (`calibrate.rs`): 存储摇杆校准数据，包括校准步骤、校准模式、摇杆中心点、摇杆范围等。
- **`ControllerCalibration`** (`calibrate.rs`): 存储左右摇杆的校准数据。

#### 5.1.2. 全局静态变量

- **`CURRENT_DEVICE`** (`controller.rs`): 存储当前连接的设备信息。
- **`CONTROLLER_DATA`** (`controller.rs`): 存储当前控制器的实时数据。
- **`RAW_CONTROLLER_DATA`** (`controller.rs`): 存储原始的控制器数据，用于校准。
- **`JOYSTICK_ROTATION_STATES`** (`controller.rs`): 存储摇杆旋转的物理状态。
- **`FREQ`** (`controller.rs`): 存储轮询频率。
- **`TIME_INTERVAL`** (`controller.rs`): 存储轮询时间间隔。
- **`CONTROLLER_CALIBRATION`** (`calibrate.rs`): 存储当前控制器的校准数据。

#### 5.1.3. 重要函数

- **`initialize`** (`controller.rs`): 初始化控制器模块，启动 Gilrs 事件监听线程、设备监听线程等。
- **`listen`** (`controller.rs`): 主设备状态监听循环，负责轮询设备状态、处理预设切换、执行映射等。
- **`gilrs_listen`** (`controller.rs`): 初始化 Gilrs 事件监听线程。
- **`poll_controller`** (`controller.rs`): 根据控制器类型分发轮询任务。
- **`poll_other_controllers`** (`controller.rs`): 轮询非Xbox控制器状态。
- **`handle_preset_switching_decision`** (`controller.rs`): 处理预设切换决策。
- **`detect_controller_type`** (`controller.rs`): 根据厂商ID识别控制器类型。
- **`use_device`** (`controller.rs`): 选择使用指定设备。
- **`disconnect_device`** (`controller.rs`): 断开当前设备连接。
- **`set_frequency`** (`controller.rs`): 设置轮询频率。
- **`get_controller_data`** (`controller.rs`): 获取控制器数据。
- **`query_devices`** (`controller.rs`): 查询可用设备。
- **`try_auto_connect_last_device`** (`controller.rs`): 尝试自动连接上次连接的设备。
- **`update_joystick_rotation_state`** (`controller.rs`): 计算并更新单个摇杆的旋转状态。
- **`get_calibrated_stick_values`** (`controller.rs`): 获取校准后的摇杆值。
- **`load_or_create_config`** (`controller.rs`): 加载或创建设备配置文件。
- **`list_supported_connected_devices`** (`controller.rs`): 检测当前连接的设备并匹配支持列表。
- **`initialize`** (`calibrate.rs`): 初始化校准模块。
- **`start_stick_calibration`** (`calibrate.rs`): 开始摇杆校准。
- **`next_stick_calibration_step`** (`calibrate.rs`): 进入下一步摇杆校准。
- **`cancel_stick_calibration`** (`calibrate.rs`): 取消摇杆校准。
- **`save_current_calibration`** (`calibrate.rs`): 保存当前校准数据。
- **`reset_calibration_to_default`** (`calibrate.rs`): 重置校准为默认值。
- **`get_calibration_state`** (`calibrate.rs`): 获取校准状态。
- **`set_calibration_mode`** (`calibrate.rs`): 设置校准模式。
- **`apply_calibration`** (`calibrate.rs`): 应用校准数据到原始摇杆值。
- **`load_calibration`** (`calibrate.rs`): 加载指定设备的校准数据。
- **`reset_calibration`** (`calibrate.rs`): 重置校准数据。
- **`get_current_calibration`** (`calibrate.rs`): 获取当前校准数据。

### 5.2. 映射模块 (`src-tauri/src/mapping.rs`)

该模块负责将手柄输入映射到键盘或鼠标操作。

#### 5.2.1. 核心数据结构

- **`Mapping`**: 存储一个映射规则，包括手柄按钮组合、目标键盘/鼠标操作、触发状态等。
- **`Action`**: 存储一个完整的操作指令，包括修饰键和主要操作。
- **`PrimaryAction`**: 枚举类型，定义了主要操作类型（按键按下、鼠标点击、鼠标滚轮）。
- **`TriggerState`**: 存储触发状态，用于控制按键的重复触发和加速。

#### 5.2.2. 全局静态变量

- **`GLOBAL_MAPPING_CACHE`**: 存储全局映射配置缓存。
- **`SUB_MAPPING_CACHE`**: 存储副预设映射配置缓存。
- **`CONTROLLER_LAYOUT_MAP`**: 存储不同类型手柄的按键布局映射。
- **`DYNAMIC_TRIGGER_STATES`**: 存储每个映射的动态触发状态。
- **`ENIGO_SENDER`**: Enigo 工作线程的发送器。

#### 5.2.3. 重要函数

- **`initialize`** (`mapping.rs`): 初始化映射模块，加载映射配置。
- **`map`** (`mapping.rs`): 核心映射函数，将手柄输入映射到相应的操作。
- **`load_mappings`** (`mapping.rs`): 加载映射配置到全局缓存。
- **`save_mappings`** (`mapping.rs`): 保存全局映射缓存到文件。
- **`add_mapping`** (`mapping.rs`): 添加一个新的映射配置。
- **`update_mapping`** (`mapping.rs`): 更新一个已存在的映射配置。
- **`delete_mapping`** (`mapping.rs`): 删除一个映射配置。
- **`get_mappings`** (`mapping.rs`): 获取当前所有映射配置。
- **`get_mapping_by_id`** (`mapping.rs`): 根据 ID 获取单个映射配置。
- **`update_mappings_order`** (`mapping.rs`): 更新映射的顺序。
- **`refresh_mappings`** (`mapping.rs`): 刷新映射配置。
- **`set_mapping`** (`mapping.rs`): 设置所有映射配置。
- **`get_current_controller_layout_map`** (`mapping.rs`): 获取当前连接手柄的按键布局映射。
- **`parse_composed_key_to_action`** (`mapping.rs`): 解析按键组合字符串，生成结构化的 `Action`。
- **`enigo_worker`** (`mapping.rs`): Enigo 工作线程，接收命令并执行实际的键盘/鼠标操作。

### 5.3. 预设模块 (`src-tauri/src/preset.rs`)

该模块管理不同的配置预设。

#### 5.3.1. 核心数据结构

- **`Preset`**: 存储一个预设方案，包括预设名称、映射文件名、死区设置、副预设配置等。
- **`PresetItems`**: 存储预设的具体配置项。

#### 5.3.2. 全局静态变量

- **`CURRENT_PRESET`**: 存储当前预设。
- **`CURRENT_PRESET_LIST`**: 存储预设列表。
- **`CURRENT_SUB_PRESET`**: 存储当前副预设。

#### 5.3.3. 重要函数

- **`initialize`** (`preset.rs`): 初始化预设模块，加载预设列表。
- **`switch_to_preset`** (`preset.rs`): 切换到指定预设。
- **`load_preset`** (`preset.rs`): 加载指定预设。
- **`create_preset`** (`preset.rs`): 创建新的预设。
- **`delete_preset`** (`preset.rs`): 删除预设。
- **`rename_preset`** (`preset.rs`): 重命名预设。
- **`check_presets_list`** (`preset.rs`): 获取所有预设名称列表。
- **`update_deadzone`** (`preset.rs`): 更新死区设置。
- **`update_preset_items`** (`preset.rs`): 更新预设配置项。
- **`get_current_preset`** (`preset.rs`): 获取当前预设。
- **`load_presets_from_list_to_global`** (`preset.rs`): 加载预设列表到全局。

### 5.4. 设置模块 (`src-tauri/src/setting.rs`)

该模块管理应用的全局设置。

#### 5.4.1. 核心数据结构

- **`AppSettings`**: 存储应用设置，包括开机自启、最小化到托盘、记住上次连接状态、轮询频率、主题、校准模式等。

#### 5.4.2. 全局静态变量

- **`GLOBAL_SETTINGS`**: 存储全局应用设置。

#### 5.4.3. 重要函数

- **`initialize`** (`setting.rs`): 初始化设置模块，加载应用设置。
- **`get_setting`** (`setting.rs`): 获取当前设置。
- **`update_settings`** (`setting.rs`): 更新应用设置。
- **`get_current_settings`** (`setting.rs`): 获取当前应用设置。
- **`load_settings`** (`setting.rs`): 加载应用设置到全局。
- **`save_settings`** (`setting.rs`): 保存全局应用设置到文件。

### 5.5. 系统托盘模块 (`src-tauri/src/tray.rs`)

该模块管理系统托盘图标和菜单。

#### 5.5.1. 重要函数

- **`initialize`** (`tray.rs`): 初始化系统托盘模块，创建托盘图标和菜单。

### 5.6. 工具模块 (`src-tauri/src/xeno_utils.rs`)

该模块提供一些通用的工具函数。

#### 5.6.1. 重要函数

- **`initialize`** (`xeno_utils.rs`): 初始化工具模块，创建配置目录。
- **`get_app_root`** (`xeno_utils.rs`): 获取应用根目录。
- **`create_config_dir`** (`xeno_utils.rs`): 创建配置目录。
- **`get_config_path`** (`xeno_utils.rs`): 获取配置文件路径。
- **`read_toml_file`** (`xeno_utils.rs`): 读取 TOML 配置文件。
- **`write_toml_file`** (`xeno_utils.rs`): 写入 TOML 配置文件。
- **`ensure_config_dir`** (`xeno_utils.rs`): 确保配置目录存在。
- **`ensure_dir`** (`xeno_utils.rs`): 确保目录存在。

## 6. 前端核心状态与函数

### 6.1. 核心状态 (`src/ts/global_states.ts`)

- **`state`**: 存储应用的所有响应式状态，包括设备信息、设置、映射、预设、UI 状态等。

### 6.2. 重要函数

- **`initUIElements`** (`src/ts/global_states.ts`): 初始化 UI 元素。
- **`scanDevices`** (`src/ts/LeftPanel.ts`): 扫描设备。
- **`toggleDeviceConnection`** (`src/ts/LeftPanel.ts`): 切换设备连接状态。
- **`onDeviceSelected`** (`src/ts/LeftPanel.ts`): 设备选择事件处理。
- **`switchPreset`** (`src/ts/RightPanel.ts`): 切换预设。
- **`addButtonMap`** (`src/ts/RightPanel.ts`): 添加按键映射。
- **`editButtonMap`** (`src/ts/RightPanel.ts`): 编辑按键映射。
- **`deleteButtonMap`** (`src/ts/RightPanel.ts`): 删除按键映射。
- **`updateSettings`** (`src/ts/RightPanel.ts`): 更新设置。
- **`saveDeadzoneSettings`** (`src/ts/RightPanel.ts`): 保存死区设置。
- **`setPollingFrequency`** (`src/ts/RightPanel.ts`): 设置轮询频率。
- **`changeTheme`** (`src/ts/RightPanel.ts`): 切换主题。
- **`switchTab`** (`src/ts/RightPanel.ts`): 切换标签页。
- **`openCaliModal`** (`src/ts/JoystickCaliModal.ts`): 打开摇杆校准模态窗口。
- **`handleRenamePreset`** (`src/ts/PresetEditModal.ts`): 处理预设重命名。
- **`initializeSubPresetOptions`** (`src/ts/PresetEditModal.ts`): 初始化副预设选项。

## 7. 前端组件与逻辑详细分析

### 7.1. 主要组件

#### 7.1.1. LeftPanel.vue
- **功能**: 左侧面板，包含设备选择、连接按钮、状态显示、控制器图像。
- **重要元素**:
  - `#device`: 设备选择下拉框。
  - `#connect-button`: 设备连接按钮。
  - `#scan-button`: 扫描设备按钮。
  - `#status-message`: 状态消息显示区域。
  - `#open-joystick-cali-modal`: 打开摇杆校准模态窗口的按钮。
- **重要函数**:
  - `scanDevices` (`src/ts/LeftPanel.ts`): 扫描设备。
  - `toggleDeviceConnection` (`src/ts/LeftPanel.ts`): 切换设备连接状态。
  - `onDeviceSelected` (`src/ts/LeftPanel.ts`): 设备选择事件处理。

#### 7.1.2. RightPanel.vue
- **功能**: 右侧面板，包含预设管理、按键映射、摇杆设置、软件设置等。
- **重要元素**:
  - `#preset`: 预设选择下拉框。
  - `#create-preset`: 创建预设按钮。
  - `#edit-preset`: 编辑预设按钮。
  - `#delete-preset`: 删除预设按钮。
  - `#add-button-map`: 添加按键映射按钮。
  - `#deadzone`: 右摇杆死区设置滑块。
  - `#deadzone-left`: 左摇杆死区设置滑块。
  - `#auto-start`: 开机自启动设置。
  - `#minimize-to-tray`: 最小化到托盘设置。
  - `#polling-frequency`: 轮询频率设置。
  - `#theme`: 界面主题设置。
- **重要函数**:
  - `switchPreset` (`src/ts/RightPanel.ts`): 切换预设。
  - `newPreset` (`src/ts/RightPanel.ts`): 新建预设。
  - `editPreset` (`src/ts/RightPanel.ts`): 编辑预设。
  - `deletePreset` (`src/ts/RightPanel.ts`): 删除预设。
  - `addButtonMap` (`src/ts/RightPanel.ts`): 添加按键映射。
  - `editButtonMap` (`src/ts/RightPanel.ts`): 编辑按键映射。
  - `deleteButtonMap` (`src/ts/RightPanel.ts`): 删除按键映射。
  - `saveDeadzoneSettings` (`src/ts/RightPanel.ts`): 保存死区设置。
  - `updateSettings` (`src/ts/RightPanel.ts`): 更新设置。
  - `setPollingFrequency` (`src/ts/RightPanel.ts`): 设置轮询频率。
  - `changeTheme` (`src/ts/RightPanel.ts`): 切换主题。
  - `switchTab` (`src/ts/RightPanel.ts`): 切换标签页。

#### 7.1.3. MappingModal.vue
- **功能**: 按键映射模态窗口，用于添加或编辑映射规则。
- **重要元素**:
  - `#controller-button`: 输入源选择下拉框。
  - `#key-detector-area`: 按键检测区域。
  - `#key-display`: 按键显示区域。
  - `#initial-interval`: 初始触发间隔输入框。
  - `#min-interval`: 最小触发间隔输入框。
  - `#acceleration`: 加速因子滑块。
  - `#mousewheel-amount`: 滚轮滚动量滑块。
- **重要函数**:
  - `detectKey` (`src/ts/MappingModal.ts`): 检测按键。
  - `mappingsConfirm` (`src/ts/MappingModal.ts`): 确认映射。
  - `closeButtonMapModal` (`src/ts/MappingModal.ts`): 关闭映射模态窗口。

#### 7.1.4. JoystickCaliModal.vue
- **功能**: 摇杆校准模态窗口，用于进行摇杆校准。
- **重要元素**:
  - `#joystick-left`: 左摇杆显示区域。
  - `#joystick-right`: 右摇杆显示区域。
  - `#handle-left`: 左摇杆手柄。
  - `#handle-right`: 右摇杆手柄。
  - `#progress-x-left`: 左摇杆X轴进度条。
  - `#progress-y-left`: 左摇杆Y轴进度条。
  - `#progress-x-right`: 右摇杆X轴进度条。
  - `#progress-y-right`: 右摇杆Y轴进度条。
- **重要函数**:
  - `openCaliModal` (`src/ts/JoystickCaliModal.ts`): 打开摇杆校准模态窗口。
  - `closeCaliModal` (`src/ts/JoystickCaliModal.ts`): 关闭摇杆校准模态窗口。
  - `startCalibration` (`src/ts/JoystickCaliModal.ts`): 开始摇杆校准。
  - `nextStep` (`src/ts/JoystickCaliModal.ts`): 进入下一步摇杆校准。
  - `cancelCalibration` (`src/ts/JoystickCaliModal.ts`): 取消摇杆校准。
  - `saveCalibration` (`src/ts/JoystickCaliModal.ts`): 保存摇杆校准数据。
  - `resetToDefault` (`src/ts/JoystickCaliModal.ts`): 重置摇杆校准为默认值。

#### 7.1.5. PresetEditModal.vue
- **功能**: 预设编辑模态窗口，用于编辑预设名称和副预设配置。
- **重要元素**:
  - 预设名称输入框。
  - 副预设选择下拉框。
  - 切换按键选择下拉框。
  - 切换模式单选按钮（按住/切换）。
- **重要函数**:
  - `handleRenamePreset` (`src/ts/PresetEditModal.ts`): 处理预设重命名。
  - `initializeSubPresetOptions` (`src/ts/PresetEditModal.ts`): 初始化副预设选项。

### 7.2. 核心逻辑文件

#### 7.2.1. LeftPanel.ts
- **功能**: 处理左侧面板的逻辑，如设备扫描、连接、状态更新。
- **重要函数**:
  - `scanDevices`: 扫描设备。
  - `toggleDeviceConnection`: 切换设备连接状态。
  - `onDeviceSelected`: 设备选择事件处理。
  - `disconnectCurrentDevice`: 断开当前设备。
  - `closeAndQueryDevice`: 断开设备并查询新设备。
  - `updateStatusMessage`: 更新状态消息。

#### 7.2.2. RightPanel.ts
- **功能**: 处理右侧面板的逻辑，如预设切换、映射管理、设置更新。
- **重要函数**:
  - `switchPreset`: 切换预设。
  - `newPreset`: 新建预设。
  - `editPreset`: 编辑预设。
  - `deletePreset`: 删除预设。
  - `addButtonMap`: 添加按键映射。
  - `editButtonMap`: 编辑按键映射。
  - `deleteButtonMap`: 删除按键映射。
  - `saveDeadzoneSettings`: 保存死区设置。
  - `updateSettings`: 更新设置。
  - `setPollingFrequency`: 设置轮询频率。
  - `changeTheme`: 切换主题。
  - `switchTab`: 切换标签页。
  - `updateControllerButtons`: 根据设备类型更新手柄按键选项。
  - `formatKeyDisplay`: 格式化按键显示。
  - `startKeyDetection`: 开始按键检测。
  - `stopKeyDetection`: 停止按键检测。

#### 7.2.3. MappingModal.ts
- **功能**: 处理按键映射模态窗口的逻辑，如按键检测、映射添加/编辑。
- **重要函数**:
  - `detectKey`: 检测按键。
  - `mappingsConfirm`: 确认映射。
  - `closeButtonMapModal`: 关闭映射模态窗口。

#### 7.2.4. JoystickCaliModal.ts
- **功能**: 处理摇杆校准模态窗口的逻辑，如校准步骤控制、校准数据显示。
- **重要函数**:
  - `openCaliModal`: 打开摇杆校准模态窗口。
  - `closeCaliModal`: 关闭摇杆校准模态窗口。
  - `startCalibration`: 开始摇杆校准。
  - `nextStep`: 进入下一步摇杆校准。
  - `cancelCalibration`: 取消摇杆校准。
  - `saveCalibration`: 保存摇杆校准数据。
  - `resetToDefault`: 重置摇杆校准为默认值。
  - `setCalibrationMode`: 设置校准模式。
  - `updateJoystickVisualsInModal`: 更新模态窗口中的摇杆视觉效果。

#### 7.2.5. PresetEditModal.ts
- **功能**: 处理预设编辑模态窗口的逻辑，如预设重命名、副预设配置。
- **重要函数**:
  - `handleRenamePreset`: 处理预设重命名。
  - `initializeSubPresetOptions`: 初始化副预设选项。
  - `initEditablePresetName`: 初始化可编辑的预设名称。