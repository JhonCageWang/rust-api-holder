# 🦀 Rust API Holder · 后端开发指南(给 wzy)

> 这是**给你一个人**的路线图。`PLAN.md` 是项目整体计划,这份只关注你的部分:
> - `crates/core/` 的实装
> - `crates/app/` 的 Tauri Commands
> - 你应该什么时候做什么、怎么写、怎么测

---

## 一、📦 我已经帮你写好的代码(2026-08-21 至今)

我先把**项目骨架 + 部分核心模块**写完了,你不必从零开始。先理解这些,**再开始你的工作**。

### ✅ 完全实装 + 测试(可以直接用,不用动)

| 文件 | 内容 | 行数 | 测试数 |
|---|---|---|---|
| `crates/core/src/error.rs` | `Error` 枚举 + `Result` 别名 + `thiserror` 派生 | 72 | 2 |
| `crates/core/src/environment/mod.rs` | `Environment` 模型 + **`interpolate()` 变量插值**(完整实装) | 92 | 3 |
| `crates/core/src/storage/mod.rs` | `Database` 封装(SQLite + Mutex) | 84 | 2 |
| `crates/core/src/storage/migrations.rs` | Migration 系统 + v1 schema(5 张表 + 索引) | 164 | 2 |
| `crates/core/src/lib.rs` | crate 入口 + `pub use error::{Error, Result}` | 32 | 0 |
| `crates/app/src/main.rs` | Tauri Builder + tracing + AppState | 72 | 0 |
| `crates/app/src/commands/mod.rs` | `ping` + `app_info` 两个 Command | 45 | 0 |

### 🟡 搭好骨架,但逻辑是 `todo!()`(需要你去填)

| 文件 | 现状 | 你要做的事 |
|---|---|---|
| `crates/core/src/http/mod.rs` | `Request`/`Response`/`Method`/`Auth`/`KeyValue`/`Body` 结构 + 测试 | **实装 `execute()` 函数** — Week 2 |
| `crates/core/src/collection/mod.rs` | `Collection` / `RequestItem` / `NewCollection` 结构 + 1 测试 | 加 `NewRequestItem` + Repository 引用 |
| `crates/core/src/history/mod.rs` | `HistoryEntry` 结构 + 1 测试 | 加 snapshot 序列化 + Repository 引用 |
| `crates/core/src/import/mod.rs` | 完全是占位 `todo!()` | Week 7:Postman JSON 解析 |

### 📋 配置类文件(无需修改)

- 根 `Cargo.toml`(workspace + 共享依赖)
- `crates/{core,app}/Cargo.toml`
- `crates/app/build.rs`
- `crates/app/tauri.conf.json`
- `crates/app/capabilities/default.json`
- `crates/app/icons/icon.png`

---

## 二、📊 模块成熟度矩阵

```
✅ = 完成   🟡 = 部分   ⬜ = 待办   — =  不适用
```

| 模块 | 模型 | 业务逻辑 | 测试 | 文档 | 你下周的优先级 |
|---|---|---|---|---|---|
| `error` | ✅ | ✅ | ✅ 2 | ✅ | — |
| `environment/interpolate` | ✅ | ✅ | ✅ 3 | ✅ | — |
| `storage/Database` | ✅ | ✅ | ✅ 2 | ✅ | — |
| `storage/migrations` | ✅ | ✅ | ✅ 2 | 🟡 | — |
| `http/Request` | ✅ | — | ✅ 2 | ✅ | — |
| **`http/execute`** | ✅ | **⬜ todo!** | ⬜ | ✅ | **🔴 Week 2 起点** |
| `collection` | ✅ | ⬜ | 🟡 1 | 🟡 | Week 3 |
| `environment/CRUD` | ⬜ | ⬜ | ⬜ | ⬜ | Week 3 + Week 6 |
| `history` | ✅ | ⬜ | 🟡 1 | 🟡 | Week 6 |
| `import` | ⬜ | ⬜ | ⬜ | ⬜ | Week 7 |
| Tauri Commands | 🟡 | 🟡 2 | — | 🟡 | Week 4 + 持续 |
| `storage/repo/*` | ⬜ | ⬜ | ⬜ | ⬜ | **Week 3 重点** |

---

## 三、🎯 你的实施路线(从今天开始)

```
Week 2        Week 3          Week 4         Week 5-6         Week 7
   │            │               │              │               │
HTTP 模块   Repository 层    Tauri 桥接    环境/历史实装   Postman 导入
   │            │               │              │               │
   ▼            ▼               ▼              ▼               ▼
execute()   5 个 Repository   6 个 Command   完整功能      解析器
+ 变量插值   + 集成测试       + 错误传递     + 优化         + 测试
```

---

## 四、📝 Week 2:HTTP 模块(从这里!)

