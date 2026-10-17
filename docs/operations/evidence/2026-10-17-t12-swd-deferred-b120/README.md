# swd deferred 消解批 120：灵武（`lingwu`）+ `insertPhase` 额外阶段原语（字段批 v307/存档 v92，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **灵武**（妮可，`character/swd.js:6325`）+ 新引擎原语 **额外阶段**（`insertPhase`）
- **规则 v306→307、存档 v91→92**（`TurnState` 追加 `cards_used_turn`/`extra_phases`/`extra_phase_lingwu`）
- 上游回读：`character/swd.js:6325`（`lingwu`）、`game/game.js:23955`（`countUsed`）、`game/game.js:24508`（`insertPhase`）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'phaseAfter'}`，`frequent:true` | 阶段收束（`commit_phase` 尾段）内联臂（原生无 PhaseEnd 帧，同水没墓穴/光塔口径） |
| 条件 | `player.countUsed() >= player.hp && event.skill != 'lingwu'` | `turn.cards_used_turn >= hp`（新增回合级已用牌数：CardUsed 提交处 +1、回合开始清零；上游 `countUsed()` 无参口径=本回合全部使用，含出牌阶段外）+ 防递归位 `extra_phase_lingwu` |
| 效果 | `player.insertPhase()` → 在当前阶段之后插入一个**出牌阶段** | `turn.extra_phases.push(2)`（码同 `Phase`）；阶段收束时队列非空则弹出为下一阶段（原语通用：任意阶段码可入队） |
| 防递归 | 插入阶段的 `next.skill = 'lingwu'` → `event.skill != 'lingwu'` 过滤 | 弹出时置 `extra_phase_lingwu = true`；该阶段收束时先取该位并清零，位为真则不再插入 |
| 频繁技 | `frequent:true`（不询问） | 恒发动（登记差异同既有频繁技口径） |

包数据：`packages/extensions/shahai-swd-upstream-noname` 中 `swd_nicole` 的 `lingwu` 由 `mapping:null`（defer：insertPhase 引擎缺口）改为 `mapping:"host:lingwu"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 33 → 32**（映射 158 → 159）。
- 台账口径（登记表）：**150/184 → 151/184 映射，deferred 32 → 31**（登记表以核验脚本口径为准）。

## 验证

```
cargo test --offline -p engine-native --test swd_lingwu_extra_phase
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（灵武） | **5 passed / 0 failed**（[targeted.log](targeted.log)）：已用 1 张且体力 1 → 阶段收束插入额外出牌阶段、插入阶段收束不再插入；对照（已用 < 体力不插入；无灵武不插入）；存档往返（队列 `[2,3]` + 插入位 + 回合计数保持，恢复后依序弹出出牌/弃牌阶段）；换回合计数清零 |
| engine-native 全量 | **926 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（307→306 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v307.log](replay-golden-v307.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 472（tag 475 追加） |

## 影响

- 台账 swd 行与 `T12-SWD-DEFERRED-SKILLS` 计数更新（deferred 32 → 31，映射 151/184）。
- 额外阶段原语可承载后续同类技能（`insertPhase` 家族）。
