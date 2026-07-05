# ADR-001: 魔改 gilrs-core WGI UUID 计算以匹配 SDL DB GUID

**状态**：Accepted
**日期**：2026-07-03
**决策者**：Hotakus + Sisyphus（AI 辅助）

## 上下文

### 问题陈述

XialloControl 使用 gilrs 库（v0.11.0，git submodule）作为手柄输入抽象层。非 Xbox/PS4 手柄（如北通 Betop vid:0x20bc）出现按键错位。

根因定位：gilrs-core 的 Windows WGI 后端计算的 UUID 与 SDL_GameControllerDB 的 GUID 字节序不匹配，导致 `MappingDb::get(uuid)` 永远查不到 SDL DB 条目，退回 `Mapping::default`（按 `nec::BTN_SOUTH` 等常量 1:1 映射）。对走 `RawGameController` 路径的手柄，原始按钮序号不按 nec 常量排列，按键错位。

两个独立缺陷：
1. **WgiGamepad 路径**返回 `Uuid::nil()`（`gamepad.rs:554-555`），nil UUID 查 DB 查不到
2. **RawGameController 路径**用 `Uuid::from_fields`（大端序），但 SDL DB GUID 用小端序编码 bustype/vid/pid，两者不匹配

### 约束

- gilrs 是 git submodule，指向 `Hotakus/gilrs` fork（HTTPS: `https://github.com/Hotakus/gilrs.git`）
- gilrs 上游在 GitLab（`gilrs-project/gilrs`），无写权限
- 项目已深度集成 gilrs 0.11.0，切换库成本高
- gilrs issue #190/#193 已报告同源问题，作者至今未修
- gilrs 作者在源码注释中承认 UUID 计算与 SDL DB 不匹配（`gamepad.rs:561-565` 原文："In my testing though, it caused my controllers to not find mappings"）

## 决策

魔改 `gilrs-core/src/platform/windows_wgi/gamepad.rs` 的 UUID 计算逻辑（commit `9571795`）：

1. 移除 `match wgi_gamepad.is_some()` 分支，统一用 VID/PID 构造 UUID（不再对 WgiGamepad 返回 nil）
2. 将 `Uuid::from_fields`（大端序）改为 `Uuid::from_fields_le`（小端序），匹配 SDL DB GUID 编码

额外修复 `gilrs/src/gamepad.rs:970` 的 `deadzone()` 方法（commit `968f153`）：`axis_or_btn_name(axis).unwrap()` 改为 `match`，None 时回退到 `stick_deadzone` 并打 warning，不再 panic。

### 选择理由

- gilrs 作者已在源码注释中承认 UUID 计算与 SDL DB 不匹配
- 修改量约 15 行（UUID 修复）+ 7 行（deadzone 修复），仅动两个函数
- 修复后 `add_mappings` 注入的 2255 行 SDL DB 条目自动按 VID/PID 匹配设备
- 实测验证：Betop Controller（vid:0x20bc, pid:0x1263）从 `映射源: Driver` 变为 `映射源: SdlMappings`

## 替代方案

### 方案 1：不魔改，走 P2 运行时 set_mapping
**优点**：不碰 submodule，与上游升级完全解耦
**缺点**：需项目侧自建 VID/PID → SDL 行索引（约 100+ 行新代码 + 配置文件解析），每个未登记手柄需手动加映射行；`add_mappings` 注入的 DB 对 WGI 后端仍然无效
**未选原因**：根因在 gilrs UUID 计算，不修复则 `add_mappings` 注入方案（P1）对 Windows WGI 后端基本无效，P2 变成唯一可行路径而非优化路径；魔改 15 行 vs P2 100+ 行

### 方案 2：切换到 sdl2 crate
**优点**：sdl2 自带完整 SDL GameController 支持，无 UUID 字节序问题
**缺点**：需 SDL2 原生库依赖（Windows 需分发 `SDL2.dll` 约 1.5MB），项目从 gilrs 迁移到 sdl2 工作量大（所有 `gamepad.is_pressed(Button::South)` 调用都要改），Tauri 打包变复杂
**未选原因**：迁移成本远超 15 行魔改，且违背项目"极简（Windows 安装包仅 4MB）"的设计目标

## 后果

### 正面
- SDL DB 里 2255 行条目自动对 WGI 后端生效
- `add_mappings` 注入方案（PR #16 的 P1）真正可用
- 修复 gilrs issue #190 同源问题

### 负面
- `src-tauri/third-party/gilrs` submodule 与上游 gilrs 脱节，需长期维护 `Hotakus/gilrs` fork
- gilrs 升级时需 rebase 两个 commit（`9571795` UUID 修复 + `968f153` deadzone 修复）
- **WGI/hidapi 可见性不一致**（v0.22.2 发现）：XInput 兼容手柄（如 GameSir G7 Pro vid:0x3537）在 WGI/gilrs 层可被检测，但对 hidapi 不可见，导致 `DeviceInfo.device_path` 为 `None`，`listen()` 轮询线程静默失效（issue #15）。v0.22.2 在 `use_device` 中以 `wgi:{vid}` sentinel 兜底。**后续功能若依赖 hidapi 设备路径需注意此约束。**

