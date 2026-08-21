# 🏛️ Rust API Holder · 架构与代码阅读指南

> 这份文档回答两个问题:
> 1. **数据是怎么从用户操作流到 SQLite/HTTP 再流回来的?**
> 2. **我应该按什么顺序读代码?**
> 
> 配套文档:[`PLAN.md`](./PLAN.md)(整体计划)+ [`DEV_GUIDE.md`](./DEV_GUIDE.md)(你的任务)

---

## 一、🌊 整体架构图

```
┌──────────────────────────────────────────────────────────────────┐
│  macOS / Windows / Linux Desktop                                  │
│                                                                  │
│  ┌─────────────────────────┐   ┌──────────────────────────┐    │
│  │  WebView 进程(Chromium)│   │  Rust 主进程(Tauri 2)  │    │
│  │                         │   │                          │    │
│  │  Vue 3 + TypeScript     │   │  commands::xxx           │    │
│  │  Naive UI               │   │         ↓                │    │
│  │  Pinia                  │   │  core::http / storage    │    │
│  │                         │   │         ↓                │    │
│  │  useInvoke ───────┐     │   │  reqwest / rusqlite      │    │
│  │                 │     │   │                          │    │
│  └─────────────────│─────┘   └──────────────────────────┘    │
│                    │                  ↑                        │
│                    │     Tauri IPC   │                        │
│                    │ (JSON over     │                        │
│                    │  stdin/stdout) │                        │
└────────────────────│──────────────────│────────────────────────┘
                     ↓                  ↑
            ┌────────────────────────────────────┐
            │  真实世界:HTTP 服务 / SQLite 文件 │
            │  https://api.example.com          │
            │  ~/Library/Application Support/...│
            │     /com.jhoncagewang.api-holder/ │
            │     api-holder.db                 │
            └────────────────────────────────────┘
```

**两个进程,职责清晰**:
- **WebView 进程**:跑 Vue 前端代码(JS),渲染 UI,处理用户交互
- **Rust 主进程**:跑 Tauri + 你的 Rust 代码,执行实际逻辑,访问文件系统/网络

---

## 二、🔄 完整执行流程(以"发送请求"为例)

### 场景:用户点 "Send" 按钮,发送 `GET https://httpbin.org/get`

```
[1] 用户在 WebView 里点 "Send" 按钮
    └─→ HomeView.vue 的 button @click 触发
         │
[2] Vue 组件调用 invokeT('execute_request', { requestId: 'xxx' })
    └─→ composables/useInvoke.ts
         │
[3] invokeT 内部判断是否在 Tauri 环境
    └─→ window.__TAURI_INTERNALS__ 存在?
         ├─ 是 → window.__TAURI_INTERNALS__.invoke('execute_request', args)
         │       (这是 Tauri JS API 的入口)
         │
         └─ 否 → mockInvoke('execute_request', args)
                (返回假数据,纯浏览器开发用)
                │
[4] Tauri JS API 通过 IPC(JSON-RPC over stdin/stdout)
    └─→ 把请求发给 Rust 主进程
         │
[5] Rust 端 Tauri 接收,匹配到 #[tauri::command] 函数
    └─→ crates/app/src/commands/request.rs::execute_request
         │  接收 State<AppState> 和参数
         │
[6] Command 函数调 core 的业务逻辑
    └─→ api_holder_core::http::execute(req, vars).await
         │
[7] core::http::execute 里:
    ├─ interpolate(req.url, &vars)       ← 替换 {{var}}
    ├─ reqwest::Client::new().send(...)   ← 真实网络请求
    ├─ 拿到 reqwest::Response
    └─ 转成我们的 Response 结构
         │
[8] 调 history_repo.create(...) 写历史
    └─→ api_holder_core::storage::Database.with_conn(|c| {
          rusqlite::INSERT INTO history ...
        })
         │
[9] Response 数据从 Rust 返回 Tauri 框架
    └─→ 序列化成 JSON
         │
[10] JSON 通过 IPC 回到 WebView 进程
     └─→ invokeT() 的 Promise resolve(response)
          │
[11] Vue 组件的 .then() 拿到数据
     └─→ Pinia store 更新 / ref 触发重渲染
          │
[12] Naive UI 组件用新数据重新渲染
     └─→ 用户看到响应结果
```

### 关键节点

