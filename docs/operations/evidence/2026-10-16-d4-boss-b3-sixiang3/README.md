# 2026-10-16 D4 Boss 关卡流程 · 批次 B3（白芒/赤炎/玄麟 3/3x）

**核验日期**：2026-10-16　**待做 ID**：`T11-BOSS-HIDECHAIN-8`（本批 3/3x 六技；四象隐藏链 2+3 全系齐）　**ADR**：[0783](../../../adr/0783-boss-stage-flow-subsystem.md)　**前置**：[B2](../2026-10-16-d4-boss-b2-sixiang/README.md)（`3c6ed8f6`）

## 结论

落包 `boss_baimang3/3x`、`boss_chiyan3/3x`、`boss_xuanlin3/3x`（HostBehavior tag **421–426**），侧座 `reinit` 复用 B1 青木3 `SixiangChainParams` 参数表。规则 **293→294**；存档不变 **v79**。本批不做翼编排、还人、bianshen4、D5。

## 技能与 tag

| 技能名 | HostBehavior | tag | 上游 | 原生承载 |
|---|---|---|---|---|
| 白芒3 | `BossBaimang3` | **421** | dieBegin 三人 hide | Death BeforeTriggers `Timing::First` |
| 白芒3·转阶 | `BossBaimang3x` | **422** | →少昊+侧座蓐收/明星+明星 fellow | Normal priority -10；`death_cancelled` + 链 |
| 赤炎3 | `BossChiyan3` | **423** | dieBegin 三人 hide | First |
| 赤炎3·转阶 | `BossChiyan3x` | **424** | →炎帝+侧座祝融/焰灵 | Normal -10 |
| 玄麟3 | `BossXuanlin3` | **425** | dieBegin（上游同侧邻座条件） | First；三人 hide 近似青木3 |
| 玄麟3·转阶 | `BossXuanlin3x` | **426** | →颛顼+共工 fellow+次座玄冥 | Normal -10 |

条目：蓐收挂 `[神裔,星囚,白芒3,白芒3·转阶]`；祝融挂 `[神裔,星夏,赤炎3,赤炎3·转阶]`；共工挂 `[神裔,绝红,玄麟3,玄麟3·转阶]`（上游玄冥亦挂 `boss_xuanlin3`——原生仅共工承载，同青木3 只挂阶段 2 Boss）。

`SixiangChainParams` 侧座先于 `add_fellows`（队尾插入会改 `ring_prev`，否则白芒3 侧座+额外 fellow 互踩）。

## 登记差异

与 B1/B2 同族：无 dieAfter 窗、无八座 changeSeat、changeBoss 座位保留清区摸 4、hide 为日志/stage 近似。另：

1. **白芒3x / 玄麟3x 侧座 maxHp--**：未建模（参数表无该字段）。
2. **玄麟3 hide 同侧条件**：近似为三人 hide（青木3 同款）。
3. **玄冥条目不挂玄麟3**：仅共工承载（青木3 同款）。

## 版本与 golden

| 项 | 值 |
|---|---|
| `STANDARD_RULESET_VERSION` | 293→**294** |
| `ARCHIVE_FORMAT_VERSION` | **79**（不变） |
| HostBehavior 最高 tag | **426** |
| 版本回退探针 | 新代码 + 规则回退 294→293 + 旧 golden → 14 场全绿（[probe-293-newcode.log](probe-293-newcode.log)）——位移唯一来源=规则版本字段 |
| 14 golden | 程序化重录（[golden-v294-actuals.txt](golden-v294-actuals.txt)）；`replay_scenarios` 2/2（[replay-golden-v294.log](replay-golden-v294.log)） |

## 测试

| 口径 | 结果 |
|---|---|
| 定向 `d4_boss_b3_*` | **4/4**——[directed-b3.log](directed-b3.log) |
| 回归 `d4_boss_b*`（B0–B3） | **17/17**——[directed-b1b2b3.log](directed-b1b2b3.log) |
| 全量 | **833/0/1**——[engine-native-tests.log](engine-native-tests.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=.scratch/target-d4-b3-gate1b cargo check -p engine-native --all-targets --offline` **exit 0**——[gate1b-all-targets.log](gate1b-all-targets.log) |

独立 `CARGO_TARGET_DIR=.scratch/target-d4-b3`（全量）/ `.scratch/target-d4-b3-gate1b`（Gate）；未 `cargo clean` 共享 target。

## B4 建议

1. **bianshen4** 死亡换 Boss（阴间系候选池种子化；复用 `death_cancelled` + `reinit`）。
2. **翼编排**（`boss_qingyi` / `boss_chiyi` / `boss_baiyi` 轮次硬绑子技）。
3. **还人**（秦广王关卡面；可达性依赖映种排除边界另登记）。
4. HostBehavior 自 **tag 427** 起；无新存档字段则仅升规则版本。
