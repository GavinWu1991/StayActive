# StayActive

[English README](./README.md)

StayActive 是一款 macOS 菜单栏应用，通过轻量模拟活动并可选阻止休眠，让电脑保持“活跃”状态。  
技术栈：Tauri v2（`Rust` + `React` + `Vite`）。

## 当前范围

- 平台支持：**仅 macOS**
- CI/CD 支持：**仅 macOS**
- 交付阶段：**MVP**

## MVP 功能

- 菜单栏托盘应用（图标显示 `on` / `off` 状态）
- 一键切换 **Stay Active**
- 定时预设：`10 分钟`、`30 分钟`、`1 小时`、`2 小时`、`3 小时`
- 自定义结束时间选择器
- 托盘中显示倒计时，并支持取消
- 设置窗口（自动化行为 + 语言 + 运动区域）
- 中英文多语言
- 辅助功能权限引导与应用内提示流程

## CI/CD（GitHub Actions，macOS-only）

- PR 流水线：`.github/workflows/ci-pr.yml`
  - 触发：对 `main` 的 Pull Request
  - 执行：`macos-latest` 上的 `quality-gate`
- 主分支流水线：`.github/workflows/release-main.yml`
  - 触发：`main` 分支 push + 可选手动触发
  - 顺序：`quality-gate` -> `build-installers-macos` -> `publish`
  - 发布产物包含可追踪元数据（`source_revision`、`pipeline_run_id`）

参考文档：

- `docs/ci/github-actions-pipeline.md`
- `specs/005-github-actions-pipeline/contracts/workflow-triggers.md`

## 首次运行（重要）

如果应用不是来自 Mac App Store：

1. 首次不要直接双击启动；
2. Finder 中右键 `.app` -> **打开** -> 再确认 **打开**；
3. 在  
   **系统设置 -> 隐私与安全性 -> 辅助功能**  
   中授予 StayActive 权限。

## 本地开发

### 前置要求

- macOS
- Node.js LTS
- Rust stable

### 安装依赖

```bash
npm install
```

### 运行（前端开发）

```bash
npm run dev
```

### 运行桌面应用（建议用于权限调试）

```bash
npm run dev:app
```

### 构建

```bash
npm run build
cargo tauri build
```

构建产物（macOS `.app`）：

`src-tauri/target/release/bundle/macos/StayActive.app`

### 可选 ad-hoc 签名（MVP）

```bash
codesign --force --deep -s - src-tauri/target/release/bundle/macos/StayActive.app
```

## 质量门禁命令

当前流水线质量门禁使用以下命令：

```bash
npm ci
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

也可使用封装脚本：

```bash
bash scripts/ci/commands.sh quality-gate
```

## 调试日志

Debug 构建日志路径：

`~/Library/Logs/StayActive/debug.log`

实时查看：

```bash
tail -f ~/Library/Logs/StayActive/debug.log
```
