# 双击检测逻辑重新设计方案

## 问题分析

当前的双击检测存在以下问题：
1. 快速轮询导致短时间内被判定多次点击
2. 长按时交替触发映射输出
3. 单击仍然会触发映射输出
4. 没有合理利用press和release状态变化来判断用户操作

## 设计目标

1. 基于press和release状态变化来准确判断用户的按键操作
2. 防止快速轮询导致的误判
3. 确保双击模式下只有真正的双击才会触发映射
4. 长按不应该触发双击判断

## 数据结构设计

### ButtonCheckState 结构体扩展

```rust
#[derive(Clone, Debug)]
pub struct ButtonCheckState {
    /// 上次按下的时间
    pub last_press_time: Option<Instant>,
    /// 上次释放的时间
    pub last_release_time: Option<Instant>,
    /// 当前是否处于长按触发后的状态
    pub long_press_triggered: bool,
    /// 单击事件是否已准备好触发（在等待双击可能性时）
    pub single_press_pending: bool,
    /// 双击是否已经触发
    pub double_press_triggered: bool,
    /// 记录完整的按下次数
    pub press_count: u32,
    /// 记录完整的释放次数
    pub release_count: u32,
    /// 上一次的按键状态，用于检测状态变化
    pub last_button_state: bool,
    /// 第一次按下的时间，用于双击判断
    pub first_press_time: Option<Instant>,
}
```

### Default 实现

```rust
impl Default for ButtonCheckState {
    fn default() -> Self {
        Self {
            last_press_time: None,
            last_release_time: None,
            long_press_triggered: false,
            single_press_pending: false,
            double_press_triggered: false,
            press_count: 0,
            release_count: 0,
            last_button_state: false,
            first_press_time: None,
        }
    }
}
```

## 双击检测逻辑设计

### 核心思路

1. **状态变化检测**：只有当按键状态从释放变为按下，或从按下变为释放时才进行计数
2. **完整按压周期**：一次完整的按压包含press和release两个状态变化
3. **时间窗口判断**：两次完整的按压必须在指定的时间窗口内完成
4. **防重复触发**：一旦双击触发，需要等待完整的释放后才能重新开始检测

### 详细逻辑

#### 1. 状态变化检测和防重复计数

```rust
// 检测按键状态变化
let state_changed = button_is_pressed != check_state.last_button_state;

// 只有在状态真正发生变化时才进行处理
if state_changed {
    if button_is_pressed {
        // 从释放变为按下 - 记录一次按下
        // 但需要确保之前已经有过对应的释放，或者这是第一次按下
        if check_state.release_count >= check_state.press_count {
            check_state.press_count += 1;
            check_state.last_press_time = Some(now);
            
            // 如果是第一次按下，记录第一次按下时间
            if check_state.press_count == 1 {
                check_state.first_press_time = Some(now);
            }
        }
    } else {
        // 从按下变为释放 - 记录一次释放
        // 只有在之前有过按下的情况下才记录释放
        if check_state.press_count > check_state.release_count {
            check_state.release_count += 1;
            check_state.last_release_time = Some(now);
        }
    }
    
    // 更新上一次的按键状态
    check_state.last_button_state = button_is_pressed;
} else if button_is_pressed {
    // 状态没有变化但按键仍被按下
    // 检查是否超时，超时则重置状态
    if let Some(first_press_time) = check_state.first_press_time {
        let time_since_first_press = now.duration_since(first_press_time).as_millis() as u64;
        if time_since_first_press > check_mode_param {
            // 长按超时，重置状态
            reset_double_click_state(check_state);
        }
    }
}
```

#### 2. 双击判断逻辑

```rust
// 检查是否满足双击条件
if check_state.press_count >= 2 && check_state.release_count >= 2 {
    if let Some(first_press_time) = check_state.first_press_time {
        let time_since_first_press = now.duration_since(first_press_time).as_millis() as u64;
        
        // 检查时间窗口
        if time_since_first_press <= check_mode_param {
            // 检查是否已经触发过双击（防止重复触发）
            if !check_state.double_press_triggered {
                // 触发双击
                check_state.double_press_triggered = true;
                return true;
            }
        } else {
            // 超时，重置计数
            reset_double_click_state(check_state);
        }
    }
}

// 检查是否需要重置状态（超时或完成双击）
if should_reset_state(check_state, now, check_mode_param) {
    reset_double_click_state(check_state);
}
```

