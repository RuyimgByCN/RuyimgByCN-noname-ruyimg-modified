# ADR 0784：桌面端原生执行引擎原型（PA09 草案）

日期：2026-10-26　|　状态：**草案（原型未落地，不构成默认架构变更）**
关联：[性能架构](../../noname-performance-architecture.md)、[PA08 证据族](../operations/evidence/)、[台帐 PA09](../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)

## 背景

台帐 PA09：Tauri 桌面当前在 WKWebView 里跑 WASM 引擎（`engine-native-wasm` 窄壳，Worker 内运行）。两个已知缺口：

1. **wasm 下 AI 时限静默失效**——`engine-native/src/ai.rs` 中 `started = None`（无墙钟语义差异的宿主探测），AI 时限与线程池收益在浏览器档从未生效。
2. **单线程 worker**——浏览器 DedicatedWorker 无线程池；AI 时限内的并行搜索（`ai_pool`/`PlanPool`）只在原生测试中运转。

PA08 扩面（2026-10-26）已把表现层实测铺满：有头 Chrome 120 Hz、Tauri WKWebView（60 Hz、真实点击 n=14）、Safari/Firefox/Chrome 三浏览器整局、大包/满手牌压测、32 分钟长时（Long Task 0、帧 p99 10.3 ms、内存终值 17 MB）。**这些数据同时是 PA09 原型的对照组基线。**

## 决策（草案）

1. **原型范围**：引擎在 Tauri Rust 进程原生运行，沿用现有 Worker 协议语义（init/start/autoStep/playCard/…/restore 的请求-信封往返），以 Tauri IPC（`invoke` 命令）替代 Worker `postMessage` 通道；web-client 增加第三种 `EngineClient` 变体（与现有 `NetworkEngineClient` 同层），消息结构不变。
2. **对照口径**：与 PA08 完全同负载（4 座身份局 seed 42、连续多局、120 Hz 外接显示）对比：帧间隔/transfer/input_to_paint（PA08 指标）＋ AI 时限生效性（`ai.rs` 原生 `Instant`）＋线程池收益（`PlanPool` 并行 vs 单线程，T13 已有基准工具）。
3. **是否切换默认架构由用户拍板**：本 ADR 只交原型 + 数据 + 建议；未拍板前默认仍是 WASM-in-WebView。

### 已知代价与风险（草案阶段评估）

- IPC 每步往返经 Tauri bridge（serde + JSON/IPC 序列化），可能比 Worker postMessage 慢——原型须实测 transfer 口径对照；
- 双实现维护面（wasm 窄壳已有 1700+ 行）；规则不迁移（内核同一 crate）；
- 扩展包数据通道（packsJson/存档 hex）需经 IPC 边界重新定义载荷上限。

## 后果（若原型数据支持切换）

- AI 时限与线程池在桌面档生效（联机服务器已原生运行同内核，桌面与服务器行为对齐）；
- WASM 包（8.4 MB）从桌面安装包移除的空间收益；
- 不支持 WASM 的旧桌面环境回退路径需保留。

## 状态

原型未开始。本草案先行记录决策框架与对照基线指针；原型落地后补实测数据并请用户裁决。
