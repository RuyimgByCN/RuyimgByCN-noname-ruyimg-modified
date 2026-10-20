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

## 1.3 ✅ 第四轮：**WS 全链路（建局 / 加入 / 命令 / 个人化结果）**

`net_smoke::websocket_creates_joins_and_submits_over_wire`：**两条真实 WS 连接**走完整协议。

```
DBG WS 建局：room=1 player=1
DBG WS 加入：room=1 player=2
DBG WS 命令结果：result=Accepted { new_revision: 2, state_hash: [216, 37, …] } revision=2
```

| 判据 | 结果 |
|---|---|
| A 连接 → `CreateRoom` → **`JoinOk{room=1, player=1}`** | ✅ |
| B 连接 → `JoinRoom{room=1, seat=1}` → **`JoinOk{player=2}`** | ✅ |
| A 发 **`SubmitCommand{EndPhase}`** → **`CommandOutcome{Accepted{new_revision:2}}`** | ✅（含 `state_hash`）|
| 编解码 | ✅ 全部走 `engine-protocol::{encode_c2s, decode_s2c}`（**真实线上格式**）|

⇒ **WS 上的「建局→加入→提交命令→拿回个人化结果」全链路已验证**。

## 1.4 ✅ 第五轮：**WS 上的整局对局**

`net_smoke::websocket_plays_match_to_completion`：两条真实 WS 连接驱动到终局，
`legal_actions` 为空时用 **`RequestSnapshot`** 刷新（快照带 `choice`）。

```
DBG WS 整局：room=1 A=1 B=2
DBG WS 整局结果：outcome=LoyalistWin steps=52 snapshots=52 rev=53
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.22s
```

| 判据 | 结果 |
|---|---|
| **WS 上跑完整对局** | ✅ **`outcome=LoyalistWin`**，**52 步 / 52 次快照 / rev=53**，0.22 s |
| 断言 | ✅ `steps>0 && snapshots>0`、`outcome != InProgress` |
| 协议 | ✅ 全部经 `encode_c2s`/`decode_s2c`（**真实线上格式**）|
| 机器人选步 | 同 `long_run_bot.rs`（`ResolveChoice` 原样 → 出牌 → `EndPhase`），**非强 AI** |

⇒ **「真实网络（loopback WS）上的完整对局」已达成本轮之前未做的最后一块**。

## 1.5 ✅ 第六轮：**断线重连（`reconnect_token`）**

`net_smoke::websocket_reconnect_restores_seat_and_revision`：

```
DBG 重连：建局 room=1 player=1 token_len=36
DBG 重连：B 加入 player=2
DBG 重连：命令后 rev=2 legal=7
DBG 重连：重连后 player=1 rev=2
```

| 判据 | 结果 |
|---|---|
| 建局拿到 `reconnect_token` | ✅ 长度 **36** |
| 走一步后 `rev=2`、`legal=7` | ✅ |
| **断开后用 token 重连** | ✅ 恢复 **同一 `player_id`（1）** |
| **修订号不倒退** | ✅ 重连后 `rev=2`（= 断开前）|

### 1.5.1 ⚠ **前一轮结论更正 + 一处真实改进**

**更正**：上一轮我写「座位未满时**没有**显式错误回推」——**错误** ❌。
真相：服务端**一直**会回 `Error` 帧，只是**我的测试循环只等 `CommandOutcome`、把 `Error` 忽略了** ❌。

**实测（本轮新增用例 `websocket_submit_before_match_start_returns_error`）**：

```
DBG 未开局提交：code=Conflict message=match not started
```

⇒ 行为本身**正确** ✅（有明确错误、有可读信息）。

**真实的小问题（已修）**：该错误原先映射为 **`ErrorCode::Internal`**（`map_room_err` 的兜底分支），
语义是「**服务端内部故障**」❌，而这里是**客户端时序冲突** ✅。已在 `lobby.rs:map_room_err` 增加：

```rust
"match not started" => HandlerError::new(ErrorCode::Conflict, "match not started"),
```

并用断言的 `assert_eq!(code, Conflict)` **锁住**该映射。

## 1.6 ✅ 第七轮：**WS 多房间隔离**

`net_smoke::websocket_multi_room_isolation`：**3 个房间各自两条 WS 连接**，交替推进 3 轮。

```
DBG 多房间：room#0 id=1 A=1
DBG 多房间：room#1 id=2 A=1
DBG 多房间：room#2 id=3 A=1
DBG 多房间结果：revs=[4, 4, 4]
```

| 判据 | 结果 |
|---|---|
| 三个 `room_id` 互不相同 | ✅ **1 / 2 / 3** |
| 每房间 `EndPhase` 均 `Accepted` | ✅ |
| **各房间修订号独立** | ✅ 每房间各自 **rev=4** |
| **不同 seed ⇒ `state_hash` 不同** | ✅ 断言通过（**房间不共享引擎**）|

## 2 做到了什么 / 没做到什么（**如实**）

| 项 | 状态 |
|---|---|
| 路由可从测试侧构造 | ✅ **已抽取**（`build_router`，`cargo build -p game-server` 通过）|
| 真实监听 + HTTP 往返测试 | ✅ **已做**（`/health` + `/metrics`，见 §1.1）|
| WS 握手/回路测试 | ✅ **已做**（升级 101 + `Welcome` + `Error` 往返，见 §1.2）|
| **WS 上的建局/加入/命令/结果** | ✅ **已做**（见 §1.3）|
| **用 WS 跑整局** | ✅ **已做**（52 步跑到 `LoyalistWin`，见 §1.4）|
| TLS / 跨机 / 浏览器端 | ❌ **未做**（**仅 loopback 明文**）|
| **断线重连**（`reconnect_token`）| ✅ **已做**（同一席位 + 修订号保持，见 §1.5）|
| **WS 下的多房间并发** | ✅ **已做**（3 房间隔离，见 §1.6）|
| **座位未满时提交命令的错误回推** | ✅ **有**（`Conflict` / `match not started`）——上一轮「静默等待」的判断已**更正** |
| 错误码语义 | ✅ 由 `Internal` 修正为 **`Conflict`**（`map_room_err`）|

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
