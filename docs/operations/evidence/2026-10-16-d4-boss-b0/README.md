# 2026-10-16 D4 Boss 关卡流程子系统 · 批次 0（骨架）

**核验日期**：2026-10-16　**待做 ID**：`T11-BOSS-STAGE-SUBSYSTEM`　**ADR**：[0783](../../../adr/0783-boss-stage-flow-subsystem.md)　**开工清单**：[batch-0.md](../2026-10-16-d4-boss-adr/batch-0.md)

## 结论

B0 六项机制已落地：默认空态 `BossStageState`（`kind=None`）挂 `GameMode::Boss`；存档 **78→79**（旧档版本门禁拒绝、无迁移）；规则 **290→291** 且摘要哈希纳入 stage 字段；内部原语 `change_boss` / `reinit_boss_seat` / `add_boss_fellow`；禁将与孟婆乱入旗只读查询空实现。本批**不挂** HostBehavior（最高 tag 仍为 **410** `SwdMinjing`；内容技自 **411** 起留给 B1+）。无隐藏链/还人/翼/bianshen 内容技；排除 D5/`ylyg*`、映姬、地藏大愿、庞统外部技。

## 六项完成状态

| ID | 状态 | 落地 |
|---|---|---|
| `B0-STAGE-STATE` | 完成 | `BossStageState` / `BossStageKind` / `BossStagePending`；`GameMode::Boss.stage`；默认 `default_none()` |
| `B0-ARCHIVE-ROUNDTRIP` | 完成 | `ARCHIVE_FORMAT_VERSION` 78→**79**；enc/dec `BossStageState`；v78 头拒收单测 |
| `B0-HASH-PROBE` | 完成 | `canonical_hash` Boss 分支纳入 stage；规则 290→**291**；回退探针全绿后 14 golden 重录 |
| `B0-CHANGE-BOSS-API` | 完成 | `Engine::change_boss` / `reinit_boss_seat`（装配重挂；座位环不破坏） |
| `B0-ADD-FELLOW-API` | 完成 | `Engine::add_boss_fellow`（队尾+`seat_ring` 同步）；`debug_add_fellow` 转发 |
| `B0-BAN-QUERY-STUB` | 完成 | `is_boss_general_banned` / `boss_mengpo_spawned` / `boss_mengpo_died`（默认空表/false） |

## 版本与 golden

| 项 | 值 |
|---|---|
| `STANDARD_RULESET_VERSION` | 290→**291** |
| `ARCHIVE_FORMAT_VERSION` | 78→**79** |
| HostBehavior 最高 tag | **410**（未占用 411） |
| 版本回退探针 | 新代码 + 规则回退 291→290 + **旧 golden 表** → 14 场全绿（[probe-290-newcode.log](probe-290-newcode.log)）——位移唯一来源=规则版本字段（标准场景不进 Boss 哈希分支） |
| 14 golden | 程序化行级重录（[golden-v291-actuals.txt](golden-v291-actuals.txt)）；`replay_scenarios` 2/2（[replay-golden-v291.log](replay-golden-v291.log)） |

## 测试

| 口径 | 结果 |
|---|---|
| 定向 `d4_boss_b0_*` | **6/6** |
| 全量 | **822/0/1**——[engine-native-tests.log](engine-native-tests.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=.scratch/target-d4-b0-gate1b cargo check -p engine-native --all-targets` **exit 0**——[gate1b-all-targets.log](gate1b-all-targets.log) |

独立 `CARGO_TARGET_DIR=.scratch/target-d4-b0`（全量）/ `.scratch/target-d4-b0-gate1b`（Gate）；未 `cargo clean` 共享 target。

## B1 建议

按 ADR §6：先落 `boss_qingmu2`/`boss_qingmu2x`（最短链对照），再 `boss_qingmu3`/`boss_qingmu3x`（三人 hide+侧座），其余六技参数化同构调用本批 `change_boss` / `add_boss_fellow` / 座位环访问器；HostBehavior 自 **tag 411** 顺序编号。