### 目标
让 `core/src/http/mod.rs` 的 `execute()` 函数真正能发请求,带变量插值,并有完整测试。

### 任务清单

#### 1️⃣ 看懂现有代码(1 小时,周一晚)
- [ ] 通读 `http/mod.rs`,理解 `Request`/`Response`/`Method`/`Auth` 结构
- [ ] 跑 `cargo test -p api-holder-core http` 看现有测试
- [ ] 看 `http/mod.rs` 顶部的 doc comment,理解设计意图

#### 2️⃣ 实现基础 execute()(1.5 小时,周二)
**在 `http/mod.rs` 替换 `pub async fn execute(_req: Request) -> Result<Response>`**:

```rust
// 你的目标代码结构
use std::time::Instant;

pub async fn execute(req: Request) -> Result<Response> {
    // 1. 用 reqwest::Client::new() 建 client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // 2. 构造 reqwest::RequestBuilder
    let mut builder = client.request(req.method.to_reqwest(), &req.url);

    // 3. 应用 headers / query / auth / body
    apply_headers(&mut builder, &req.headers);
    apply_query(&mut builder, &req.query);
    apply_auth(&mut builder, &req.auth);
    apply_body(&mut builder, &req.body)?;

    // 4. 发送并测量时间
    let start = Instant::now();
    let resp = builder.send().await?;
    let duration = start.elapsed();

    // 5. 把 reqwest::Response 转成我们的 Response
    to_response(resp, duration).await
}
```

**Rust 学习重点**:
- `async/await` — `fn execute() -> impl Future` 自动推导
- `?` 操作符 — `Result` 的链式传播
- `reqwest` API:`Client::builder()`, `RequestBuilder`

#### 3️⃣ 加 helper 函数(30 分钟)
需要写(参考 reqwest 文档):
- `apply_headers(builder, headers)` — 遍历 Vec<KeyValue>,用 `builder.header(k, v)`
- `apply_query(builder, queries)` — 用 `builder.query(&[(k, v)])` 或循环
- `apply_auth(builder, auth)` — match 各种 Auth 变体
- `apply_body(builder, body)` — match Body 变体,设置 `body()` 或 `json()`
- `to_response(resp, duration)` — 读 status / headers / text(), 包装成 Response

#### 4️⃣ 加 3-4 个测试(1 小时)
```rust
#[tokio::test]
async fn test_execute_get_success() {
    let req = Request {
        method: Method::Get,
        url: "https://httpbin.org/get".into(),
        // ...
    };
    let resp = execute(req).await.unwrap();
    assert_eq!(resp.status, 200);
}

#[tokio::test]
async fn test_execute_with_query_params() {
    // 测 query 参数传递
}

#[tokio::test]
async fn test_execute_404() {
    // 测 404 响应
}
```

**Rust 学习重点**:`#[tokio::test]` vs `#[test]`(前者需要 tokio runtime)

#### 5️⃣ 集成:变量插值(45 分钟)
`http/execute` 应该在构造 URL 前调用 `environment::interpolate`。

设计思路:
- 让 `execute()` 接受可选的 `vars: HashMap<String, String>`
- 在内部对 `req.url` 和所有 header/query value 跑 `interpolate()`
- 然后再发请求

```rust
pub async fn execute(req: Request, vars: Option<&HashMap<String, String>>) -> Result<Response> {
    let req = apply_interpolation(req, vars);
    // 然后发请求
}
```

#### 6️⃣ 跑通验证(30 分钟)
```bash
cargo test -p api-holder-core http    # 单元测试
cargo test --workspace                 # 全部测试
cargo clippy --workspace              # lint
```

### Week 2 验收标准
- ✅ `cargo test --workspace` 全绿
- ✅ 至少 6 个 http 相关测试通过
- ✅ `cargo clippy` 0 警告(可选)
- ✅ commit message:`feat(core): implement http execution with variable interpolation`

---

## 五、📝 Week 3:存储层(Repository)

### 目标
为 5 个表各写一个 Repository,提供 CRUD + 查询。

### 任务清单

#### 1️⃣ 看懂 storage 现有代码(1 小时)
- [ ] 看 `storage/mod.rs` 的 `Database` 封装
- [ ] 看 `storage/migrations.rs` 的 5 张表结构
- [ ] 跑 `cargo test -p api-holder-core storage` 看现有测试

#### 2️⃣ 创建 repo 子模块结构(30 分钟)
创建以下文件:
```
crates/core/src/storage/
├── mod.rs              (已存在,加 pub mod repo;)
├── db.rs               ← 把 Database 移到这(可选)
├── migrations.rs       (已存在)
└── repo/
    ├── mod.rs          (新建)
    ├── collection.rs   (新建)
    ├── request.rs      (新建)
    ├── environment.rs  (新建)
    └── history.rs      (新建)
```

