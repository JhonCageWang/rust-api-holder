# Rust API Holder · 项目计划书

> 一个轻量级的 API 调试工具,用 Rust + Tauri 重写 Apifox/Postman 的核心体验。
> 目标:体积小(<10MB)、启动快(秒开)、本地优先、不臃肿。

> 📌 **v2 修订(2024-XX)**:根据用户时间预算(周内 1h/天,周末 2h/天,共 9h/周)+ 角色分工(用户只写后端),原 7 周计划已扩展为 9-10 周,并明确了协作分工。

---

## 一、🎯 项目愿景与定位

### 解决什么问题
- Postman / Apifox 体积越来越大(几百 MB),功能堆叠过多
- 很多团队协作功能我们用不上
- 想要一个**打开就能用**的本地工具

### 我们的取舍
| ✅ 做 | ❌ 不做(v1) |
|---|---|
| HTTP 请求构造与发送 | 团队协作 / 云同步 |
| 集合(Collection)管理 | Mock Server |
| 环境变量 | API 文档发布站点 |
| 请求历史 | 自动化测试套件 |
| 响应查看(JSON / Header / 耗时) | gRPC / WebSocket / GraphQL(预留接口) |
| 导入 Postman v2.1 JSON | 插件市场 |
| 数据本地 SQLite 存储 | 多端同步 |

### 成功标准(MVP 验收)
1. 打包后体积 < 15MB
2. 冷启动 < 1.5 秒
3. 能完整跑通:导入 Postman 集合 → 选环境 → 发送请求 → 查看响应
4. 关闭重开数据不丢

---

## 二、🧰 技术栈选型

### 后端 (Rust)
| 用途 | Crate | 选择理由 |
|---|---|---|
| GUI 框架 | `tauri` v2 | 跨平台、打包小、官方活跃 |
| HTTP 客户端 | `reqwest` | 生态最成熟,支持异步 / HTTPS / 代理 |
| 异步运行时 | `tokio` | reqwest 依赖,且是事实标准 |
| 序列化 | `serde` / `serde_json` | 必备 |
| 数据库 | `rusqlite` (bundled) | 零依赖、SQL 灵活、bundled 模式无需外部 so |
| 错误处理 | `thiserror` + `anyhow` | 库用 thiserror,应用层用 anyhow |
| 时间 | `chrono` | 时间戳 / 格式化 |
| ID 生成 | `uuid` | 集合 / 请求唯一标识 |
| 日志 | `tracing` | Tauri 生态推荐 |
| 配置 | `toml` + `serde` | 简单配置文件 |

### 前端
| 用途 | 技术 | 选择理由 |
|---|---|---|
| 框架 | **Vue 3** + `<script setup>` | 国人友好、上手快、文档全 |
| 构建 | **Vite 5** | Tauri 官方推荐模板 |
| 类型 | **TypeScript** | 接口契约清晰,IDE 提示好 |
| 状态管理 | **Pinia** | Vue 3 官方推荐 |
| UI 组件 | **Naive UI** | 专为 Tauri 类桌面应用优化、体积小 |
| 代码高亮 | `highlight.js` | JSON / XML / HTML 高亮 |
| 图标 | `@vicons/ionicons5` | 与 Naive UI 配套 |
| Markdown | `markdown-it` | 响应体 markdown 渲染 |

> 💡 **为什么不选 React / Svelte?**
> Vue 3 + Naive UI 是国内桌面端最稳的组合,生态文档中文友好,对学习者更友好。

### 开发工具链
- `cargo` + `cargo-watch`(增量编译)
- `pnpm`(前端包管理,比 npm 快)
- `tauri-cli`(桌面打包)
- `sqlx-cli` 或直接用 `rusqlite`(我们不用 sqlx,免运行时)
- Git + GitHub(版本管理)

---

## 三、📁 项目目录结构

采用 **Cargo Workspace**,把核心逻辑和 Tauri 壳子解耦。这样:
- 核心逻辑可以单测、不依赖 GUI
- 未来想做 CLI 版本(`api-holder send xxx.json`)很容易
- 编译更快(改了 GUI 不会重编核心)

