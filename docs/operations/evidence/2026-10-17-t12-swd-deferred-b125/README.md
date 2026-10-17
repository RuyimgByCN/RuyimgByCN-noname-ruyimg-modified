# swd deferred 消解批 125：魅惑（`meihuo`）+「失去牌」事件原语（v312，2026-10-17）

- 日期：2026-10-17
- 范围：新引擎原语 **`EventKind::CardsLost`（失去牌事件）** + swd 来源 deferred 技能 **魅惑**（`meihuo`）
- **规则 v311→312**（新增事件种类与发射点；**无 `PlayerState` 字段变化**，存档仍 v96——事件帧编码新增判别 18，故仍属协议批）
- 上游回读：`character/swd.js`（`meihuo` 45 行）、`game/game.js`（`loseEnd` 语义）

## 「失去牌」事件原语

| 面 | 内容 |
|---|---|
| 事件种类 | `EventKind::CardsLost`（表尾追加，判别 20） |
| 载荷 | `EventData::CardsLost { player, cards, origins }`——`origins: Vec<LostOrigin>`（`Hand`/`Equip`/`Judge`）与 `cards` 等长，对应上游 `card.original`（'h'/'e'/'j'） |
| 发射点 | `discard_from_hand`（弃置手牌）、`give_hand_card`（交出手牌）、`take_owned_card`（手牌/装备被取走，装备来源标 `Equip`）；**装备替换不发**（上游 `equiping` 期过滤同口径，登记差异） |
| 编码 | 存档事件载荷判别 18（origins 逐字节 0/1/2）+ 快照校验 `check_event_data` 等长检查 + 规范哈希判别 15 |
| 帧语义 | 子帧在发射步内成为栈顶：`Ctx::current_frame_id()` **跳过 `CardsLost` 帧**（新增 `top_frame_id()` 取裸栈顶），使「我正处理的那个事件」语义对既有 20+ 处 `current_frame_id()` 调用点保持稳定 |

## 魅惑（`meihuo`，tag 480）

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `loseEnd`（自己失去牌） | `CardsLost` 触发（可选确认门） |
| 条件 | `!player.equiping && countCards('e')==0` 且失去的牌含 `original=='e'` | 失去牌来源区含 `Equip` 且自己装备区已空（装备替换不发事件，同上游 `equiping` 过滤） |
| ① 选目标 | `chooseTarget([1,1])`：其他有 `he` 牌者（可取消） | `SelectPlayers`（其他存活且手/装备非空，min1/max1，`allow_cancel`） |
| ② 取牌 | `choosePlayerCard(target,'he',true)`（强制） | `SelectCards`（目标手牌+装备，min1/max1，不可取消） |
| ③ 结算 | `get.position(card)=='e'` → `player.equip(card)`；否则 `player.gain(card)` | `take_owned_card`（撤销装备技）后：装备牌入手 → `Equipped` 帧（`commit_equip` 全托管）；非装备入手 |

包数据：`shahai-swd-upstream-noname` 中该将的 `meihuo` 由 `mapping:null` 改为 `mapping:"host:meihuo"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 28 → 27**（映射 163 → 164）。
- 台账口径（登记表）：**155/184 → 156/184 映射，deferred 27 → 26**。

## 验证

```
cargo test --offline -p engine-native --test swd_meihuo_cards_lost
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（魅惑 + 事件） | **5 passed / 0 failed**（[targeted.log](targeted.log)）：装备来源触发 → 取目标装备并直接装备（不入自己手牌）；取目标手牌入手；两对照（仍有装备不开窗、未持技/手牌来源不开窗）；失去牌事件挂起中存档往返后仍可继续 |
| engine-native 全量 | **948 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（312→311 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v312.log](replay-golden-v312.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 477（tag 480 追加） |

## 过程勘误（回归抓漏）

子帧插入一度破坏两处既有语义（b22 玉鳞/朱羽、b77 横刀）：
1. 机器在一步内发射子帧后，`ctx.res.frame_mut(ctx.current_frame_id())` 读到子帧 → 伤害直写落空；
2. 若把 `current_damage`/`current_card_use` 改为「就近匹配」，则 `CardResponded` 触发机会误读外层 `CardUsed`（横刀花色锚错）。

**定案**：只让 `current_frame_id()` 跳过 `CardsLost` 子帧（其余访问器语义不变）——两处回归均恢复，全量 948/0/1。
