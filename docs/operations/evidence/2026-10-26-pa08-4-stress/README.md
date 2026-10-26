# PA08-4 大扩展包 / 满手牌压力场景（2026-10-26）

- 日期：2026-10-26 14:00–16:10 +08:00
- 工作包：[台帐 PA08](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 扩面第四切片（大内容压力）＋ [PA10](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 增量化裁决依据
- 源码：`HEAD = 18edc033` 加未提交工作树。新增压测入口（全部 URL 门控、缺省零影响）：
  - `client.start()` 读 `packsUrl`/`hexUrl`（[autoDrive.urlParam](../../../../apps/web-client/src/engine/autoDrive.ts) 同口径）——`packsUrl` 覆盖扩展载荷；`hexUrl` 先建标准局再以存档整体替换（worker 的 restore 需既有句柄与观察者）。
  - 外链驱动 `--packs <安装载荷 JSON>`（服务在 `/packs`）与 `--hex <存档十六进制>`（服务在 `/hex`），避免 URL 长度上限。
  - 满手牌存档由原生测试 [pa08_fullhand_dump.rs](../../../../crates/engine-native/tests/pa08_fullhand_dump.rs) 产出：4 座身份局、0 号出牌阶段、手牌 30 张（基本 20＋锦囊 6＋装备 4 混排），`FULLHAND_HEX:` 输出即 [payload/fullhand.hex](payload/fullhand.hex)（3800 字符）。
- 环境：同 [PA08-3](../2026-10-26-pa08-3-browsers/README.md)；driver=auto、steps 上限 80。

**一句话结论**：**满手牌（30 张）在 Chrome 120 Hz 与 WKWebView 60 Hz 均整局正常终局 rev 144（两引擎完全一致）**；序列化与刷新全面远低预算——view_json p50 0.1 ms / p95 0.2 ms、view_parse ≤0.1 ms、适配 ≤0.7 ms、无 Long Task、掉帧率 0.035%。**PA10 的触发条件（序列化或 Vue 刷新超预算）不成立，维持现状**。

## 1 原始数据

[runs/](runs/)（RESULT 摘录见 [RESULT-lines.txt](runs/RESULT-lines.txt)）。

| run | 终局 | 帧间隔 p50 / p99 | roundtrip p50 / p95 | view_json p50 / p95 | view_parse p95 | adapt p95 | long tasks |
|---|---|---:|---:|---:|---:|---:|---:|
| chrome-bigpack（sp 整包 924K） | rev 280 | 8.30 / 9.30 ms | 2.2 / 9.5 ms | 0 / 0.1 ms | 0.1 ms | 0.2 ms | 0 |
| chrome-fullhand（手牌 30） | **rev 144** | 8.30 / 10.2 ms | 2.3 / 9.3 ms | 0.1 / 0.2 ms | 0.1 ms | 0.7 ms | 0 |
| tauri-fullhand（手牌 30） | **rev 144** | 17.0 / 27.0 ms | 3.0 / 10.0 ms | —（同口径见 summary） | — | — | 0 |

标准局对照（[PA08-3 chrome-auto](../2026-10-26-pa08-3-browsers/README.md)）：roundtrip p50 0.6 ms、view_json p50 0 ms——满手牌把 worker 段合计抬高约 0.3–0.5 ms（仍 <1 ms），回包 p95 的抬升（2.4→9.3 ms）主要来自 120 Hz 任务调度（transfer p95 8.5 ms），非序列化。

## 2 场景说明与限制

1. **满手牌**：`debug_give_hand_card` 系内核既有 debug 面（测试专用）；浏览器侧经 restore_hex 消费，属压测载荷而非产品功能。恢复态改变了发牌轨迹，故终局 rev 144 ≠ 标准局 280——同时给了跨引擎确定性的一次额外验证（Chrome 与 WKWebView 完全一致）。
2. **大扩展包**：`extensions/` 下 33 个编译包均为 **VM 模块包（manifest+modules）**，不含 cards/pile 安装载荷段——装载路径已跑通（sp.npack.json 924K：解析零 Long Task、步进指标与标准局同量级），但**不改牌堆构成、不改视图规模**，视图压力口径与标准局相同。**改牌堆的大包需 pile 安装载荷格式**（content-native-loader 的 cards/pile 桥接面，属扩展交付道范围），登记待做（见台帐）。
3. 8 座长局序列化压力已在 headless/有头 p8 整局覆盖（[基线](../2026-10-26-web-frame-latency/README.md)、[PA08-1](../2026-10-26-pa08-headed-baseline/README.md)）。

## 3 PA10 裁决依据

PA10 触发条件＝「PA08 在大包、满手牌、8 座长局显示序列化或 Vue 刷新超预算」。实测：

- 满手牌 30 张：view_json+view_parse+adapt 合计 p95 ≤ 1 ms（预算 8.3 ms/帧@120Hz）；
- p8 整局（基线表 B）：序列化 p99 ≤0.2 ms；
- 大包（VM 模块包）装载：无 Long Task、步进指标同标准局。

→ **序列化与 Vue 刷新均未超预算，PA10 维持现状（不做版本化二进制增量）**；后续随 PA08 剩余场景（移动端、长时）复测。

## 4 未覆盖

1. 改牌堆的大扩展包（pile 载荷格式，待扩展交付道提供安装载荷样例）；
2. 移动端（PA08-6）、≥30 分钟长时与内存（PA08-5）。

## 5 本批验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test pa08_fullhand_dump -- --nocapture` | 1 passed（手牌 30 断言 + 存档往返） |
| `node scripts/web-frame-external.mjs --hex …`（chrome/tauri） | 2/2 终局 rev 144 一致 |
| `node scripts/web-frame-external.mjs --packs extensions/sp/sp.npack.json`（chrome） | 终局 rev 280（装载路径跑通） |
| `pnpm --dir apps/web-client test` / `vue-tsc --noEmit` | 80 passed / 0 failed；0 错误 |
| `python3 -B scripts/check-source-lines.py` / `check_local_links.py --all` | 通过；坏链 0 |
