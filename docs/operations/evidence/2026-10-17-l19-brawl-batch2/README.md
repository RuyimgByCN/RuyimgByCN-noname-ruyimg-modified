# L19 末日乱斗 批 2：灾变（`zaibian`）+ 装载器（身份链 / 无尽模式配置）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L19 末日乱斗**（`extension/末日乱斗/extension.js`）
- 本批：技能 `zaibian`（tag 525）+ 装载器 `content-native-loader::doomsday`
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 台账：L19「部分重构（批 2/N）」

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 灾变 | `trigger {player:'phaseUseBegin'}` 锁定——`忠+2-反-内 > 0` 时摸该数 | `Zaibian`（tag 525）：`PhaseBegin(Play)` → 按 `identity_roles()` 统计（Lord/Loyalist=忠、Rebel=反、Traitor=内）→ 摸 `max(0, 忠+2-反-内)` |
| 身份链 | `chanceIdentity` 链：依次给「下一个玩家」分配 `er/san/si/wu1/liu/qi/ba/jiu1/shi/shiyi/…`，每步以 `players.length > N` 续链（`chanceIdentityN` 门 = N+1） | `IDENTITY_NAMES`（**15 项链** er..shiliu）+ `IDENTITY_GATES`（首项无门）+ `identity_chain(seats)`（链式截断，与上游门逐项对齐） |
| 无尽模式 | `_wujinmoshiDraw/Damage/DamageI/Recover/MaxHandCard/DXB`（`player == game.me` 限定人类位） | `WujinMoshiConfig`（`parse_wujinmoshi` 键值解析 + `applies_to/draw_bonus/damage_bonus/recover_bonus/hand_card_bonus/non_human_damage_delta`；人类位以 `human_seats` 参数注入=登记差异） |

> 计数修正记录：身份链首版误写 16 项链并误设门序 → 依上游 `chanceIdentityN` 门（N+1）校正为 **15 项**（er..shiliu）并修正测试期望（4 人 → `er/san`；5 人 → `er/san/si`；16 人 → 14 项；17 人 → 15 项）。

## 验证

```
cargo test --offline -p engine-native --test l19_zaibian_batch2
cargo test --offline -p content-native-loader --test doomsday_loader
cargo test --offline -p engine-native --test l19_brawl_batch1      # 批 1 回归
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（灾变） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：出牌阶段开始摸 2 张（Prototype 无身份 → 0+2-0-0）；对照（未持技不摸） |
| 定向（装载器） | **3 passed / 0 failed**（[loader-tests.log](loader-tests.log)）：身份链五档门（2/3/4/5/6 人）+ 16/17 人链长；灾变摸牌四例（含下限 0）；无尽模式加成仅人类位 + `floor(DXB/15)-I` 正负例 |
| 定向（批 1 回归） | 3/3 通过（[targeted-regression.log](targeted-regression.log)） |
| engine-native 全量 | **1083 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **94/0/0**（[deps-tests.log](deps-tests.log)；较上批 +3） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 521（tag 525 追加） |

## 登记差异

- 身份链在原生为**策略函数**（返回分配序列），模式注册与身份牌展示不迁移；`chanceIdentity0`（`yi`）上游被注释且非链成员，未纳入。
- 无尽模式的「人类位」以 `human_seats` 参数注入（上游 `player == game.me`）；加成尚未接入内核摸牌/伤害/回复/手牌上限收束（仅装载面 + 查询）。
- 乱斗模式框架（模式注册、24 处武将引用、动画）未迁移；其余 63 技待批次。
