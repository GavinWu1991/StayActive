# StayActive 软件详细设计文档 (StayActive Design Spec)

## 1. 系统概览 (System Overview)

StayActive 是一款基于 **Tauri v2** 和 **Rust** 开发的 macOS 菜单栏应用。其核心功能是通过轻量级模拟 HID 事件重置系统的 `HIDIdleTime`，从而防止 Teams 等协作软件因闲置变黄，并抑制系统进入睡眠。

### 技术栈选型

* **后端**: Rust (Tauri v2)
* **前端**: React + Vite (用于极简的配置窗口)
* **模拟引擎**: `enigo` (v0.2+)
* **系统断言**: `keepawake-rs` (直接调用 IOKit)
* **权限管理**: `tauri-plugin-macos-permissions`
* **任务调度**: `tokio-cron-scheduler` (用于定时任务)

---

## 2. 核心架构设计 (Core Architecture)

### 2.1 线程模型 (Threading Model)

为了确保 macOS 系统的稳定性， StayActive 采用三层并发设计：

1. **Main Thread (Runtime)**: 运行 Tauri 主事件循环和系统托盘（System Tray） 。


2. **Worker Thread (Automation)**: 独立的异步线程，负责执行随机移动/点击逻辑，使用 `tokio::select!` 监听来自 UI 的“暂停/终止”信号 。


3. **Monitor Thread**: 运行 `rdev` 或 `objc2` 监听器，检测用户是否正在物理操作。若用户活动，则自动挂起模拟器，实现“干预保护” 。



### 2.2 数据模型：可扩展动作协议 (Action Protocol)

为了支持未来的“事件组合配置”，系统将动作序列抽象为可 JSON 序列化的枚举。

```rust
#
#[serde(tag = "type", content = "payload")]
enum Action {
    MouseMove { x: i32, y: i32, relative: bool },
    MouseClick { button: String, count: u32 },
    KeyPress { key: String },
    Wait { ms: u64 },
    // 预留：切换应用、屏幕截图、逻辑判断
}

struct AutomationProfile {
    name: String,
    steps: Vec<Action>,
    loop_interval: Duration, // 可支持随机范围 (Min, Max)
}

```

---

## 3. 功能详细设计 (Feature Details)

### 3.1 模拟策略：微量位移与 Retina 适配

* **策略**: 默认在当前坐标  像素内做随机位移。
* **坐标转换**: 调用 `core-graphics` 获取 `scale_factor`。模拟坐标计算公式： 。

### 3.2 权限引导与检测

* 应用启动时调用 `AXIsProcessTrustedWithOptions`。
* 若无权限，Tauri 后端向前端发送 `permission_required` 事件，前端显示全屏引导页，说明如何通过“右键点击 -> 打开”绕过 Gatekeeper，以及如何在“系统设置”授予辅助功能权限 。



### 3.3 未来扩展性：定时/指定时间退出

* **实现方案**: 使用 `chrono` 获取当前时间，配合 `tokio-cron-scheduler` 。


* **配置项**:
* `stop_at`: 指定 ISO 时间字符串（如 "2026-02-03T18:00:00"）。
* `duration`: 运行 X 小时后自动关闭。



---

## 4. 开发工作拆解与 AI 指令 (WBS for AI)

### 第一步：初始化与托盘模式 (AI Instruction 1)

> "基于 Tauri v2 初始化一个 Rust 项目。修改 `tauri.conf.json` 以隐藏 Dock 图标（`LSUIElement: true`），在 Rust 后端实现一个带有 'Start/Stop', 'Settings', 'Quit' 的系统托盘。所有 UI 组件应使用原生系统菜单实现。"

### 第二步：核心自动化引擎 (AI Instruction 2)

> "集成 `enigo` 和 `keepawake-rs`。在 Rust 后端创建一个循环任务：每隔随机时间（45-90秒）移动鼠标 1-3 像素。在任务激活时，调用 `IOPMAssertionCreateWithName` 防止系统休眠。支持通过 Tauri Command 开启和关闭此任务。"

### 第三步：动作序列与 JSON 序列化 (AI Instruction 3)

> "设计一个 `Action` 枚举和 `AutomationProfile` 结构体，使用 `serde` 实现 JSON 转换。编写一个解释器函数，能够按照 Profile 中的步骤顺序执行动作（如：移动 -> 等待500ms -> 点击）。"

### 第四步：权限与 Retina 适配 (AI Instruction 4)

> "编写 Rust FFI 调用 `AXIsProcessTrusted` 来检查辅助功能权限。实现一个辅助函数获取当前主屏幕的缩放比例（Scale Factor），并确保所有 `enigo` 的位移量都根据此比例进行缩放。"

---

## 5. 打包与分发说明 (MVP Distribution)

### 5.1 Ad-hoc 签名

针对方案 1，必须在编译后执行：

```bash
# AI 生成的代码中应包含此自动构建脚本
codesign --force --deep -s - target/release/bundle/macos/StayActive.app

```

### 5.2 用户绕过指南模板 (README.md 内容建议)

1. **首次运行**: 下载后，请**不要直接双击**。
2. **安全绕过**: 在 Finder 中**右键点击**图标，选择“打开”。
3. **权限授予**: 在弹出的系统窗口中点击“打开”。随后前往“系统设置 -> 隐私与安全性 -> 辅助功能”，勾选“StayActive”。

---

## 6. 技术风险评估 (Technical Risks)

* **Teams 深度检测**: 部分版本的 Teams 可能会检测特定的系统断言（Power Assertion）。
* *对策*: 模拟脚本应保持周期性的小幅度鼠标位移，这比单纯的电源断言更难被应用层屏蔽 。


* **主线程阻塞**: 系统托盘事件如果处理过久会导致界面卡死。
* *对策*: 所有模拟动作必须在 `tauri::async_runtime::spawn` 的闭包中异步执行 。