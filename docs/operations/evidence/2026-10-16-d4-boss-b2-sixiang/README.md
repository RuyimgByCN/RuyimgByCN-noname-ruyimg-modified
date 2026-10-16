# 2026-10-16 D4 Boss 关卡流程 · 批次 B2（白芒/赤炎/玄麟六技）

**核验日期**：2026-10-16　**待做 ID**：`T11-BOSS-HIDECHAIN-8`（本批白芒/赤炎/玄麟 2/2x）　**ADR**：[0783](../../../adr/0783-boss-stage-flow-subsystem.md)　**前置**：[B1](../2026-10-16-d4-boss-b1-qingmu/README.md)（`853b2bb9`）

## 结论

续用上一任中断半成品（`host.rs` 已挂 enum/tag/filter/TriggerMeta），补齐执行臂、目录装配、规则版本与定向。落包 `boss_baimang2/2x`、`boss_chiyan2/2x`、`boss_xuanlin2/2x`（HostBehavior tag **415–420**），复用 B1 `SixiangChainParams`。规则 **292→293**；存档不变 **v79**。本批不做 3/3x、翼编排、还人、bianshen、D5。

## 技能与 tag

| 技能名 | HostBehavior | tag | 上游 | 原生承载 |
|---|---|---|---|---|
| 白芒 | `BossBaimang2` | **415** | dieBegin hide | Death BeforeTriggers `Timing::First` |
| 白芒·转阶 | `BossBaimang2x` | **416** | →蓐收+两明星+全场回满+挑战者重整 | Normal priority -10；`death_cancelled` + 链 |
| 赤炎 | `BossChiyan2` | **417** | dieBegin hide | First |
| 赤炎·转阶 | `BossChiyan2x` | **418** | →祝融+两焰灵+全场回满+挑战者重整 | Normal -10 |
| 玄麟 | `BossXuanlin2` | **419** | dieBegin hide | First |
| 玄麟·转阶 | `BossXuanlin2x` | **420** | →共工+一玄冥+全场回满+挑战者重整 | Normal -10 |

条目：白虎挂 `[神裔,狂笑,白芒,白芒·转阶]`；朱雀挂 `[神裔,焚天,赤炎,赤炎·转阶]`；玄武挂 `[神裔,灵渠,玄麟,玄麟·转阶]`。

## 登记差异

与 B1 同族：无 dieAfter 窗、无八座 changeSeat、changeBoss 座位保留清区摸 4、hide 为日志/stage 近似。玄麟定向测以 1 体+1 伤绕开灵渠「amount>1 归零」。

## 版本与 golden

| 项 | 值 |
|---|---|
| `STANDARD_RULESET_VERSION` | 292→**293** |
| `ARCHIVE_FORMAT_VERSION` | **79**（不变） |
| HostBehavior 最高 tag | **420** |
| 版本回退探针 | 新代码 + 规则回退 293→292 + 旧 golden → 14 场全绿（[probe-292-newcode.log](probe-292-newcode.log)）——位移唯一来源=规则版本字段 |
| 14 golden | 程序化重录（[golden-v293-actuals.txt](golden-v293-actuals.txt)）；`replay_scenarios` 2/2（[replay-golden-v293.log](replay-golden-v293.log)） |

## 测试

| 口径 | 结果 |
|---|---|
| 定向 `d4_boss_b2_*` | **4/4**——[directed-b2.log](directed-b2.log) |
| 全量 | **829/0/1**——[engine-native-tests.log](engine-native-tests.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=.scratch/target-d4-b2-gate1b cargo check -p engine-native --all-targets --offline` **exit 0**——[gate1b-all-targets.log](gate1b-all-targets.log) |

独立 `CARGO_TARGET_DIR=.scratch/target-d4-b2`（全量）/ `.scratch/target-d4-b2-gate1b`（Gate）；未 `cargo clean` 共享 target。

## 半成品说明

开工时 `git status` 仅 `host.rs` 未提交改动（上一任 Connection failed）：已含 tag 415–420 枚举、`ALL` 登记、`from_tag`/`key`、TriggerMeta 与 filter 臂并入青木同款。本批续写执行臂、`sixiang_mark_hide` 技能名参数化，并完成 standard/lib/测试/文档。

## B3 建议

1. 白芒3/3x、赤炎3/3x、玄麟3/3x（侧座 reinit，复用青木3 参数表）。
2. 或按 ADR §6 先行 **bianshen4** 死亡换 Boss（阴间系候选池种子化）。
3. HostBehavior 自 **tag 421** 起；无新存档字段则仅升规则版本。
