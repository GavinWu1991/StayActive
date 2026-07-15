# StayActive

[English README](./README.md)

**StayActive** 是一款轻量的 macOS 菜单栏应用，用于让电脑保持“活跃”状态——协作软件（如 Microsoft Teams）不易因闲置变黄，并可选阻止系统休眠。

应用仅驻留菜单栏（不显示 Dock 图标），通过模拟轻微鼠标活动，并可选持有系统唤醒断言来工作。

> **平台：** 仅 macOS · **状态：** MVP（v0.1.0）

---

## 面向用户

### 能解决什么问题？

- 长时间开会或专注时，让 Teams 等应用保持“在线/可用”
- 开启「保持活跃」时可选择阻止 Mac 休眠
- 菜单栏一键开关；支持定时自动关闭
- 界面支持中文与英文

### 下载

1. 打开最新 [GitHub Release](https://github.com/GavinWu1991/StayActive/releases)。
2. 下载 macOS `.dmg`（或 `.app` 产物）。
3. 日常使用前，请先按下方 **首次运行** 完成授权。

### 首次运行（重要）

当前包并非来自 Mac App Store，系统 Gatekeeper 会拦截普通双击启动。

1. 首次请**不要**直接双击打开。
2. 在 Finder 中**右键**点击 `.app` → **打开** → 再确认 **打开**。
3. 按提示授予**辅助功能**权限：  
   **系统设置 → 隐私与安全性 → 辅助功能** → 勾选启用 **StayActive**。

未授予辅助功能时，应用无法模拟输入，并会引导你完成授权。

### 如何使用

点击菜单栏图标：

| 菜单项 | 作用 |
|--------|------|
| **保持活跃** | 开启 / 关闭活动模拟 |
| **定时器** | 在 10 分钟 / 30 分钟 / 1 小时 / 2 小时 / 3 小时后自动停止，或 **自定义…** 结束时间 |
| 倒计时（已设置定时时） | 显示剩余时间；点击可取消定时（保持活跃会继续运行） |
| **设置…** | 间隔、鼠标移动/点击选项、阻止休眠、语言 |
| **退出** | 退出应用 |

托盘图标会反映开/关状态。

### 隐私与行为说明

- 活动仅在本机完成：轻微鼠标移动/点击，不会上传到服务器。
- 若你刚操作过鼠标/键盘，模拟会短暂让开，避免打扰真实操作。
- 当前为 MVP，个别协作软件在边缘场景仍可能判定闲置；如遇问题欢迎提 Issue。

---

## 面向贡献者

欢迎提交问题、文档改进与 PR。请尽量保持改动范围清晰，并与现有 macOS MVP 方向一致。

### 技术栈

| 层级 | 技术 |
|------|------|
| 壳层 | [Tauri v2](https://v2.tauri.app/) |
| 后端 | Rust（`src-tauri/`） |
| 前端 | React 18 + Vite + TypeScript（`src/`） |
| 输入模拟 | `enigo` |
| 防休眠 | `keepawake` |

### 前置要求

- macOS
- [Node.js](https://nodejs.org/) 20+（LTS；CI 使用 Node 20）
- [Rust](https://rustup.rs/) stable（edition 2021，`rust-version` ≥ 1.70）
- Tauri CLI：`cargo install tauri-cli`（或通过项目既有方式使用 `cargo tauri`）

### 快速开始

```bash
npm install
```

仅前端（浏览器）：

```bash
npm run dev
```

**桌面应用（推荐）**——调试辅助功能权限时需要（系统设置里只能添加 `.app`）：

```bash
npm run dev:app
```

脚本会构建 debug `.app` 并打开。请在 **辅助功能** 中添加该 `.app`，必要时再跑一次 `dev:app`。

### 目录结构

```
src/                 React 界面（设置、定时选择器、多语言）
src-tauri/           Rust 后端（托盘、自动化、权限、定时器）
scripts/             开发辅助与 CI 命令封装
docs/                设计与 CI 文档
specs/               功能规格 / 契约
.github/workflows/   PR 质量门禁、主分支构建、标签发版
```

### 构建

```bash
npm run build
cargo tauri build
```

应用包路径：

`src-tauri/target/release/bundle/macos/StayActive.app`

可选本地 ad-hoc 签名（MVP）：

```bash
codesign --force --deep -s - src-tauri/target/release/bundle/macos/StayActive.app
```

### 质量门禁

与 CI 相同：

```bash
bash scripts/ci/commands.sh quality-gate
```

或手动执行：

```bash
npm ci
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

### 调试日志

Debug 构建日志路径：

`~/Library/Logs/StayActive/debug.log`

```bash
tail -f ~/Library/Logs/StayActive/debug.log
```

### CI / CD（仅 macOS）

| 流水线 | 触发 | 内容 |
|--------|------|------|
| [ci-pr.yml](.github/workflows/ci-pr.yml) | PR → `main` | 在 `macos-latest` 跑质量门禁 |
| [release-main.yml](.github/workflows/release-main.yml) | 推送到 `main` | 质量门禁 → 构建安装包 → 上传产物（**不**发公开 Release） |
| [release-tag.yml](.github/workflows/release-tag.yml) | `v*` 标签 | 构建并发布到 [GitHub Release](https://github.com/GavinWu1991/StayActive/releases) |

详见：[docs/ci/github-actions-pipeline.md](./docs/ci/github-actions-pipeline.md)

### 发布正式版本

1. 确认 `src-tauri/tauri.conf.json` 中的 `version`（例如 `0.1.0`）。
2. 合并到 `main` 后：

```bash
git tag v0.1.0
git push origin v0.1.0
```

标签流水线会自动发布 Release 与下载资源。

### 延伸阅读

- 设计说明：[docs/start.md](./docs/start.md)
- 功能契约见 `specs/` 目录

---

## 许可证

仓库尚未发布 LICENSE 文件。在明确许可之前，版权归作者所有——如需再分发请先沟通。