| # | 位置 | 关键文件 |
|---|---|---|
| [1-2] | Vue 组件 | `ui/src/views/HomeView.vue` |
| [3] | Tauri 调用封装 | `ui/src/composables/useInvoke.ts` |
| [4] | IPC 跨进程 | (Tauri 内置,不需要看) |
| [5] | Tauri 入口 | `crates/app/src/main.rs` |
| [6-7] | Command + 业务逻辑 | `crates/app/src/commands/*.rs` + `crates/core/src/http/mod.rs` |
| [8] | 持久化 | `crates/core/src/storage/mod.rs` |
| [9-12] | 回流 | 反向 |

---

## 三、🗺️ 文件地图(每个文件是干嘛的)

### 📂 Workspace 根(`/Users/wangpangpang/rust-api-holder/`)

| 文件 | 作用 | 你需要读吗 |
|---|---|---|
| `Cargo.toml` | Workspace 根,定义两个成员 + 共享依赖 | **必读**(理解项目结构) |
| `Cargo.lock` | 自动生成的依赖锁文件 | 不读(改不了也不该改) |
| `README.md` | 项目说明 | 不读(给其他人看的) |
| `PLAN.md` | 9 周开发计划 | 选读(知道大方向) |
| `DEV_GUIDE.md` | 你的任务清单 | **必读**(你的工作指南) |
| `ARCHITECTURE.md` | 本文档 | **必读**(正在读) |

### 📂 Rust 后端核心(`crates/core/`)

```
crates/core/
├── Cargo.toml           ← 依赖(读 workspace 共享的 + core 自己特有的)
│
└── src/
    ├── lib.rs           ⭐ 入口,声明所有 pub mod
    │                     第一站:了解这个 crate 暴露什么
    │
    ├── error.rs          ⭐ Error 类型(必读)
    │                     所有错误都从这里来,理解它能看懂所有 `?`
    │
    ├── http/
    │   └── mod.rs        ⭐ Request/Response/Method 数据结构
    │                      ⚠️ execute() 还在等你写(Week 2)
    │
    ├── environment/
    │   └── mod.rs        ⭐ interpolate() 已实装,必读
    │                      Repository/CRUD 还没写
    │
    ├── collection/
    │   └── mod.rs        🟡 Collection/RequestItem 模型
    │                      必读(理解数据结构)
    │
    ├── history/
    │   └── mod.rs        🟡 HistoryEntry 模型
    │                      选读(Week 6 才用)
    │
    ├── import/
    │   └── mod.rs        ⬜ 完全是占位,Week 7 才写
    │
    └── storage/
        ├── mod.rs        ⭐ Database 封装(已实装)
        └── migrations.rs ⭐ SQL schema(已实装)
```

### 📂 Tauri 应用壳(`crates/app/`)

```
crates/app/
├── Cargo.toml            ← 依赖 + bin 配置
│
├── build.rs              ← Tauri 构建脚本(嵌入资源)
│
├── tauri.conf.json       ⭐ Tauri 应用配置
│                         (窗口大小 / 图标 / 打包目标)
│
├── capabilities/
│   └── default.json      ⭐ Tauri 2 权限(替代 v1 的 allowlist)
│
├── icons/
│   └── icon.png          ← 应用图标(目前是占位)
│
└── src/
    ├── main.rs           ⭐⭐ Tauri 入口
    │                      setup() / invoke_handler 注册
    │
    └── commands/
        └── mod.rs        ⭐⭐ Tauri Commands 定义
                          现有:ping + app_info
                          Week 4 起:加更多
```

### 📂 前端(`ui/`)

```
ui/
├── package.json          ← npm 依赖
├── vite.config.ts        ← Vite 构建配置(端口、Tauri dev URL)
├── tsconfig.json         ← TypeScript 配置
├── index.html            ← HTML 入口(挂载 #app)
│
└── src/
    ├── main.ts           ⭐ Vue 应用启动(注册组件、Pinia、Router)
    ├── App.vue           ⭐ 顶层布局(三 Tab)
    ├── env.d.ts          ← TypeScript 类型声明
    │
    ├── router/
    │   └── index.ts      ⭐ Vue Router 配置(三个路由)
    │
    ├── types/
    │   └── api.ts        ⭐⭐⭐ 前端类型契约
    │                      任何新增 Tauri Command 必须同步更新这里
    │
    ├── composables/
    │   └── useInvoke.ts  ⭐⭐ Tauri invoke 封装
    │                      命令签名映射在这里
    │
    ├── stores/
    │   └── app.ts        ⭐ Pinia 全局状态
    │                      (isBackendReady / appInfo / activeEnv)
    │
    └── views/
        ├── HomeView.vue       ⭐ 主界面(请求编辑 + 响应)
        ├── EnvironmentsView.vue  环境管理
        └── HistoryView.vue       请求历史
```

