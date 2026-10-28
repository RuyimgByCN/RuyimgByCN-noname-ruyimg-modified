# NC03／NC09：本地 view_json 身份与可见牌面投影（裁决⑥⑦）

- 实际执行：2026-10-28（[开始](start-time.txt)）；源码基准 [`ec672a1d3c413d39c9972ee02118ae92a21223ee`](source-head.txt) + 本会话 B7 批工作树改动（规则 v324）。**规则版本、存档 v105、全局原生 ABI 753、Worker／WS 消息结构均不变**（裁决⑥⑦明确边界）。
- 授权：[用户裁决登记](../2026-10-28-user-decisions/README.md) 第 6、7 项——仅扩展本地 WASM `view_json` 内容与 Worker 对已有字段的复用。

## 修复的反例（占位违规，GT-013）

1. **identity 恒 null 占位**：内核 `PlayerCellView.role` 已按「本人/主公/已死亡可见」过滤（与服务端 `identity_tag_for` 同口径），但 WASM `view_to_json` 不序列化该字段，Worker 适配层 `nativeObserverToSnapshot` 把每个座位 `identity` 硬编码为 `null`——已授权的身份信息被占位值抹掉。
2. **牌面全占位**：`view_json` 只发裸 CardId；Worker 的 `known_cards` 仅含本人手牌+本人装备且 `def/suit/rank` 全为 `null`/`"Other"`——产品 UI（`BattleBoard.vue`）以 `def != null` 为渲染门，本人手牌、全场装备/判定、弃牌堆因此全部不渲染。

## 修复设计

| 层 | 改动 |
|---|---|
| WASM `view_to_json`（[lib.rs](../../../../crates/engine-native-wasm/src/lib.rs)） | 座位对象新增 `identity`（`Lord/Loyalist/Rebel/Renegade/null`，与协议 `IdentityTag` 及 TS `SeatView.identity` 命名一致）；顶层新增 `cards`：对 `visible_card_ids()`（本人手牌+全场装备/判定+弃牌/处理区）附真实牌面 `{def, suit, rank, kind}`。`def` = 牌种目录序号（与服务端 `kind_code_u32` 同式，杀=0 是合法值）；`kind` 与服务端 `kind_tag` 同映射（六类直映，其余 `Other` 不猜测）——跨入口同契约 |
| Worker `nativeObserverToSnapshot`（[nativeBackend.ts](../../../../apps/web-client/src/worker/nativeBackend.ts)） | `identity` 透传（复用 `SeatView.identity`）；`known_cards` 由 `cards` 牌面填充：本人手牌（Hand）、全场装备/判定（Equip/Judge，带 owner）、弃牌（DiscardPile）、处理区（Processing）——复用已有 `CardView` 字段，消息结构不变 |

替代方案考虑：①Worker 侧自建 id→牌面目录——违反「单一可见性契约」（牌面权威在内核目录）且需跨端同步，弃；②`kind` 用更细的 TS 联合命名（如 TrickNanman）——服务端 `kind_tag` 现映射只有六类+Other，本地更细会造成同局面跨入口 `kind` 字段不一致，按「不猜测、同契约」弃，粗粒度收敛列为待办。

## 验收（GT-013：不泄露 + 不占位）

| 入口 | 断言 | 结果 |
|---|---|---|
| WASM 新测试 [nc03_view_projection.rs](../../../../crates/engine-native-wasm/tests/nc03_view_projection.rs) | ①5 座身份局全 viewer 矩阵：可见集={主公}∪{本人}，非主公他人 null、主公对所有 viewer 同座位；`cards` 键集==可见牌集、每张 def/suit/rank 非占位（suit∈四花色、rank∈1..13）；他人手牌无 own 列表、牌堆仅数量 ②save→restore 后 identity 矩阵与牌面逐字一致 | 2/0（[wasm-suite.log](wasm-suite.log)：全 crate **68/0/0**，17 二进制） |
| Worker 纯函数（[native-adapter.test.mjs](../../../../apps/web-client/scripts/native-adapter.test.mjs) +2 测） | identity 透传非恒 null；known_cards 分区/owner/face 映射（Hand/Equip/Judge/DiscardPile/Processing），他人手牌不进入 | 前端 **116/0**（[frontend-test.log](frontend-test.log)，typecheck 0 错） |
| 真实 Worker 链路（[native-worker-smoke.mjs](../../../../apps/web-client/scripts/native-worker-smoke.mjs) +2 检查） | 身份局 start 回包 `seats.identity=["Lord",null,null,null]`；vm-only 建局后本人手牌带真实 face（如 `{def:7,suit:"Spade",rank:1,kind:"ShaFire"}`）且无他人 Hand 条目 | **64/0**（[smoke-native.log](smoke-native.log)，先从本批源码 [重建](wasm-rebuild.log) exit 0） |
| 跨入口 def 口径（game-server [nc09_native_view_bridge.rs](../../../../apps/game-server/tests/nc09_native_view_bridge.rs) +断言） | 服务端 `CardView.def` == 目录牌种序号 == 本地 wasm `cards.def` 同式（同局面两入口对同一张牌同一 def） | game-server 全量 **360/0/0**（[game-server.log](game-server.log)） |

死亡公开（遗言）语义由内核 `t13_info_isolation` 覆盖，本批 JSON 层为 `role` 的 1:1 序列化。

## 验证汇总

- `cargo test --offline -p engine-native-wasm`：**68/0/0**（基线 66，+2 新测试）
- `cargo test --offline -p game-server`：**360/0/0**（含新增 def 跨入口断言）
- `pnpm --dir apps/web-client build:native`（本批源码重建）exit 0；`typecheck` 0 错；`test` **116/0**（基线 114，+2）；`smoke:native` **64/0**（基线 62，+2）
- 行数门禁：[gates.log](gates.log) 通过（8652 文件 ≤2000；`engine-native-wasm/src/lib.rs` 1893 行）
- engine-native / 规则版本 / golden：本批未触及（B7 批已全量 1617/0/73）

## 兼容性

- 规则 v324（B7 批）、存档 v105、ABI 753 不变；Worker→主线程与 WS 消息**结构**不变（`ObserverSnapshot`/`SeatView`/`CardView` 均为既有类型，仅内容从占位变真实）。
- 隐私面：`cards` 键集被断言恒等于内核 `visible_card_ids()`——他人手牌与牌堆 id 不下发（WASM 侧本就经 `HandView::Hidden` 隔离）。

## 未覆盖项（登记，不占用本批授权）

- 服务端 `known_cards_for` 的 `suit/rank` 仍为 `None`、公开区（装备/判定/弃牌/处理区）不进服务端 known_cards——跨入口**字段丰富度**仍有差距（本批裁决明确只授权本地；服务端补齐涉及 WS 载荷内容变化，需单独请示）。
- `kind` 粒度收敛（服务端 `kind_tag` 仅六类直映，Weapon/Armor/马匹等 → Other）同上，跨入口统一待授权。
- 产品 UI 对新牌面的完整渲染验收（本批验证到 `def != null` 渲染门输入面；整局视觉验收属 NC09 产品全流程，未做）。