#### 3️⃣ 实现 Collection Repository(2 小时)
**在 `repo/collection.rs`**:
```rust
use crate::Result;
use crate::collection::{Collection, NewCollection};
use super::Database;

pub struct CollectionRepo<'a> {
    db: &'a Database,
}

impl<'a> CollectionRepo<'a> {
    pub fn new(db: &'a Database) -> Self { Self { db } }

    pub fn create(&self, new: NewCollection) -> Result<Collection> {
        todo!()
    }

    pub fn list(&self) -> Result<Vec<Collection>> {
        todo!()
    }

    pub fn get(&self, id: &str) -> Result<Option<Collection>> {
        todo!()
    }

    pub fn update(&self, id: &str, name: Option<&str>) -> Result<Collection> {
        todo!()
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        todo!()
    }
}
```

**Rust 学习重点**:
- `&'a Database` — 生命周期标注,Repository 借用 Database
- `rusqlite`:`Connection::prepare` / `Statement::query_map` / `Row::get`
- `Error::NotFound` / `Error::InvalidInput` 用起来

#### 4️⃣ 其他 Repository(类似,各 1.5 小时)
按相同模式写 `request.rs`、`environment.rs`、`history.rs`。

**特别注意**:
- `environment.rs` 的"激活切换"是事务:`UPDATE environments SET is_active = 0; UPDATE ... SET is_active = 1 WHERE id = ?`
- `history.rs` 不需要 update / delete,只需 create + list + get
- `request.rs` 的 body/headers/auth 要 JSON 序列化后存 TEXT 字段

#### 5️⃣ 集成测试(2 小时)
**在 `crates/core/tests/repo_integration.rs`**(新建):
```rust
use api_holder_core::storage::Database;
use api_holder_core::storage::repo::CollectionRepo;
use api_holder_core::collection::NewCollection;

#[test]
fn test_collection_crud() {
    let db = Database::open_in_memory().unwrap();
    let repo = CollectionRepo::new(&db);

    let created = repo.create(NewCollection {
        name: "Test".into(),
        description: None,
        parent_id: None,
    }).unwrap();

    assert_eq!(created.name, "Test");

    let list = repo.list().unwrap();
    assert_eq!(list.len(), 1);

    repo.delete(&created.id.to_string()).unwrap();
    assert_eq!(repo.list().unwrap().len(), 0);
}
```

### Week 3 验收标准
- ✅ 5 个 Repository 都有 CRUD
- ✅ `cargo test --workspace` 全绿(包括集成测试)
- ✅ commit:`feat(core): add Repository layer for all entities`

---

## 六、📝 Week 4:Tauri Commands 桥接

### 目标
把 core 能力暴露给前端,通过 Tauri Commands。

### 任务清单

#### 1️⃣ 看现有 commands(30 分钟)
- [ ] 读 `crates/app/src/commands/mod.rs`
- [ ] 看 `main.rs` 里 `invoke_handler` 注册

#### 2️⃣ 加 commands(每个 30-60 分钟)
**至少 6 个**:

```rust
// commands/collection.rs
#[tauri::command]
pub fn list_collections(state: State<AppState>) -> Result<Vec<Collection>, String> {
    // 用 state.db 调 repo
    // 错误转 String(给前端)
}

#[tauri::command]
pub fn create_collection(
    name: String,
    parent_id: Option<String>,
    state: State<AppState>,
) -> Result<Collection, String> {
    // ...
}
```

命令清单:
1. `list_collections`
2. `create_collection`
3. `update_collection`
4. `delete_collection`
5. `list_requests(collection_id)`
6. `get_request(id)`

**Rust 学习重点**:
- `State<AppState>` — Tauri 状态注入
- `Result<T, String>` — 错误必须能序列化给前端
- `#[tauri::command]` 宏

#### 3️⃣ 在 main.rs 注册(15 分钟)
```rust
.invoke_handler(tauri::generate_handler![
    commands::ping,
    commands::app_info,
    commands::list_collections,    // ← 加
    commands::create_collection,   // ← 加
    // ...
])
```

#### 5️⃣ **关键**:同步告诉前端类型契约!
每加一个 command,你需要告诉我:
```
✅ 本周完成:crates/app 加了 collection.rs(6 个 commands)
🔜 新增的 Command 签名:
  - name: list_collections
    args: undefined
    returns: Collection[]
  - name: create_collection
    args: { name: string, parent_id?: string }
    returns: Collection
```

我会在 `ui/src/composables/useInvoke.ts` 里加对应类型,前端就能用。

### Week 4 验收标准
- ✅ 至少 6 个 Command 实装
- ✅ commit 后我立刻接上前端
- ✅ 端到端:启动 Tauri,左侧集合树显示真实数据

---

## 七、📝 Week 5-9 简版