```
rust-api-holder/
├── PLAN.md                          # 本文件
├── README.md                        # 项目说明(给人看的)
├── LICENSE                          # MIT
├── .gitignore
│
├── Cargo.toml                       # Workspace 根配置
├── Cargo.lock
│
├── crates/
│   ├── core/                        # 🧠 核心业务逻辑(纯 Rust,无 GUI 依赖)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── http/                # HTTP 请求执行
│   │       │   ├── mod.rs
│   │       │   ├── client.rs        # reqwest 封装
│   │       │   └── response.rs      # 响应模型
│   │       ├── collection/          # 集合 / 请求模型
│   │       │   ├── mod.rs
│   │       │   ├── collection.rs
│   │       │   └── request.rs
│   │       ├── environment/         # 环境变量
│   │       │   ├── mod.rs
│   │       │   └── variable.rs
│   │       ├── history/             # 历史记录
│   │       │   └── mod.rs
│   │       ├── import/              # 导入(Postman v2.1)
│   │       │   ├── mod.rs
│   │       │   └── postman.rs
│   │       ├── storage/             # SQLite 持久化
│   │       │   ├── mod.rs
│   │       │   ├── db.rs            # 连接管理
│   │       │   ├── migrations.rs    # 建表 SQL
│   │       │   └── repo/            # 各实体的 Repository
│   │       │       ├── mod.rs
│   │       │       ├── collection.rs
│   │       │       ├── request.rs
│   │       │       ├── environment.rs
│   │       │       └── history.rs
│   │       └── error.rs             # 核心错误类型
│   │
│   └── app/                         # 🖥️ Tauri 桌面应用壳
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       ├── build.rs
│       ├── icons/                   # 应用图标(后续生成)
│       └── src/
│           ├── main.rs              # Tauri 入口
│           └── commands/            # Tauri Commands(给前端调用的)
│               ├── mod.rs
│               ├── collection.rs
│               ├── request.rs
│               ├── environment.rs
│               └── history.rs
│
├── ui/                              # 🎨 前端(Vue 3)
│   ├── package.json
│   ├── pnpm-lock.yaml
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.ts
│       ├── App.vue
│       ├── router/
│       │   └── index.ts
│       ├── stores/                  # Pinia 状态
│       │   ├── collection.ts
│       │   ├── environment.ts
│       │   └── request.ts
│       ├── views/                   # 页面
│       │   ├── HomeView.vue         # 主界面(左侧集合,右侧请求)
│       │   ├── EnvironmentsView.vue # 环境管理
│       │   └── HistoryView.vue      # 历史
│       ├── components/              # 组件
│       │   ├── CollectionTree.vue    # 左侧集合树
│       │   ├── RequestEditor.vue    # 请求编辑区
│       │   ├── ResponseViewer.vue   # 响应展示
│       │   ├── KeyValueEditor.vue   # Header / Param 表格
│       │   └── BodyEditor.vue       # Body 编辑(JSON / form / raw)
│       ├── composables/             # 组合式函数
│       │   └── useInvoke.ts         # Tauri invoke 封装
│       └── styles/
│           └── global.css
│
└── scripts/                         # 🛠️ 开发脚本(可选)
    ├── dev.sh                       # 同时启动 rust watch + vite
    └── gen-icon.sh
```

### 设计要点
1. **`crates/core` 是纯 Rust 库**,可以单独跑测试,完全脱离 Tauri
2. **`crates/app` 只做"桥接"**:把 core 的能力封装成 Tauri Commands
3. **`ui/` 完全 TypeScript 类型化**,接口契约通过 `useInvoke` 统一
4. **Repository 模式**:数据库操作全部走 `storage/repo/`,业务层不直接碰 SQL

---

## 四、💾 数据模型设计

### SQLite 表结构