---

## 四、📖 三条阅读路线(看你的目的)

### 🛣️ 路线 A:**完全新手,想理解整个项目**(4 小时)

按这个顺序读,每步都先看代码,再跑相关测试:

```
Step 1: 项目骨架 (15 min)
   → 读 Cargo.toml(workspace 结构)
   → 读 crates/app/tauri.conf.json(理解 Tauri 配置)
   → 读 DEV_GUIDE.md 第一节(我写了什么)

Step 2: 前端从启动到显示 (1 hour)
   → ui/index.html (15 行)
   → ui/src/main.ts (启动顺序)
   → ui/src/App.vue (顶层组件)
   → ui/src/router/index.ts (路由)
   → ui/src/views/HomeView.vue (页面)
   → 跑 vite dev 在浏览器看效果

Step 3: 前端调后端的桥 (30 min)
   → ui/src/composables/useInvoke.ts
   → ui/src/types/api.ts
   → 理解 invokeT() 怎么把 'ping' 变成真实 invoke

Step 4: Tauri 入口 (30 min)
   → crates/app/src/main.rs
   → crates/app/src/commands/mod.rs
   → 理解 #[tauri::command] 怎么暴露给前端

Step 5: 核心库结构 (1 hour)
   → crates/core/src/lib.rs
   → crates/core/src/error.rs
   → crates/core/src/environment/mod.rs(interpolate 已实装,能跑测试)
   → crates/core/src/storage/mod.rs + migrations.rs
   → 跑 cargo test --workspace 看 9 个测试通过

Step 6: 跑起来 + 玩一下 (30 min)
   → cargo tauri dev (你桌面弹窗)
   → 在浏览器打开 vite dev 5173 看 mock 模式
   → 对比两边的 invokeT 调用

做完 Step 6 后,你就理解整个项目了。
```

### 🛣️ 路线 B:**想实现一个新功能**(比如"加一个 list_collections command")

```
Step 1: 看前端怎么用
   → 打开 ui/src/composables/useInvoke.ts
   → 找 list_collections(目前没有,加一行)

Step 2: 看后端结构
   → crates/app/src/commands/mod.rs
   → 加 #[tauri::command] pub fn list_collections(...)

Step 3: 看业务逻辑放哪
   → crates/core/src/collection/mod.rs (模型)
   → crates/core/src/storage/mod.rs (Database)
   → 写 Repository(Week 3 才有,目前 model 阶段)

Step 4: 在 main.rs 注册
   → crates/app/src/main.rs 的 invoke_handler!
   → 加 commands::list_collections

Step 5: 跑 cargo tauri dev 验证
   → 在 devtools console 看 invoke 是否成功
   → 加 eprintln! 或 tracing::debug! 调试
```

### 🛣️ 路线 C:**遇到 bug,逆向追踪**(30 分钟)

```
场景:用户点 Send 没反应

Step 1: 看前端报错(浏览器 DevTools console)
   → ui/src/views/HomeView.vue 的 click handler 有没有错?

Step 2: 看 invoke 调用是否发出
   → 在 ui/src/composables/useInvoke.ts 的 invokeT 加 console.log

Step 3: 看后端 Command 是否被调用
   → crates/app/src/commands/mod.rs 的 command 函数加 println!
   → 或 tracing::info!("called list_collections")

Step 4: 看 core 业务逻辑
   → 在 core 的某个函数加 dbg!() 打印中间值

Step 5: 找到 bug,修
```

---

## 五、🎯 阅读代码的几个技巧

### 1. 从 `lib.rs` 开始,看 `pub mod` 列表
```rust
// crates/core/src/lib.rs
pub mod collection;
pub mod environment;
pub mod error;
pub mod history;
pub mod http;
pub mod import;
pub mod storage;
```
这就是整个 crate 的"目录"。

