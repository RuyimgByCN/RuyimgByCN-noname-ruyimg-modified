# swd deferred 消解批 137：脉轮（`mailun`）——阶段开始 8 选 1 效果（字段批 v316/存档 v100，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **脉轮**（`character/swd.js` 主技 + 8 条子技 `mailun31/32/41/42/5/6/7/8`）
- **规则 v315→316、存档 v99→100**（`PlayerState` 追加脉轮三字段）
- 上游回读：`character/swd.js`（`mailun` 142 行 + 8 条子技）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'phaseBegin'}`，`direct:true`（可选） | `Mailun`（tag 495）：`PhaseBegin` 触发（自己）+ 可选确认门 |
| 选择 | 8 种效果（`maxHp==6` 时不含效果 1）；`storage.mailun` 记录 | `PickTag`（1..8，`max_hp==6` 时过滤 1，可取消） |
| 旧效果清除 | step0 移除 `mailun31/32/41/42/5/6/7/8` | 选中时清零 `mailun_mode` 与两个一次性位 |
| 1 | 减少一点体力并增加一点体力上限 | `LoseHp` 帧（1 点）+ `max_hp += 1` |
| 2 | 增加一点体力并减少一点体力上限 | `recover(1)` + `lose_max_hp(1)` |
| 3 / 4 | 令即将造成和即将受到的**首次**伤害 ∓1（`mailun31/32`/`mailun41/42`，用后自移除） | 模式位 3/4 + 两一次性位 `mailun_out_used`/`mailun_in_used`；`commit_damage` 内联两臂（造成侧/受到侧各一次） |
| 5 / 6 | 少摸一张且手牌上限 +1 / 多摸一张且上限 -1 | 摸牌基数 ±1（`phaseDrawBegin trigger.num∓1`，下限 0）+ `hand_limit` ±1 |
| 7 / 8 | 进攻距离 +1 防御距离 -1 / 反向 | `computed_distance` 双端各 ∓1（`globalFrom`/`globalTo` 两端生效；下限 1） |

包数据：`shahai-swd-upstream-noname` 中该将的 `mailun` 由 `mapping:null` 改为 `mapping:"host:mailun"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 16 → 15**（映射 175 → 176）。
- 台账口径（登记表）：**167/184 → 168/184 映射，deferred 15 → 14**。

## 验证

```
cargo test --offline -p engine-native --test swd_mailun_modes
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.js
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（脉轮） | **6 passed / 0 failed**（[targeted.log](targeted.log)）：效果 5（摸 1 且上限 +1）、效果 3（首次造成伤害 -1 且一次性、第二次全额）、效果 7（双向距离 -1）、效果 1（失 1 体力上限 +1；上限 6 时候选 7 项不含效果 1）、对照（未持技不开窗、放弃不改状态）、三字段存档往返 |
| engine-native 全量 | **996 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（316→315 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v316.log](replay-golden-v316.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 492（tag 495 追加） |

## 登记差异

- 效果 1 的「体力上限 +1」直接改 `max_hp` 并沿用 `LoseHp` 帧（原生无 MaxHpChanged 事件种；上游亦无独立事件）。
- 效果 3/4 的「首次伤害」按**模式位 + 一次性位**建模（上游为临时子技用后移除）；一次性位在重选效果时清零（上游同步移除子技）。
- AI 偏好（上游 `mailun` 的 `else` 分支按体力自动选择）未迁移（无 AI 面）。
