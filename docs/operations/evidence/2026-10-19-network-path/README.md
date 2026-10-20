# 真实网络路径：**路由可抽取**（2026-10-19）

- 工作包：**NC08 / 产品级长程（网络面）**；规则版本 v322（工作树）/ 存档格式 v105

## 1 本轮改动（**已跟踪文件**）

`apps/game-server/src/main.rs`：把内联的 `Router::new()…` 抽成

```rust
fn build_router(lobby: Arc<Lobby>, original_content: ServeDir) -> Router { … }
```

`main()` 现在只负责：装配 → `build_router` → `TcpListener::bind` → `axum::serve`。

**动机**：此前路由**在 `main()` 内部构造**，测试**无法**在 `127.0.0.1:0` 上起真实监听 ⇒
「真实网络往返」这一面**没有任何可复现证据**。

## 2 做到了什么 / 没做到什么（**如实**）

| 项 | 状态 |
|---|---|
| 路由可从测试侧构造 | ✅ **已抽取**（`build_router`，`cargo build -p game-server` 通过）|
| 真实监听 + HTTP 往返测试 | ❌ **未做** |
| WS 握手/回路测试 | ❌ **未做** |

## 3 为什么**还**写不出网络测试（**精确前置条件**）

`Lobby::new` 需要 **6 个装配参数**：

```rust
Lobby::new(cfg, engine_template, content_lock, content_catalog, content_assets, Arc::new(native_content))
```

其中 `engine_template` / `content_lock` / `content_catalog` / `content_assets` / `native_content`
都在 `main()` 里**就地装配**（读配置、import 扩展、构建目录…）⇒ 测试侧**没有现成装配器**。

**下一步（机械可做）**：把 `main()` 里「装配 Lobby」那段抽成

```rust
pub(crate) fn bootstrap_lobby(cfg: ServerConfig) -> Result<Arc<Lobby>, String>
```

然后测试即可：`build_router(bootstrap_lobby(test_cfg)?, ServeDir::new(tmp))` +
`TcpListener::bind("127.0.0.1:0")` + `TcpStream` 发一条**裸 HTTP/1.1** `GET /health`
（**零新依赖**）⇒ 形成**真实 socket 往返**证据。

## 4 环境限制（**实测**）

本环境 **`/bin/ps` 被沙箱拒绝**（`Operation not permitted`，见[内存证据](../2026-10-19-product-memory/README.md)）；
**监听取证未尝试**（避免在无明确需要时起服务）。
⇒ 「真实网络」仍属**未验收**，本轮只**移除了一项结构性障碍**。
