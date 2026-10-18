# swd deferred 消解批 155：攻心（`gongxin`）——看手牌选红桃：弃置或置牌堆顶（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **攻心**（`character/extra.js:11396`——外部包技能但本仓有定义）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 上游回读：`character/extra.js`（`gongxin` 全量）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'phaseUse'`，`usable:1`；`filterTarget`：其他且有手牌 | `Gongxin`（tag 514）：出牌阶段主动面（`activate`；存在有手牌的其他角色；`host_skill_def` 每回合一次） |
| 选目标 | `chooseButton` 前的目标选择 | `SelectPlayers`（其他存活且有手牌，min1/max1，可取消） |
| 看手牌 | 对话窗列出目标**全部**手牌；`filterButton`：两张按钮须异类（一为字符串选项、一为牌），且牌须**红桃** | `SelectCards` 展示目标**全部手牌**；应答非红桃时**拒绝并重开窗**（同口径：展示全手、仅红桃可落地） |
| 选择 | 按钮字符串：`'弃置此牌'` / `'置于牌堆顶'` | `PickTag`（0=弃置、1=置顶，不可取消） |
| 效果 | 弃置：`target.discard(card)`；置顶：`target.lose(card, ui.cardPile, 'visible', 'insert')` | 弃置：`discard_from_hand(target, card)`；置顶：`take_hand_card` + `deck.push`（牌堆顶=末尾，摸牌自末尾取） |

包数据：`shahai-swd-upstream-noname` 中该将的 `gongxin` 由 `mapping:null` 改为 `mapping:"host:gongxin"`，npack 已 rehash。

## 计数

- 武将技能条目：**deferred 2 → 1**（映射 **180/184 → 181/184**）。

## 验证

```
cargo test --offline -p engine-native --test swd_gongxin_hand_peek
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（攻心） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：目标窗 → 手牌窗（**含非红桃展示**）→ 选非红桃被拒并重开窗（牌未动）→ 选红桃 → 选择窗 → **弃置**（入弃牌堆）；置顶分支（`deck.last()==该牌`）；对照（未持技拒绝） |
| engine-native 全量 | **1053 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 510（tag 514 追加） |

## 登记差异

- 上游为**单窗双按钮**（字符串选项 + 红桃牌）；原生拆为「手牌窗 → 选择窗」两窗，非红桃应答以**重开窗**承载（等价于 `filterButton` 的拒绝）。
- 置顶日志以 `CardsGiven(from=目标, to=自己)` 记录（原生无「置入牌堆」专用日志；仅日志面差异）。
- 上游 `ai`/`threaten`/`result`（AI 估值）未迁移。