```sql
-- 集合(文件夹/分组)
CREATE TABLE collections (
    id          TEXT PRIMARY KEY,        -- uuid
    name        TEXT NOT NULL,
    description TEXT,
    parent_id   TEXT,                    -- 支持嵌套,null = 根
    sort_order  INTEGER DEFAULT 0,
    created_at  INTEGER NOT NULL,        -- unix timestamp
    updated_at  INTEGER NOT NULL
);

-- 请求
CREATE TABLE requests (
    id            TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL,
    name          TEXT NOT NULL,
    method        TEXT NOT NULL,         -- GET / POST / ...
    url           TEXT NOT NULL,
    headers       TEXT NOT NULL,         -- JSON: [{"key","value","enabled"}]
    query_params  TEXT NOT NULL,         -- JSON
    body_type     TEXT NOT NULL,         -- none / json / form / raw
    body_content  TEXT,
    auth_type     TEXT,                  -- none / bearer / basic / apikey
    auth_config   TEXT,                  -- JSON
    sort_order    INTEGER DEFAULT 0,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);

-- 环境
CREATE TABLE environments (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    is_active  INTEGER NOT NULL DEFAULT 0,  -- 只有一个能激活
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 环境变量
CREATE TABLE variables (
    id             TEXT PRIMARY KEY,
    environment_id TEXT NOT NULL,
    key            TEXT NOT NULL,
    value          TEXT NOT NULL,
    enabled        INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (environment_id) REFERENCES environments(id) ON DELETE CASCADE
);

-- 请求历史
CREATE TABLE history (
    id           TEXT PRIMARY KEY,
    request_id   TEXT,                  -- 关联原始请求(可空,原始请求删了历史还在)
    method       TEXT NOT NULL,
    url          TEXT NOT NULL,
    request_snapshot TEXT NOT NULL,     -- JSON,完整快照
    status_code  INTEGER,
    response_headers TEXT,             -- JSON
    response_body TEXT,
    duration_ms  INTEGER,
    error        TEXT,                  -- 网络错误信息
    sent_at      INTEGER NOT NULL,
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE SET NULL
);

CREATE INDEX idx_history_sent_at ON history(sent_at DESC);
CREATE INDEX idx_requests_collection ON requests(collection_id);
CREATE INDEX idx_variables_env ON variables(environment_id);
```

### 关键设计决策
- **环境变量激活**:用 `is_active` 单字段,激活时事务切换(只能一个激活)
- **请求体存 JSON 字符串**:简单、灵活,后期要换结构化只改解析层
- **历史快照**:存完整请求 + 响应,原始请求改了历史也不变,符合"历史"语义
- **不存外键字段**:Header / Param 等表用 JSON 存储,因为结构简单且不需要单独查询

---

## 五、🖼️ UI/UX 设计

### 主界面布局(单页)

```
┌─────────────────────────────────────────────────────────────────┐
│  [Logo] Rust API Holder     [Env: dev ▼] [⚙️] [📥Import] [📤Export]│  ← 顶栏
├──────────────┬──────────────────────────────────────────────────┤
│              │  ┌─Tabs─┐  GET /users/{id}                  [Send]│
│ 📂 Collections│  │Params│Headers│Body│Auth│                       │
│  📁 Dev      │  └──────┘                                          │
│   • GET /usr │  ┌──────────────────────────────────────────────┐ │
│   • POST /lg │  │ Key         │ Value                          │ │
│   • PUT /xxx │  │ page        │ 1                              │ │
│  📁 Prod     │  │ size        │ 20                             │ │
│   • ...      │  └──────────────────────────────────────────────┘ │
│              │                                                    │
│ 📜 History   │  ─── Response ────────────────────────── 200 32ms ─│
│  • /users  1m│  │ Body │ Headers │                                  │
│  • /login  5m│  │ {                                                       │
│              │  │   "id": 1,                                              │
│              │  │   "name": "alice"                                       │
│              │  │ }                                                       │
├──────────────┴──────────────────────────────────────────────────┤
│ Status: ready                                          v0.1.0   │  ← 状态栏
└─────────────────────────────────────────────────────────────────┘
```

