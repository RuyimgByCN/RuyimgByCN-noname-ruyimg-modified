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

## 1.1 ✅ 第二轮：**真实 socket 往返已跑通**

把 `main()` 里装配 Lobby 的那段抽成：

```rust
fn bootstrap(cfg: &ServerConfig) -> Arc<Lobby>   // 扩展导入 → 内容锁 → 目录/资产 → 原生内容 → Lobby
```

并在 `main.rs` 内新增 `#[cfg(test)] mod net_smoke`：**起真实 `TcpListener("127.0.0.1:0")` + `axum::serve`**，
用 **`std::net::TcpStream` 发裸 HTTP/1.1**（**零新依赖**）打两个端点：

```
DBG 真实网络往返：127.0.0.1:57632 -> HTTP/1.1 200 OK
DBG 真实网络往返（有状态端点 /metrics）：127.0.0.1:57632 -> HTTP/1.1 200 OK；body_len=877
test result: ok. 1 passed; 0 failed; 0 ignored
```

| 判据 | 结果 |
|---|---|
| 真实 TCP 监听（**临时端口**）| ✅ `127.0.0.1:0` → 实际端口 |
| 无状态端点 `/health` | ✅ **200 OK**，body `ok` |
| **有状态端点 `/metrics`** | ✅ **200 OK**，body **877 字节** |
| 依赖 | ✅ **零新增**（只用 `std`）|

## 1.2 ✅ 第三轮：**真实 WebSocket 往返跑通**

`#[cfg(test)] mod net_smoke::websocket_welcome_and_error_roundtrip`（`tokio-tungstenite` 已是依赖）：

```
DBG WS 升级：ws://127.0.0.1:57880/ws -> status 101 Switching Protocols
DBG WS Welcome：{"type":"Welcome","payload":{"connection_id":1,"protocol_version":5,"versions":{"engine_ap…
DBG WS 错误回复：{"type":"Error","payload":{"code":"BadMessage","message":"json: expected ident at line 1 column 2","room_id":null}}
```

| 判据 | 结果 |
|---|---|
| **WS 升级握手** | ✅ **101 Switching Protocols** |
| **服务端主动推送** | ✅ 首帧 `Welcome`（`protocol_version: 5`、含 `versions`）|
| **客户端→服务端→回复** | ✅ 发非法文本 ⇒ 收到 `Error{BadMessage, "json: expected ident …"}` |
| 关闭 | ✅ `ws.close(None)` 正常收尾 |

## 2 做到了什么 / 没做到什么（**如实**）

| 项 | 状态 |
|---|---|
| 路由可从测试侧构造 | ✅ **已抽取**（`build_router`，`cargo build -p game-server` 通过）|
| 真实监听 + HTTP 往返测试 | ✅ **已做**（`/health` + `/metrics`，见 §1.1）|
| WS 握手/回路测试 | ✅ **已做**（升级 101 + `Welcome` + `Error` 往返，见 §1.2）|
| TLS / 跨机 / 浏览器端 / 真实对局走 WS | ❌ **未做**（本轮 WS 只验握手与错误回路，**未**用 WS 跑整局）|

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