| Week | 重点 | 详细任务 |
|---|---|---|
| **5** | 联调 + 请求执行 | `execute_request` Command(收 Collection/Request + vars → 调 http::execute → 写 history) |
| **6** | 环境 + 历史 | `EnvironmentRepo::activate`(事务) + `HistoryRepo::list_recent(50)` |
| **7** | Postman 导入 | `import/mod.rs` 实装 `import_postman_collection(json)` → 批量 insert |
| **8** | 打磨 | `release` profile、tracing 日志完善、错误处理统一 |
| **9** | 发布 | `tauri.conf.json` 打包配置 + 文档 + Release |

---

## 八、📚 Rust 学习卡点速查(按场景触发)

### 写 `async fn` 时
- 函数签名:`pub async fn foo() -> Result<T>`
- 调用方:`let x = foo().await?;`
- 测试:`#[tokio::test]`(需要 dev-dependencies 里的 `tokio` + `macros`)

### 写 Repository 时
- `let conn = self.db.with_conn(|c| { ... })?;` ← 拿连接的固定模式
- SQL 用 `rusqlite::params!["...", value]`

### 写 Tauri Command 时
- 状态:`state: State<'_, AppState>`
- 错误:必须 `Result<T, String>`,错误转 `e.to_string()`
- 异步:`async` Command 用 `tokio::spawn` 跑重活

### 用 serde 时
- `#[derive(Serialize, Deserialize)]` 加字段
- 默认值:`#[serde(default)]` 或 `#[serde(default = "fn_name")]`
- 重命名:`#[serde(rename = "...")]`

### 编译报错时
**贴错误给我**(包含 `error[E0XXX]` 那几行),不要一个人啃。

---

## 九、🌿 Git 工作流

### Commit 风格(Conventional Commits)
```
feat(core): add Repository layer for all entities
fix(http): handle empty body correctly
docs: update DEV_GUIDE with Week 4 details
chore: bump rusqlite to 0.32
test(storage): add integration tests for collection repo
```

### 每完成一个 Stage,做一次 commit
- Week 2 完成 → 1 commit(`feat(core): implement http execution`)
- Week 3 完成 → 1 commit(`feat(core): add Repository layer`)
- Week 4 完成 → 1 commit(`feat(app): add Tauri commands for collections`)

### Commit 之前先跑测试
```bash
cargo test --workspace
cargo clippy --workspace    # 可选,但推荐
```

---

## 十、📞 协作 Checklist(每周日上午同步时)

准备告诉我:
```
✅ 本周完成:
  - 实装了 X 模块的 Y 函数
  - 加了 Z 个测试,全部通过
  - commit: <hash> <message>

🔜 下周 Tauri Commands(我需要这些签名来写前端):
  - name: ...
    args: ...
    returns: ...

❓ 遇到的问题 / 学到的概念(我帮你复盘):
  - borrow checker 报错 X,我现在懂了 Y
  - async / await 的 Z 点还不清楚

📆 下周计划:
  - Week N: 做 ABC
```

---

## 十一、🚀 现在就开干(Week 2 Day 1 任务)

**本周投入 9 小时,目标:HTTP 模块完整实装**

### Day 1(周一,1 小时)
- [ ] 通读 `crates/core/src/http/mod.rs` + `environment/mod.rs` + `error.rs`
- [ ] 跑 `cargo test -p api-holder-core` 看现有测试
- [ ] 跑 `cargo doc -p api-holder-core --open` 看模块文档

### Day 2-3(周二/三,各 1.5 小时)
- [ ] 在 `http/mod.rs` 写 `execute()` 主函数
- [ ] 写 5 个 helper 函数

### Day 4-5(周四/五,各 1.5 小时)
- [ ] 加 3-4 个 `#[tokio::test]` 测试
- [ ] 集成 `interpolate()` 到 `execute()`
- [ ] 跑 `cargo test --workspace`,全绿
- [ ] **commit** `feat(core): implement http execution with variable interpolation`

### 周末(4 小时)
- [ ] 跟我同步 Week 2 成果
- [ ] 我会帮你看 `execute()` 的设计,提改进建议
- [ ] 一起规划 Week 3 (Repository)

---

## 十二、❓ 你现在该做的事

**就 4 步,15 分钟搞定**:

1. [ ] 把这份 `DEV_GUIDE.md` 看一遍(尤其是第二/三/四节)
2. [ ] 跑 `cargo test --workspace`,确认现有 9 个测试全过
3. [ ] 跑 `cargo doc -p api-holder-core --no-deps --open`,浏览模块文档
4. [ ] 确认 Tauri 窗口在你桌面,然后告诉我:**"开干 Week 2"**

之后我们按 Week 2 Day 1-5 节奏推进,周日同步 📅

---

> 📌 记住:**卡壳就贴错误给我**,不要一个人死磕。
> 你的 Rust 学习曲线 ≈ 项目进度,**两者一起成长**才健康。