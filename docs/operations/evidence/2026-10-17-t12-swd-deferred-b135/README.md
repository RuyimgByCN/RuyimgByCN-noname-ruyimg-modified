# swd deferred 消解批 135：革新印（`gxianyin`）——弃一花色手牌换另一花色等量牌（v314 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **革新印**（`character/swd.js`）
- **规则/存档版本不变**（v314 / 存档 v98；无字段新增）
- 上游回读：`character/swd.js`（`gxianyin` 137 行）

## 上游语义与原生接线

| 步 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'phaseUse'`，`usable:1`，`filter`：手牌非空 | `Gxianyin`（tag 493）：出牌阶段主动面（`activate`；`host_skill_def` 每回合限一次） |
| ① 选花色 | `chooseControl`（手牌中出现过的花色；AI 按牌值偏好） | `PickTag`（手牌花色集合，升序） |
| ② 弃置 | `player.lose(hs, ui.discardPile)._triggered=null`——**不触发失去牌类事件** | 逐张 `take_hand_card` + `to_discard`（**不走** `discard_from_hand`，故不发射 `CardsLost`、不记义山） |
| ③ 选获得花色 | `chooseControl`（其余三色） | `PickTag`（其余三色） |
| ④ 获得 | `get.cardPile(该花色)` 逐张取 `hs.length` 张 → `directgain` | 从**牌堆**取该花色至多 N 张入手；不足则继续从**弃牌堆**取（登记差异：上游仅 `get.cardPile`） |

包数据：`shahai-swd-upstream-noname` 中该将的 `gxianyin` 由 `mapping:null` 改为 `mapping:"host:gxianyin"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 18 → 17**（映射 173 → 174）。
- 台账口径（登记表）：**165/184 → 166/184 映射，deferred 17 → 16**。

## 验证

```
cargo test --offline -p engine-native --test swd_gxianyin_suit_swap
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（革新印） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：弃两张黑桃（静默——义山记录为空）→ 选红桃 → 从牌堆获得 2 张红桃（等量、牌堆红桃减少）；两对照（非出牌阶段拒绝、未持技/空手拒绝） |
| engine-native 全量 | **986 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v314.log](replay-golden-v314.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 490（tag 493 追加） |

## 登记差异

- 获得来源：牌堆不足时继续从弃牌堆取（上游仅 `get.cardPile`=牌堆，可能取不满）。
- 花色选择为玩家自选（上游同）；AI 偏好未迁移（无 AI 面）。