### 交互要点
- **快捷键**:`Ctrl/Cmd + Enter` 发送、`Ctrl/Cmd + S` 保存、`Ctrl/Cmd + K` 命令面板
- **变量插值**:`{{baseUrl}}/users/{{userId}}` 自动渲染
- **响应 JSON 美化**:折叠/展开节点、复制路径、搜索
- **未保存提示**:右上角小圆点 + 快捷键 `Ctrl+S`

---

## 六、📋 MVP 功能清单(必须完成)

| # | 功能 | 优先级 | 说明 |
|---|---|---|---|
| F1 | 创建/重命名/删除集合与文件夹 | P0 | 支持嵌套 |
| F2 | 创建/编辑/删除请求 | P0 | 基本 CRUD |
| F3 | 发送 HTTP 请求(GET/POST/PUT/PATCH/DELETE) | P0 | |
| F4 | Headers / Query / Body 编辑 | P0 | Body 支持 none / json / form / raw |
| F5 | Auth(Bearer / Basic / API Key) | P0 | |
| F6 | 响应展示(Status / Headers / Body / 耗时) | P0 | JSON 自动美化 |
| F7 | 环境变量 CRUD + 切换环境 | P0 | `{{var}}` 插值 |
| F8 | 请求历史记录 | P1 | 自动保存、可点击复用 |
| F9 | 导入 Postman v2.1 Collection JSON | P1 | |
| F10 | 导出为 Postman 格式 | P2 | 锦上添花 |

**v1 完成后,以下功能进入 Backlog**(后续版本考虑):
- WebSocket / SSE
- 请求脚本(Pre-request / Post-response)
- 全局变量
- 主题切换(深/浅)
- 多窗口 / 多 Tab
- 加密存储

---

## 七、🗓️ 开发路线图(9-10 周,并行模式)

### ⏱️ 时间预算
- 周内:每天 **1 小时** × 5 = **5h**
- 周末:每天 **2 小时** × 2 = **4h**
- **合计:9h / 周**(给后端)
- 前端(我负责)无时间上限,可以提前完成

### 🔀 并行策略
核心思路:**你专心 Rust,我提前写前端,集成期再合并**。

```
你的时间线:
W1 ──→ W2 ──→ W3 ──→ W4 ──→ W5 ──→ W6 ──→ W7 ──→ W8 ──→ W9
基础设施  HTTP    存储   桥接    联调    环境    导入    打磨    发布

我的时间线(并行):
W1 ──→ W2 ──→ W3 ──→ W4 ──→ W5 ──→ W6 ──→ W7 ──→ W8 ──→ W9
UI骨架   编辑器  树组件   集成    联调    历史UI  导入UI  细节   演示
```

集成点:**Week 4 末**(前端等你给出第一个 Tauri Command 签名)、**Week 5-9**(每周一次小集成)。

---

### 📅 Week 1(9h):基础设施搭建

**目标**:能弹出空 Tauri 窗口 + 工作区结构清晰

| 👤 你 | 🤖 我 |
|---|---|
| 安装 Rust / Node / pnpm / tauri-cli | 初始化 `ui/`(Vue 3 + Vite + TS + Naive UI) |
| `cargo new --workspace` 双 crate 骨架 | 搭三栏布局骨架(占位数据) |
| 配 `tauri.conf.json`、最小 `main.rs` | 写 `useInvoke` Tauri 调用封装 |
| 跑通 `pnpm tauri dev` 空窗口 | 提交第一个前端 PR |

**Rust 学习重点**:所有权/借用、Cargo workspace、crate / module

**验收**:弹窗 + DevTools 能看到 "Hello from Rust!"

---

### 📅 Week 2(9h):核心 HTTP 与配置层

**目标**:`crates/core` 能独立发 HTTP 请求 + 变量插值

| 👤 你 | 🤖 我 |
|---|---|
| `core::http::Request` / `Response` 数据结构 | `RequestEditor.vue` 组件框架 |
| `reqwest` 封装(方法/headers/body/auth) | `KeyValueEditor.vue` 通用组件 |
| `{{var}}` 变量插值(纯字符串) | `BodyEditor.vue`(按 body_type 切换) |
| 单元测试(变量插值 + 各 HTTP 方法) | `MethodSelect.vue`、`UrlBar.vue` |

