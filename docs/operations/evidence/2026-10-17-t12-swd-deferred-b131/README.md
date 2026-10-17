# swd deferred 消解批 131：护天（`hutian`）——阶段结束寄存牌作护体（字段批 v314/存档 v98，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **护天**（`character/swd.js`，主技 + 组技 `hutian3` + 子技 `hutian2`）
- **规则 v313→314、存档 v97→98**（`PlayerState` 追加护天四字段）
- 上游回读：`character/swd.js`（`hutian` 60 行 + `hutian2` 24 行 + `hutian3` 16 行 + `hutian4` 空壳）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'phaseEnd'}`，`direct:true`，`filter`：手牌非空且无寄存（`!storage.hutian`） | `Hutian`（tag 487）：`PhaseEnd` 触发 + 可选确认门；过滤：手牌非空且 `hutian_deposit.is_none()` |
| ① 选牌 | `chooseCardTarget`：`selectCard [1, countCards('he')]`（手/装备） | `SelectCards`（手牌+装备，min1/max=可用数，可取消） |
| ② 选目标 | `filterTarget`：`target.maxHp >= ui.selected.cards.length` | `SelectPlayers`（其他存活且 `max_hp >= 张数`，min1/max1，可取消） |
| ③ 寄存 | `player.lose(cards, ui.special)`；`storage.hutian = target`；`target.storage.hutian2 = cards`；`addTempSkill('hutian4')`（默认 expire=phaseAfter/phaseBefore） | 牌离场入寄存区（手牌 `take_hand_card` / 装备撤技后移除；**不入弃堆**）；`hutian_deposit`/`hutian_just_given`（寄存方）+ `hutian_hold`/`hutian_owner`（被寄存方） |
| ④ 补体 | `if(target.hp < cards.length) target.recover(cards.length - hp)` | `recover(target, 张数 - hp)` |
| 护体 | `hutian2`：`damageBegin`/`loseHpBegin` priority -55 forced——`hp - num < 寄存数` → `num = hp - 寄存数` | 内联两臂：`commit_damage` 与 `commit_lose_hp` 中 `amount = min(amount, hp - 寄存数)`（下限 0） |
| 归还 | `hutian3`：`phaseEnd`/`dieBegin` forced——`!hasSkill('hutian4') && storage.hutian` → `target.gain(寄存牌)` + 清理 | 阶段**收束**内联臂（原生阶段收束 commit 先于该阶段触发窗 → 寄存当次收束看不到寄存，**下一次阶段收束**归还 ✓ 与上游「下一阶段边界移除 hutian4 → 下一次 phaseAfter 归还」同口径）；死亡前置内联臂（`DeathMachine` 首步）归还 |

包数据：`shahai-swd-upstream-noname` 中该将的 `hutian` 由 `mapping:null` 改为 `mapping:"host:hutian"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 22 → 21**（映射 169 → 170）。
- 台账口径（登记表）：**161/184 → 162/184 映射，deferred 21 → 20**。

## 验证

```
cargo test --offline -p engine-native --test swd_hutian_deposit
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（护天） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：寄存 2 张（离手、入寄存区、目标体力 1→2 回复至张数）、护体上限（再受 1 伤体力不降至寄存数以下）、下一阶段收束归还（寄存区清、牌入目标手牌）；对照（目标上限过滤 + 放弃目标窗不寄存）；寄存方死亡归还；四字段存档往返 |
| engine-native 全量 | **972 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（314→313 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v314.log](replay-golden-v314.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 484（tag 487 追加） |

## 登记差异

- 寄存牌的存放以专用状态字段承载（上游 `ui.special` 特殊区；原生无通用特殊区，牌不入弃堆、随字段守恒）。
- 寄存时长按上游 `hutian4` 默认 `expire=['phaseAfter','phaseBefore']` 建模为「下一次阶段收束归还」；原生阶段收束 commit 早于触发窗，故寄存当次的收束不归还（与上游时序等价）。
