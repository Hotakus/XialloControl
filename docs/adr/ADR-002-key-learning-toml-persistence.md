# ADR-002: P2+P3 合并设计——按键学习 + TOML 本地配置持久化

**状态**：Accepted
**日期**：2026-07-03
**决策者**：Hotakus + Sisyphus（AI 辅助）

## 上下文

### 问题陈述

ADR-001 的 P1（SDL DB 注入 + gilrs UUID 魔改）已实现并合并（PR #16）。对 SDL GameController DB 收录的手柄（2255 条目），按键映射自动生效。但 SDL DB 未收录的手柄（国产杂牌、冷门型号）仍退回 `Mapping::default`，按键错位。

ADR-001 路线图原定后续两步：
- **P2**：运行时 `set_mapping` + `button_mappings.toml` 配置文件（要求用户提供 SDL 行字符串）
- **P3**：按键学习功能（用户按物理键 → 自动生成映射）

原计划 P2、P3 分为独立 PR 交付。

### 约束

- gilrs `parse_sdl_mapping`（`gilrs/src/mapping/mod.rs:268`）是 private，P2 如用 SDL 行格式需自写解析器
- gilrs `MappingData` API 均为 public（`gilrs/src/mapping/mod.rs:528-571`）：
  - `insert_btn(from: ev::Code, to: Button) -> Option<EvCode>`
  - `insert_axis(from: ev::Code, to: Axis) -> Option<EvCode>`
  - `button(idx: Button) -> Option<ev::Code>`
  - `axis(idx: Axis) -> Option<ev::Code>`
- `Gamepad::set_mapping(gamepad_id: usize, mapping: &MappingData, name: Option<&str>)` 为 public
- `MappingError` 变体：`InvalidCode` / `DuplicatedEntry` / `UnknownElement` / `NotSdl2Compatible` 等（`mod.rs:576-591`）
- 项目设计目标：极简（Windows 安装包 ~4MB），Tauri v2 + Vue 3 桌面应用
- 现有 UI 范式：`JoystickCaliModal`（摇杆校准模态框，连续输入交互）

## 决策

### 1. P2+P3 合并为一个 PR

P2（配置持久化）是 P3（按键学习）的存储层，P3 是 P2 的输入方式。拆开后各自无法独立交付价值（详见替代方案 1）。

### 2. 配置文件用 TOML 格式，不用 SDL 行格式

```toml
[controllers."20bc:1263"]
name = "Betop Controller"

[controllers."20bc:1263".buttons]
south = 0    # EvCode → gilrs::Button::South
east = 1
back = 8
start = 9
# ...共 ~15 个按键

[controllers."20bc:1263".axes]
# 本 PR 不实现摇杆学习，预留 schema 前向兼容
left_x = 0   # EvCode → gilrs::Axis::LeftX
left_y = 1
```

### 3. 本 PR 只做按键 + D-Pad 学习，摇杆学习后续 PR

按键 + D-Pad 是离散事件（按下即触发），学习逻辑简单。摇杆是连续值，需采样逻辑 + 阈值调优（详见替代方案 2）。

### 4. 新建 KeyLearningModal，不复用 JoystickCaliModal

交互范式不同（离散 vs 连续），强行复用会导致职责混淆（详见替代方案 3）。

### 选择理由

**P2+P3 合并**：P2 单独交付要求用户手写 SDL 行字符串（`a:b1,b:b2,dpdown:h0.4,...`），普通用户无法使用；P3 单独交付学习结果无持久化，每次重连都要重新学习。合并后闭环：学习 → 保存 toml → 下次自动加载。

**TOML 格式**：
- 进入 P2/P3 路径的前提是 SDL DB 无匹配——用户手柄不在 DB 里，SDL 行格式的"可从社区复制"优势不成立
- SDL 行需自写解析器 ~80 行（`parse_sdl_mapping` private），TOML 用 serde 反序列化 ~10 行
- `MappingData::insert_btn(Code, Button)` / `insert_axis(Code, Axis)` 直接对应 TOML 字段，无需中间转换
- TOML 可读性：`south = 1` vs `a:b1`，用户手动编辑不会被 hat 语法（`h0.4`）劝退

**先按键 + D-Pad**：
- 按键学习：离散事件，用户按一下 → 记录 EvCode → 完成，~15 个按键
- 摇杆学习：连续值，需采样 2s 取 |Δ| 最大 axis + 阈值过滤静止漂移，6 个 axis（LX/LY/RX/RY/L2/R2）
- 杂牌手柄最常见痛点是 ABXY/D-Pad 错位，摇杆通常按标准 X/Y 上报，错位概率低