**Rust 学习重点**:async/await、Result 与 `?`、derive、trait 入门

**验收**:`cargo test` 全绿,有一个集成测试能请求 httpsbin.org

---

### 📅 Week 3(9h):SQLite 存储层

**目标**:CRUD 全部跑通,有 schema migration

| 👤 你 | 🤖 我 |
|---|---|
| 集成 `rusqlite`(bundled 模式) | `CollectionTree.vue` 递归组件 |
| 5 张表建表 SQL + 索引 | Pinia stores(collection / request) |
| Repository 模式 × 4 模块 | 类型定义(对照你给的接口) |
| 数据库初始化(应用数据目录) | `Header` / `Sidebar` 样式收尾 |
| 集成测试:每个 repo 跑一遍 | |

**Rust 学习重点**:FFI(首次编译慢)、SQL 基础、错误处理(thiserror)、泛型

**验收**:`cargo run --example seed_data` 能写入示例数据并读回

---

### 📅 Week 4(9h):🔗 Tauri 桥接 + UI 收尾(集成点 ①)

**目标**:前端能从后端拿到真实数据,看到集合树

| 👤 你 | 🤖 我 |
|---|---|
| 把 core 能力封装为 Tauri Commands | 接入 Pinia stores 调用真实接口 |
| 实现 `capabilities/default.json` | 完整主界面(请求编辑 + 响应展示) |
| 错误传递(AppError → JSON) | 主题适配、Naive UI 视觉 |
| 给我 Command 签名清单 | 响应占位 + Loading |

**Rust 学习重点**:序列化边界、错误传递、async 命令

**验收**:左侧集合树显示真实数据库内容,点击节点右侧加载

> ⚠️ **关键里程碑**:本周如果跑通,后面就是"功能堆叠";如果不顺,可能要 +1 周缓冲

---

### 📅 Week 5(9h):🚀 核心交互闭环

**目标**:完整跑通"选请求 → 改 → 发 → 看响应"

| 👤 你 | 🤖 我 |
|---|---|
| 修桥接过程的 bug | 接 `ResponseViewer` 真实响应 |
| 完善 Command 错误处理 | JSON 美化 + `highlight.js` |
| 性能优化(连接复用?) | 发送按钮 Loading + Toast |
| | Ctrl+Enter / Ctrl+S 快捷键监听 |

**Rust 学习重点**:性能调优入门、tokio 连接池

**验收**:能完整跑一次"打开 → 选请求 → 发送 → 看响应"

---

### 📅 Week 6(9h):✨ 环境变量 + 历史

**目标**:补齐 MVP 第 6/8 项

| 👤 你 | 🤖 我 |
|---|---|
| `core::environment`(CRUD + 激活切换) | `EnvironmentsView.vue` |
| 变量插值引擎接入 HTTP 层 | 顶栏环境选择器 |
| `core::history`(自动入库 + 查询) | `HistoryView.vue` |
| | 点击历史 → 回填编辑器 |

**Rust 学习重点**:事务、上下文传递、迭代器链

**验收**:切环境 → 变量自动更新;发请求 → 历史入库

---

### 📅 Week 7(9h):📥 Postman 导入 + 快捷键完善

**目标**:能导入 Postman 集合,基础快捷键可用

| 👤 你 | 🤖 我 |
|---|---|
| `core::import::postman` 解析器 | 导入对话框 UI |
| 快捷键事件接入(全局) | 文件选择 + 进度展示 |
| | 错误提示(格式不符等) |

**Rust 学习重点**:serde 第三方 JSON 解析、模式匹配

**验收**:导入一份 Postman 集合 → 全部请求出现在左侧

---

### 📅 Week 8(9h):🎨 打磨

**目标**:让产品"能拿出手"

| 👤 你 | 🤖 我 |
|---|---|
| release profile 优化 | 空状态、错误页 |
| tracing 日志完善 | 动效、过渡、加载占位 |
| 边界 case(空集合、错误响应) | 视觉一致性 review |
| | README 截图 |