### 2. 看每个模块的顶部 doc comment
我每个模块都写了 `//!` 注释,讲清楚这个模块干嘛:
```rust
//! HTTP 请求执行模块
//! 封装 reqwest,提供:
```

### 3. 从测试看 API 设计
测试代码比实现代码更"短小直接",能快速看懂模块怎么用:
```bash
# 跑某个模块的测试
cargo test -p api-holder-core environment

# 看具体测试代码
grep -A 20 "fn test_" crates/core/src/environment/mod.rs
```

### 4. 用 `cargo doc` 看完整文档
```bash
cargo doc -p api-holder-core --no-deps --open
# 浏览器打开 HTML 文档,带超链接,可以点击跳转
```

### 5. 用 IDE 跳转
- VSCode + rust-analyzer:按住 Cmd 点击函数名,跳到定义
- `gd` / `gi` 等快捷键

### 6. 善用搜索
```bash
# 找某个函数的定义
grep -rn "fn execute" crates/core/src

# 找某个类型在哪里用
grep -rn "pub struct Collection" crates/

# 找所有 #[tauri::command]
grep -rn "#\[tauri::command\]" crates/app/src/
```

---

## 六、🔁 数据流的"对偶性"

理解一个 Rust + Tauri 项目最好的方式是:**前端和后端是对偶的**。

```
前端类型                  ←→               后端类型
─────────────────────         ─────────────────────
ui/src/types/api.ts            crates/core/src/*/mod.rs

HttpMethod                     Method
KeyValue                       KeyValue
RequestBody                    Body
Auth                           Auth
ApiRequest                     Request
ApiResponse                    Response
Collection                     Collection
RequestItem                    RequestItem
Environment                    Environment
Variable                       Variable
HistoryEntry                   HistoryEntry
AppInfo                        AppInfo
```

**两边结构必须严格对应**,所以:
- 前端加字段 → 后端必须加
- 后端 Command 返回类型改了 → 前端 invoke 类型必须改

我们在 `useInvoke.ts` 里用 TypeScript 类型签名强制了这个约束。

---

## 七、🚀 推荐你的阅读顺序(今天)

如果你**完全没看过程序**,按这个顺序:
1. **本文件第二节**(执行流程图)— 5 分钟
2. **本文件第三节**(文件地图)— 5 分钟
3. **`crates/core/src/lib.rs`** — 1 分钟(只有 32 行)
4. **`crates/core/src/error.rs`** — 10 分钟(理解所有错误)
5. **`crates/core/src/environment/mod.rs`** — 15 分钟(看完整实装的代码长啥样)
6. **`ui/src/types/api.ts`** — 10 分钟(理解前端契约)
7. **`ui/src/composables/useInvoke.ts`** — 10 分钟(理解 invoke 怎么调)
8. **`crates/app/src/commands/mod.rs`** — 10 分钟(理解 Command 怎么写)

**总计 ~70 分钟**,做完你对整个项目就有完整画面。

然后开始 Week 2 任务(DEV_GUIDE.md 第四节),就不会迷路。

---

## 八、❓ 快速 FAQ

### Q:前端改了,Rust 需要重启吗?
A:**不需要**。Vite 有 HMR(热重载),前端改了页面自动刷新。Rust 改了 `cargo tauri dev` 会自动重启。

### Q:在哪一层加新功能?
A:看你是哪种功能:
- **纯 UI** → 只改 `ui/src/views/*.vue`
- **数据展示** → 前端 + 一个后端 Command
- **新业务逻辑** → `crates/core/src/*/mod.rs` 加函数
- **新表** → `migrations.rs` 加 SQL + 新 Repository

### Q:为什么 `crates/app/src/main.rs` 这么短?
A:大部分逻辑在 `core/` 里,`app/` 只是 Tauri 壳。Tauri 的设计就是让你**业务逻辑和 GUI 解耦**。

### Q:`State<AppState>` 是怎么注入的?
A:`app.manage(AppState::default())` 注册,`State<'_, AppState>` 自动注入到 Command 参数。这是 Tauri 的"依赖注入"机制。

### Q:怎么知道前端类型和后端对不对得上?
A:看 `ui/src/composables/useInvoke.ts` 里的 `CommandSignatures` interface。任何增删 Command,两边一起改。

---

> 🎯 **核心思想**:架构清晰 + 数据单向流动 + 前后端类型同步
> 理解了这三件事,这个项目就 90% 透明了。