### 风险与缓解
- **风险**：gilrs 上游修复 UUID 后，fork 的 patch 产生冲突
  **概率**：低（issue #190 自 2025 年开放至今未修）
  **缓解**：上游修复后可切回上游 submodule，移除 fork patch，用新 ADR 取代本 ADR

## 验证

### 如何验证
- `cargo check` 编译通过（无新增 warning）
- 连接 Betop Controller 后日志显示 `映射源: SdlMappings`（而非 `Driver`）
- 不再 panic（`deadzone()` 的 None 回退生效）

### 违反时的后果
- 若回退魔改，`add_mappings` 注入的 SDL DB 对 Windows WGI 后端失效，非 Xbox/PS4 手柄按键错位问题回归

## 相关决策

### 手柄支持路线图（P0-P3）

本 ADR 是 P1 方案能生效的前置条件。完整路线图：

| 优先级 | 方案 | 状态 | 描述 |
|--------|------|------|------|
| **P0** | 扩展 VID 识别表 | ✅ PR #16 已实现 | `detect_controller_type` 从 4 个 VID 扩展到 15 个，覆盖 Razer/8BitDo/Logitech/HORI/Thrustmaster/Mad Catz/Valve/Flydigi/GameSir/Nacon/Mayflash + BETOP 多 VID；XInput 兼容手柄统一归 `Xbox` 类型复用 layout |
| **P1** | GilrsBuilder 注入 SDL DB | ✅ PR #16 已实现 | `Gilrs::new()` 改为 `GilrsBuilder::new().add_mappings(include_str!("gamecontrollerdb_ext.txt")).build()`，注入 mdqinc/SDL_GameControllerDB 完整数据库（2255 行，2026-06-10 更新） |
| **P1** | gilrs-core UUID 修复 | ✅ PR #16 已实现（本 ADR） | 魔改 WGI UUID 计算，让 SDL DB 查找生效 |
| **P1** | mapping_source 诊断 | ✅ PR #16 已实现 | `list_controllers_from_gilrs` 添加 `mapping_source()` 日志 |
| **P2** | 运行时 set_mapping + 配置 | 待开工 | `Gamepad::set_mapping()` + `button_mappings.toml` 配置文件，按 VID/PID 查 SDL 行注入。用于 SDL DB 未收录手柄的兜底。需项目侧自建 VID/PID → SDL 行索引 + SDL 行解析器（`parse_sdl_mapping` 是 private 的，需自己写） |
| **P3** | 按键学习功能 | 待开工 | 用户按物理键→记录 EvCode→构造 `MappingData::insert_btn`→`set_mapping`。复用 `JoystickCaliModal` 交互范式。任何手柄都能支持，是终极兜底方案 |

### 相关链接
- gilrs issue #190（同源问题）: https://gitlab.com/gilrs-project/gilrs/-/issues/190
- gilrs issue #193（macOS USAGE 常量问题）: https://gitlab.com/gilrs-project/gilrs/-/issues/193
- SDL_GameControllerDB 上游: https://github.com/mdqinc/SDL_GameControllerDB
- gilrs fork: https://github.com/Hotakus/gilrs
- PR #16: https://github.com/Hotakus/XialloControl/pull/16
- issue #15 (G7 Pro 无效果): https://github.com/Hotakus/XialloControl/issues/15
- v0.22.2 (device_path sentinel): commit `6a6ba18`

### 其他决策（本会话内，未单独建 ADR）
- **BETOP 不归 Xbox 类型**：用户明确要求，因杂牌北通可能无 UUID，归 Xbox 会走 rusty-xinput 但杂牌不一定真支持 XInput API。BETOP 保留 `ControllerType::Betop`，走 gilrs SDL 映射路径。
- **poll_controller 改用 controller_type 而非 uuid_is_invalid**：魔改后 WgiGamepad UUID 不再是 nil，`uuid_is_invalid` 判断失效。Xbox 分支改为始终走 `xbox::poll_xbox_controller`（rusty-xinput），`_` 兜底分支始终走 `poll_other_controllers`（gilrs）。
- **add_mappings 注入完整 DB 而非只国产条目**：gilrs 的 `MappingDb::insert` 会自动按平台过滤，完整 DB（585KB）嵌入可接受，且覆盖 gilrs 内置旧版。
- **v0.22.2 device_path sentinel fallback**：XInput 兼容手柄对 hidapi 不可见时，`use_device` 以 `wgi:{vid}` 作为 `device_path` 兜底值，避免 `listen()` 轮询线程静默失效。详见 ADR-001 负面后果第 3 条。
- **v0.22.5 XInput→gilrs 轮询回退**：ADR-001 决策"Xbox 分支始终走 `xbox::poll_xbox_controller`"在单手柄场景下正确，但在多手柄同插（如官方 Xbox 手柄占 XInput slot 0 + GameSir G7 Pro 仅 WGI 可见）时失效——`poll_xbox_controller` 匹配失败即 `physical_disconnect_device`，即使设备在 gilrs 中活跃。v0.22.5 改为 `poll_xbox_controller` 返回匹配结果，`poll_controller` 在 XInput 失败时回退到 `poll_other_controllers`（gilrs/WGI 路径）。同时消除 `poll_xbox_controller` 中 4 处 `.unwrap()` panic（`sub_product_id` 等对 hidapi 不可见设备为 `None`）。commit `abb99f3`。