**验收**:连续操作 10 次无明显卡顿或崩溃

---

### 📅 Week 9(9h):📦 发布

**目标**:产出可分发的安装包

| 👤 你 | 🤖 我 |
|---|---|
| `tauri.conf.json` 打包配置 | 录 GIF 演示 |
| GitHub Actions CI(可选) | 写 README + 使用文档 |
| 测试各平台安装包 | 创建 Release |

**验收**:macOS / Windows / Linux 至少 1 个平台有可安装包

---

### 📅 Week 10(缓冲周,可选)

如果前 9 周遇到 Rust 学习卡壳或 Tauri 集成不顺:
- Week 10 用作"修复 + 重做 + 缓冲",不开始新功能

> 💡 **学习节奏预估**:Rust 初学者前 3 周可能慢一些(borrow checker),Week 4 之后会顺。Week 5-7 主要做 CRUD 类工作,概念密度低。Week 8-9 主要是工程化,反而轻松。

---

## 八、📚 学习路线(贯穿全程)

### Rust 核心知识点地图

按周对应要重点掌握的概念:

```
Week 1 → 所有权/借用/生命周期基础、crate / module、Cargo workspace
Week 2 → 异步(async/await)、Result 与 ?、derive 宏、trait 基础
Week 3 → 错误处理(thiserror)、泛型、Sender/Receiver、生命周期进阶
Week 4 → serde 序列化、Option 处理、闭包
Week 5 → trait 对象、Box<dyn>、智能指针
Week 6 → 迭代器链、模式匹配、字符串处理
Week 7 → 性能分析、profiling、build profile
```

### 推荐配套学习资源
- 📖 **The Rust Book**(官方):https://doc.rust-lang.org/book/(必读)
- 📖 **Rust by Example**:https://doc.rust-lang.org/rust-by-example/(示例驱动)
- 🎥 **Rust 语言圣经**:https://course.rs/(中文友好)
- 🛠️ **Tauri 2 官方教程**:https://v2.tauri.app/start/
- 🛠️ **Tauri 实战**:GitHub 上搜 `tauri-vue-template` 参考

### 踩坑预警(提前打预防针)
| 坑 | 现象 | 对策 |
|---|---|---|
| 编译慢 | Tauri 首次编译 10+ 分钟 | 用 `cargo watch`,不全量重编 |
| Borrow checker 报错 | 函数签名调来调去 | 别急着绕,先理解生命周期 |
| Tauri 2 权限配置 | `deny(unsafe_access)` | `capabilities/default.json` 里加权限 |
| 异步锁死 | `.await` 时持锁 | 用 `tokio::sync::Mutex` |
| JSON 解析失败 | Postman 字段缺失 | serde 默认值 + 优雅降级 |

---

## 九、⚠️ 风险与对策

| 风险 | 等级 | 影响 | 对策 |
|---|---|---|---|
| Rust 学习曲线陡,卡壳影响进度 | 🔴 高 | 项目停滞 | 每周回顾,卡 >2 天就求助社区/降级方案 |
| Tauri 2 文档相对薄 | 🟡 中 | 调试耗时 | 借助 GitHub Issues / Discord |
| Vue / Tauri 双向类型同步 | 🟡 中 | 接口不对 | 用 `specta` 等工具自动生成 TS 类型 |
| macOS 签名 / 公证 | 🟢 低 | 发布卡住 | 暂时只发布未签名包,后续再补 |

---

## 十、✅ Week 1 立即行动清单