#### 3. 状态重置逻辑

```rust
fn reset_double_click_state(check_state: &mut ButtonCheckState) {
    check_state.press_count = 0;
    check_state.release_count = 0;
    check_state.double_press_triggered = false;
    check_state.first_press_time = None;
    check_state.single_press_pending = false;
}

fn should_reset_state(check_state: &ButtonCheckState, now: Instant, timeout: u64) -> bool {
    if let Some(first_press_time) = check_state.first_press_time {
        let time_since_first_press = now.duration_since(first_press_time).as_millis() as u64;
        // 超时或者已经完成双击
        time_since_first_press > timeout || 
        (check_state.press_count >= 2 && check_state.release_count >= 2 && check_state.double_press_triggered)
    } else {
        false
    }
}
```

## 完整的双击检测实现

```rust
CheckMode::Double => {
    let now = Instant::now();
    
    // 检测按键状态变化
    let state_changed = button_is_pressed != check_state.last_button_state;

    // 只有在状态真正发生变化时才进行处理
    if state_changed {
        if button_is_pressed {
            // 从释放变为按下 - 记录一次按下
            // 但需要确保之前已经有过对应的释放，或者这是第一次按下
            if check_state.release_count >= check_state.press_count {
                check_state.press_count += 1;
                check_state.last_press_time = Some(now);
                
                // 如果是第一次按下，记录第一次按下时间
                if check_state.press_count == 1 {
                    check_state.first_press_time = Some(now);
                }
            }
        } else {
            // 从按下变为释放 - 记录一次释放
            // 只有在之前有过按下的情况下才记录释放
            if check_state.press_count > check_state.release_count {
                check_state.release_count += 1;
                check_state.last_release_time = Some(now);
            }
        }
        
        // 更新上一次的按键状态
        check_state.last_button_state = button_is_pressed;
    } else if button_is_pressed {
        // 状态没有变化但按键仍被按下
        // 检查是否超时，超时则重置状态
        if let Some(first_press_time) = check_state.first_press_time {
            let time_since_first_press = now.duration_since(first_press_time).as_millis() as u64;
            if time_since_first_press > check_mode_param {
                // 长按超时，重置状态
                reset_double_click_state(check_state);
            }
        }
    }
    
    // 检查是否满足双击条件
    if check_state.press_count >= 2 && check_state.release_count >= 2 {
        if let Some(first_press_time) = check_state.first_press_time {
            let time_since_first_press = now.duration_since(first_press_time).as_millis() as u64;
            
            // 检查时间窗口
            if time_since_first_press <= check_mode_param {
                // 检查是否已经触发过双击（防止重复触发）
                if !check_state.double_press_triggered {
                    // 触发双击
                    check_state.double_press_triggered = true;
                    return true;
                }
            } else {
                // 超时，重置计数
                reset_double_click_state(check_state);
            }
        }
    }
    
    // 检查是否需要重置状态（超时或完成双击）
    if should_reset_state(check_state, now, check_mode_param) {
        reset_double_click_state(check_state);
    }
    
    false
}
```

## 优势分析

1. **准确的状态检测**：基于完整的状态变化周期，避免轮询间隔的影响
2. **防止误触发**：只有完整的press-release周期才会被计数
3. **时间窗口控制**：确保两次按压在指定时间内完成
4. **防重复触发**：双击触发后需要重置状态才能再次检测
5. **长按兼容**：长按不会误判为双击

## 测试场景

### 场景1：正常双击
1. 用户按下按键（press_count=1, first_press_time记录）
2. 用户释放按键（release_count=1）
3. 用户再次按下按键（press_count=2）
4. 用户再次释放按键（release_count=2）
5. 检查时间窗口，触发双击

### 场景2：单击
1. 用户按下按键（press_count=1, first_press_time记录）
2. 用户释放按键（release_count=1）
3. 等待超时，状态重置

### 场景3：长按
1. 用户按下按键（press_count=1, first_press_time记录）
2. 用户保持按下状态（无状态变化）
3. 超时后状态重置

### 场景4：快速轮询
1. 系统快速轮询，但只有状态变化时才会计数
2. 不会因为轮询频繁而误判为多次按压