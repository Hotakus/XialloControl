<h1 align="center">XenoControl</h1>

<p align="center">
  <strong>🎨 为创意工作者设计的手柄控制解决方案</strong>
</p>

<p align="center">
  <a href="https://github.com/hotakus/XenoControl/releases/latest">
    <img src="https://img.shields.io/github/v/release/hotakus/XenoControl?style=flat-square&logo=github" alt="GitHub release">
  </a>
<a href="https://www.rust-lang.org">
    <img src="https://img.shields.io/github/actions/workflow/status/hotakus/XenoControl/app.yml?style=flat-square" alt="Rust">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MPL%202.0-orange?style=flat-square" alt="License">
  </a>
  <a href="https://github.com/hotakus/XenoControl">
    <img src="https://img.shields.io/github/repo-size/hotakus/XenoControl?style=flat-square" alt="Repo size">
  </a>
  <br>
  <a href="https://tauri.app">
    <img src="https://img.shields.io/badge/Tauri-2.7.0-FFC131?style=flat-square&logo=tauri" alt="Tauri">
  </a>
  <a href="https://www.rust-lang.org">
    <img src="https://img.shields.io/badge/Rust-2024-orange?style=flat-square&logo=rust" alt="Rust">
  </a>
  <a href="https://tauri.app">
    <img src="https://img.shields.io/badge/PackageManager-PNPM-blue?style=flat-square&logo=pnpm" alt="Tauri">
  </a>
</p>

<p align="center">
  <a href="#功能特点">功能特点</a> •
  <a href="#界面预览">界面预览</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#技术栈">技术栈</a> •
  <a href="#许可证">许可证</a>
</p>

---

### 🎨 项目简介

XenoControl 是一款为创意工作者设计的手柄控制软件，让您可以将普通游戏手柄转变为高效的工作工具。无论是数字绘画、3D建模还是视频编辑，XenoControl都能帮助您摆脱键盘的束缚，实现更直观、更流畅的操作体验。

---

### ✨ 功能特点

#### 🎮 创意工作优化

- **按键映射**：将手柄按键映射为画笔、橡皮擦、撤销等常用功能
- **摇杆控制**：使用摇杆控制画笔大小、画布缩放等操作
- **多预设方案**：保存和导入多个预设配置，适应不同软件和工作流程

#### ⚙️ 高级控制

- **死区调整**：自定义摇杆死区范围，优化操作精度
- **轮询频率**：调整手柄轮询频率（1-16000Hz）确保响应灵敏（默认125Hz，推荐1000hz，高于1khz以上可能会影响性能，并且手柄必须支持高频）
- **主题切换**：提供浅色、深色及系统跟随三种视觉主题

#### 🚀 效率提升

- **开机自启**：随系统启动，随时可用（可选）
- **托盘运行**：最小化到系统托盘，不干扰创作过程 （可选）

---

### 🖼️ 界面预览

<p align="center">
  <img src="https://via.placeholder.com/800x500/2f3542/ffffff?text=XenoControl+UI+Preview" alt="XenoControl界面预览" width="800">
</p>

*优雅的界面设计，直观的功能布局，为您提供舒适的操作体验*

---

### 🚀 快速开始

1. **下载安装**
    - 前往 [Releases页面](https://github.com/hotakus/XenoControl/releases) 下载最新版本
    - 解压后运行 `XenoControl.exe`

2. **连接设备**
    - 连接您的手柄设备
    - 点击"重新扫描"按钮检测设备

3. **配置映射**
    - 在"按键映射"标签页配置手柄按键功能
    - 在"摇杆映射"标签页调整摇杆行为

4. **保存使用**
    - 将您的配置保存为预设方案
    - 开始享受更流畅的创作体验！

---

### ⚙️ 技术栈

- **核心框架**: Tauri (Rust + Web)
- **开发工具**:
    - Rust 2024 Edition
    - Web前端技术 (HTML/CSS/JS)

---

### 📜 许可证

本项目采用 [Mozilla Public License 2.0](LICENSE)
