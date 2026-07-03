# 虚拟键盘功能实现计划

## 当前状态分析

### 前端部分
- 在MappingModal.vue中已经添加了"打开虚拟键盘"的开关控件
- 在global_states.ts中有`asOpenVirtualKeyboard`状态来管理这个功能
- 在MappingModal.vue中有`updateVirtualKeyboardDisplay`函数来处理虚拟键盘开关状态变化
- 在RightPanel.ts中的`updateKeyDisplay`函数中，已经有了一个判断：如果`state.asOpenVirtualKeyboard`为true，则不执行`updateKey()`函数

### 后端部分
- 目前还没有对虚拟键盘功能的支持
- 在mapping.rs中的PrimaryAction枚举中没有虚拟键盘类型
- 在parse_composed_key_to_action函数中没有对"VirtualKeyboard"的解析

## 需要实现的修改

### 1. 前端MappingModal.ts中的修改
在`mappingsConfirm`函数中，需要添加对虚拟键盘的处理逻辑：
- 当`state.asOpenVirtualKeyboard`为true时，将`composed_shortcut_key`设置为"VirtualKeyboard"
- 确保虚拟键盘的映射能够正确提交到后端

### 2. 后端mapping.rs中的修改

#### 2.1 在PrimaryAction枚举中添加虚拟键盘类型
```rust
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case", untagged)]
pub enum PrimaryAction {
    /// 按下一个键盘按键。
    KeyPress {
        #[serde(flatten)]
        key: enigo::Key,
    },
    /// 点击一个鼠标按钮。
    MouseClick { button: enigo::Button },
    /// 滚动鼠标滚轮。
    MouseWheel { amount: i32 },
    /// 打开虚拟键盘。
    VirtualKeyboard {
        /// 标记字段，用于序列化和反序列化
        #[serde(default, skip_serializing_if = "Option::is_none")]
        virtual_keyboard: Option<bool>,
    },
    /// 空操作，不执行任何动作。
    None {
        /// 标记字段，用于序列化和反序列化
        #[serde(default, skip_serializing_if = "Option::is_none")]
        none: Option<bool>,
    },
}
```

#### 2.2 在parse_composed_key_to_action函数中添加对"VirtualKeyboard"的解析
```rust
// 主操作 - 虚拟键盘
"virtualkeyboard" => set_primary(
    &mut primary_action,
    PrimaryAction::VirtualKeyboard { virtual_keyboard: true },
)?,
```

#### 2.3 在Executable trait实现中添加虚拟键盘的执行逻辑
```rust
PrimaryAction::VirtualKeyboard { .. } => {
    // 调用系统API打开虚拟键盘
    // Windows: 使用osk.exe
    // macOS: 使用KeyboardViewer
    // Linux: 使用onboard或类似工具
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("osk.exe").spawn().unwrap_or_else(|e| {
            log::error!("Failed to open virtual keyboard: {}", e);
        });
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open").arg("-b").arg("com.apple.KeyboardViewer").spawn().unwrap_or_else(|e| {
            log::error!("Failed to open virtual keyboard: {}", e);
        });
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("onboard").spawn().unwrap_or_else(|e| {
            log::error!("Failed to open virtual keyboard: {}", e);
        });
    }
}
```

## 实现步骤

1. 修改MappingModal.ts中的`mappingsConfirm`函数，添加对虚拟键盘的处理逻辑
2. 修改mapping.rs中的PrimaryAction枚举，添加虚拟键盘类型
3. 修改mapping.rs中的parse_composed_key_to_action函数，添加对"VirtualKeyboard"的解析
4. 修改mapping.rs中的Executable trait实现，添加虚拟键盘的执行逻辑
5. 测试虚拟键盘功能是否正常工作
6. 提交代码并生成中文提交信息