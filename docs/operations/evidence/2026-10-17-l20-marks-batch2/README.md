# L20 标记补充 批 2：苦肉（`kurou`）+ 主亡忠继（`_xigua_zhuwanzhongji_`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L20 标记补充**（`extension/标记补充/extension.js`）
- 本批：`kurou`（tag 533）+ `_xigua_zhuwanzhongji_`（tag 534）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 台账：L20「部分重构（批 2/N）」

## 上游语义与原生接线

| 技能 | 上游 | 原生 |
|---|---|---|
| 苦肉 `kurou`（868 B） | `enable:'phaseUse'` direct——`player.loseHp(1)` + `player.draw(2)` | `Kurou`（tag 533）：出牌阶段主动面（`activate`）→ 发 `LoseHp(1)` + `draw_cards(2)` |
| 主亡忠继 `_xigua_zhuwanzhongji_`（1.2 KB） | `trigger {global:'dieBegin'}` 锁定——死亡者 `identity=='zhu'` 且自己 `zhong`/`mingzhong` → 身份互换（自己继位主公；原主公转为忠臣；`event.trigger('zhuUpdate')`） | `ZhuWanzhongJi`（tag 534）：`Death` 触发（死亡者 `Role::Lord` 且自己 `Role::Loyalist`）→ 覆写 `GameMode::Identity.roles`（自己→Lord、原主公→Loyalist） |

> 调试面新增：`Engine::debug_set_identity_roles(Vec<Role>)`（仅测试用，覆写身份分布并校验长度=座位数）。

## 验证

```
cargo test --offline -p engine-native --test l20_kurou_batch2
cargo test --offline -p engine-native --test l20_config_mods_batch1   # 批 1 回归
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（批 2） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：苦肉——失去 1 点体力且摸 2 张；对照（未持技拒绝）；主亡忠继——5 人身份局主公阵亡后 **1 号忠臣继位（Role::Lord）**、原主公转 Loyalist、2 号反贼不变 |
| 定向（批 1 回归） | 3/3 通过（[targeted-regression.log](targeted-regression.log)） |
| engine-native 全量 | **1093 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **94/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 530（tag 533/534 追加） |

## 登记差异

- 主亡忠继的「明忠（`mingzhong`）」身份未建模（原生 `Role` 仅 Lord/Loyalist/Rebel/Traitor）→ 仅 `Loyalist` 触发。
- 上游 `zhuUpdate` 事件与 `showIdentity`（明置展示）不迁移；原生只改身份归属。
- 苦肉的 AI 估值（`result.player`）与 `basic.order` 未迁移。
- 其余 71 个技能待批次（中位 2.5 KB）。