### 你本周的任务(每日 1h,周末 2h)
- [ ] **Day 1(周一)** 装 Rust:`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [ ] **Day 1** 装 Node 20+、pnpm(`npm i -g pnpm`)
- [ ] **Day 1** 装 tauri-cli:`cargo install tauri-cli --version "^2.0"`(耗时长,可放后台)
- [ ] **Day 2** `git init`,在 GitHub 创建 repo(可选)
- [ ] **Day 2** 跑通 hello world:`pnpm create tauri-app`(选 Vue + TS),`pnpm tauri dev` 看到窗口
- [ ] **Day 3-4** 在已跑通的 tauri-app 基础上,改成 workspace 结构(`crates/core` + `crates/app`)
- [ ] **Day 5(周五)** 写一个最小的 `reqwest` demo(脱离 Tauri,纯 Rust binary 发 GET 请求)
- [ ] **周末** 整理代码、写 `.gitignore`、提交第一个 commit

### 我(AI)同步做的事(本周任何时候)
- 初始化 `ui/` 完整骨架(三栏布局 + Naive UI + Pinia)
- 写 `useInvoke` Tauri 调用封装
- 写第一版 TypeScript 类型契约
- 把代码放到 `ui/` 目录,你按需浏览

### 工具与命令速查
```bash
# 安装
brew install rustup-init   # macOS
rustup default stable
node --version              # 需 ≥ 20
npm i -g pnpm
cargo install tauri-cli --version "^2.0"

# 初始化(可以先尝试用官方模板)
pnpm create tauri-app

# 运行
pnpm tauri dev
```

---

## 十一、👥 角色分工与协作流程

### 🎯 分工原则
| | 你(后端) | 我(AI 协作 / 前端) |
|---|---|---|
| **负责** | 所有 Rust 代码 | 所有 Vue 3 + TS 代码 |
| **产出** | `crates/core` `crates/app` `tauri.conf.json` SQL | `ui/` 目录下所有文件 |
| **关注** | Rust 学习、核心逻辑、数据库 | UI 体验、组件、TS 类型 |
| **决策** | 数据模型、错误处理策略 | UI 交互、视觉细节 |

### 🔄 协作流程(每周一次)

**每周五晚上 / 周六上午**(任选固定时间):
1. **你** commit 本周 Rust 代码,简单告诉我这周完成了什么
2. **你** 列出"下周需要前端配合什么"(通常是新的 Command 签名)
3. **我** 提交本周前端代码 + 下周 Command 接口类型定义
4. **我** 把前端代码放到对应位置,你 review 或合并(可选)

### 🚧 边界与约定
- ❌ **我不修改 `crates/` 下的任何 Rust 文件**(避免破坏你的所有权设计)
- ✅ **我不修改 SQL 决策**(以你的数据模型为准)
- ⚠️ 如果你发现前端类型定义错了,告诉我,我立刻改
- ⚠️ 如果你改了 Command 签名,**必须同步告诉我**,否则前端调不通

### 📬 沟通清单(每个 Stage 切换时)
你交付后端时,需要告诉我:
```
✅ 本周完成:crates/core 增加了什么模块
🔜 下周我会新增的 Tauri Command:
  - name: create_collection
    args: { name: string, parent_id?: string }
    returns: Collection
  - ...
📝 任何会影响前端的接口变更
```

### 🤖 我会负责的事(不让你操心)
- ✅ 所有 Vue 组件、Pinia stores、Vue Router
- ✅ Tauri invoke 封装、TypeScript 类型契约
- ✅ Naive UI 主题、视觉一致性
- ✅ 前端构建配置、性能优化
- ✅ UI 自测(`pnpm typecheck` + 本地视觉确认)

### 🎓 你的专注点
- ✅ Rust 基础学习(所有权、async、错误处理)
- ✅ `crates/core` 业务逻辑
- ✅ `crates/app` 的 Tauri Commands
- ✅ 数据库 schema 和迁移

---

## 十二、❓ 还有 2 个问题确认下

1. **周回顾时间**:每周五晚 / 周六上午,定一个固定时间做同步(我们交接代码 + 对齐下周),你方便吗?

2. **Git 协作方式**:
   - 选项 A:同一 repo,你直接 commit Rust,我把前端文件以"代码块 / 完整文件"形式贴给你,你粘贴进 `ui/`
   - 选项 B:你建好 repo 给我协作权限,我直接 push 前端(但这需要我能访问你的 GitHub)
   - 你倾向哪种?

---

> 📌 本文档会随着开发推进持续更新。
> 一旦你确认这版合理 + 上面 2 个问题的答案,我们就开始 **Week 1 的第一个 commit** 🚀