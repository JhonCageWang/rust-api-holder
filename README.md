# 🦀 Rust API Holder

一个轻量级的 API 调试工具,用 **Rust + Tauri** 重写 Apifox / Postman 的核心体验。

> 目标:体积小 (<15MB)、启动快 (<1.5s)、本地优先、不臃肿。

## ✨ 特性(规划中)

- ✅ HTTP 请求构造与发送
- ✅ 集合(Collection)管理
- ✅ 环境变量(支持 `{{var}}` 插值)
- ✅ 请求历史记录
- ✅ 响应查看(JSON 高亮 / Header / 耗时)
- ✅ 导入 Postman v2.1 Collection JSON
- ⏳ WebSocket / gRPC(后续版本)

## 🛠️ 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Rust + Tokio + Reqwest + rusqlite |
| GUI | Tauri 2 |
| 前端 | Vue 3 + TypeScript + Vite |
| UI | Naive UI + Pinia |
| 存储 | SQLite(bundled) |

## 📁 项目结构

```
rust-api-holder/
├── crates/
│   ├── core/         # 🧠 核心业务逻辑(纯 Rust,无 GUI 依赖)
│   └── app/          # 🖥️ Tauri 桌面应用壳
├── ui/               # 🎨 前端(Vue 3 + TS)
├── PLAN.md           # 详细开发计划
└── README.md         # 本文件
```

## 🚀 开发

### 环境要求

- Rust 1.75+(`rustup default stable`)
- Node.js 20+
- pnpm 8+
- macOS / Windows / Linux

### 本地运行

```bash
# 安装 Tauri CLI(首次,会装在 ~/.cargo/bin,可能要 5-10 分钟)
cargo install tauri-cli --version "^2.0"

# 安装前端依赖(首次)
cd ui && pnpm install && cd ..

# 开发模式(热重载) — 从 crates/app 跑,因为 tauri.conf.json 在那里
cd crates/app && cargo tauri dev
```

> 💡 也可以在根目录跑 `cargo tauri dev --config crates/app/tauri.conf.json`

### 构建发布

```bash
cd crates/app && cargo tauri build
```

产物在 `target/release/bundle/` 下(macOS 是 .dmg,Windows 是 .msi,Linux 是 .deb/.AppImage)。

### 验证 Cargo workspace

```bash
# 在根目录跑
cargo check --workspace
cargo test --workspace
```

## 📋 路线图

参见 [PLAN.md](./PLAN.md) — 9 周 MVP 计划。

## 📜 License

MIT — 详见 [LICENSE](./LICENSE)

## 🙏 致谢

灵感来自 Postman 和 Apifox,目标是用 Rust 的力量做出更轻量的替代品。