**新建 KeyLearningModal**：
- `JoystickCaliModal`：连续输入（推摇杆到极限），1-2 步完成
- `KeyLearningModal`：离散输入（按键事件），~15 个按键逐个映射
- UI 布局不同：校准要可视化摇杆位置/死区范围，学习要列待映射按键 + 当前按下反馈

## 替代方案

### 方案 1：P2 用 SDL 行格式，P2/P3 分开 PR

**优点**：可从社区资源（Steam 社区、SDL wiki）复制 SDL 行
**缺点**：
- `parse_sdl_mapping` private，需自写解析器 ~80 行
- hat 语法 `h0.4` 解析复杂
- 用户需理解平台前缀（`03000000` = Windows）
- P2 单独交付时用户需手写 SDL 行，普通用户无法使用
- 进入条件是 SDL DB 无匹配，用户手柄不在 DB 里，从哪复制 SDL 行？
**未选原因**：SDL 行格式的兼容性优势在"SDL DB 无匹配"场景下不成立；P2/P3 拆开各自无法独立交付

### 方案 2：一次做全（含摇杆学习）

**优点**：一步到位，无后续 PR
**缺点**：
- 摇杆学习需采样逻辑（~50 行）+ 阈值调优 + 交互设计（6 轮推动确认）
- 测试成本高：需在多种手柄上反复调阈值区分"有意推动"和"静止漂移"
- 延迟按键 + D-Pad 的交付
**未选原因**：摇杆错位概率低（通常按标准 X/Y 上报），先交付按键 + D-Pad 性价比更高

### 方案 3：复用 JoystickCaliModal

**优点**：省一个组件的脚手架代码（~20 行）
**缺点**：
- 连续输入 vs 离散输入，交互范式冲突
- UI 布局需求不同（校准可视化摇杆位置，学习列待映射按键列表）
- 合并后代码职责不清，后续维护成本高
**未选原因**：复用节省的代码量远不抵交互冲突带来的复杂度

## 后果

### 正面

- SDL DB 未收录的手柄有兜底方案（学习 → 持久化 → 自动加载）
- TOML 格式可读性好，用户可手动编辑
- 不依赖 `parse_sdl_mapping`，与 gilrs 内部 API 解耦
- TOML schema 预留 axes 段，后续摇杆学习 PR 无需 schema 迁移

### 负面

- TOML 格式与 SDL DB 行格式不互通，无法直接导入社区 SDL 行（可在 UI 留"导入 SDL 行"高级选项作为未来扩展）
- 摇杆学习不在本 PR，未收录手柄的摇杆仍可能错位（概率低）
- 新增 `button_mappings.toml` 配置文件，需管理生命周期（版本迁移、损坏处理）

### 风险与缓解

- **风险**：TOML schema 后续需扩展（摇杆学习加 axes 段）
  **概率**：高（已计划后续 PR）
  **缓解**：TOML schema 初始就预留 axes 段（本 PR buttons 段实现，axes 段空壳），避免后续迁移

- **风险**：`button_mappings.toml` 损坏导致加载失败
  **概率**：低（用户手动编辑可能写错）
  **缓解**：加载时 serde 错误不 panic，打 warning 跳过该条目，弹 UI 提示用户

- **风险**：`set_mapping` 返回 `MappingError`（如 `DuplicatedEntry`、`InvalidCode`）
  **概率**：中（学习时用户可能误操作）
  **缓解**：学习 UI 在 `set_mapping` 调用前预校验 EvCode 唯一性，调用后检查返回值并提示

## 验证

### 如何验证

- SDL DB 未收录手柄连接 → 日志显示 `映射源: Driver`（非 SdlMappings）→ 弹出 KeyLearningModal
- 逐键学习 → `set_mapping` 注入 → 按键正确响应（ABXY/Shoulders/D-Pad/Start/Back）
- `button_mappings.toml` 生成且格式正确（serde 反序列化通过）
- 重启应用 → 手柄连接 → 自动加载 toml → 不弹 UI → 日志显示已加载本地映射 → 按键正确
- 手动编辑 toml 写错 → 加载时 warning 不 panic → UI 提示

### 违反时的后果

- 若回退到 SDL 行格式，需自写 ~80 行解析器，且用户无法手动编辑
- 若 P2/P3 拆开，P2 单独交付用户无法使用（需手写 SDL 行）
- 若复用 JoystickCaliModal，模态框代码职责混淆，后续维护困难

## 相关决策

- **依赖**：ADR-001（gilrs UUID 魔改，P1 的前置条件。本 ADR 的 P2/P3 是 P1 的兜底补充）
- **修订**：ADR-001 路线图表中 P2/P3 从"分开做"改为"合并做"，P2 描述从"SDL 行格式"改为"TOML 格式"
