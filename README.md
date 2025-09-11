<h1 align="center">XialloControl</h1>

<p align="center">
  <strong>高通用性手柄映射软件（支持各种主流手柄 Xbox、PS、SwitchPro）</strong>
</p>

<p align="center">
  <a href="https://github.com/hotakus/XialloControl/releases/latest">
    <img src="https://img.shields.io/github/v/release/hotakus/XialloControl?style=flat-square&logo=github" alt="GitHub release">
  </a>
<a href="https://www.rust-lang.org">
    <img src="https://img.shields.io/github/actions/workflow/status/hotakus/XialloControl/app.yml?style=flat-square" alt="Rust">
  </a>
  <a href="LICENSE.txt">
    <img src="https://img.shields.io/badge/License-GPL%203.0-orange?style=flat-square" alt="License">
  </a>
  <a href="https://github.com/hotakus/XialloControl">
    <img src="https://img.shields.io/github/repo-size/hotakus/XialloControl?style=flat-square" alt="Repo size">
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
  <a href="#许可证">许可证</a>
</p>

---

### 功能特点：

### **XialloControl** 是一款：

#### 🎨 设计：
- **开源跨平台 （windows、linux）**
- **极简**：程序精简，windows 安装包仅有 **2 MB** 左右
- **美观**程度高，卡片式UI设计，操作界面一目了然！
- **高性能**服务提供！后端采用 Rust 语言设计，高效操作，丝滑无比！

#### ⚙️ 高级控制
- **支持**自定义摇杆死区范围，优化操作精度，提供校准功能
- **支持**市面多种手柄，如 Xbox、PS、SwitchPro 等（实测杂牌手柄也行XD）
- **高度**自定义手柄键位映射（支持鼠标、键盘映射、键盘鼠标按键和滚轮混合映射，
  支持各种快捷键，如：Shift+Alt+鼠标滚轮等复杂混合映射，满足多场景支持）

#### 🎮 创意工作优化
- **灵敏**度可调，适应各种用户各种手感！
- **支持**摇杆映射（支持摇杆死区调整，支持摇杆映射为鼠标移动、画笔大小、画布缩放等操作）
- **支持**多预设方案（保存和导入多个预设配置，适应不同软件和工作流程）

#### 🚀 效率提升
- **支持**开机自启，可调“自动连接手柄”，让你从开机到工作顺滑无比！
- **支持**托盘运行，最小化到系统托盘，不干扰创作过程！
- **安全**卸载，卸载后不影响手柄正常功能，您可以放心使用

---

### 🖼️ 界面预览

<p align="center">
  <img src="https://via.placeholder.com/800x500/2f3542/ffffff?text=XialloControl+UI+Preview" alt="XialloControl界面预览" width="800">
</p>

*优雅的界面设计，直观的功能布局，为您提供舒适的操作体验*

---

### 🚀 快速开始

1. **下载安装**
    - 前往 [Releases页面](https://github.com/hotakus/XialloControl/releases) 下载最新版本
    - Windows 用户下载 `XialloControl.exe`

2. **连接设备**
    - 点击"重新扫描"按钮检测设备
    - 从下拉列表选择你的手柄
    - 连接您的手柄设备

3. **配置映射**
    - 在"按键映射"标签页配置手柄按键功能
    - 在"摇杆映射"标签页调整摇杆行为

4. **保存使用**
    - 将您的配置保存为预设方案
    - 开始享受更流畅的创作体验！

---

### 📜 许可证

本项目的代码部分遵循 [GNU General Public License v3.0](LICENSE.txt) 许可证开源。

**关于图形资源**

本项目中使用的所有位于 `src/assets/controller` 目录下的 `.svg` 手柄布局图是本项目的原创作品，其版权归项目作者所有。

这些图形资源**不**在 GPLv3 许可范围内，并**保留所有权利 (All Rights Reserved)**。

未经项目作者明确的书面许可，**禁止**以任何形式对这些图形资源进行复制、修改、分发或用于商业目的。
