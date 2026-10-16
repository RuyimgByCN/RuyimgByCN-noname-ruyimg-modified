# 2026-10-16 D4 Boss 关卡流程 · 批次 B1（青木隐藏链打样）

**核验日期**：2026-10-16　**待做 ID**：`T11-BOSS-HIDECHAIN-8`（本批仅青木四技）　**ADR**：[0783](../../../adr/0783-boss-stage-flow-subsystem.md)　**前置**：[B0](../2026-10-16-d4-boss-b0/README.md)（`9d95f7e4`）

## 结论

最短路径落包 `boss_qingmu2` / `boss_qingmu2x` / `boss_qingmu3` / `boss_qingmu3x`（HostBehavior tag **411–414**），复用 B0 的 `change_boss` / `reinit_boss_seat` / `add_boss_fellow` / 座位环。参数化表 `SixiangChainParams` 为白芒/赤炎/玄麟六技打样。规则 **291→292**；存档不变 **v79**。本批不做白芒/赤炎/玄麟、翼编排、还人、孟婆乱入、bianshen、D5/`ylyg*`、映姬、地藏大愿。

## 技能与 tag

| 技能名 | HostBehavior | tag | 上游 | 原生承载 |
|---|---|---|---|---|
| 青木 | `BossQingmu2` | **411** | dieBegin hide | Death BeforeTriggers `Timing::First`；stage.kind→SixiangChain + 日志 |
| 青木·转阶 | `BossQingmu2x` | **412** | dieAfter priority -10 → 勾芒+两书精+全场回满+挑战者重整 | Death BeforeTriggers `Normal` priority -10；`death_cancelled` + 重挂 + fellow + 挑战者清理 |
| 青木3 | `BossQingmu3` | **413** | dieBegin 三人 hide | First；本体+环邻日志 |
| 青木3·转阶 | `BossQingmu3x` | **414** | dieAfter → 太昊+侧座勾芒/书精 | Normal -10；侧座 `reinit` + fellows 登记 |

条目：青龙挂 `[神裔,雷击,青木,青木·转阶]`；勾芒挂 `[神裔,不臣,青木3,青木3·转阶]`。

## 登记差异

1. **死亡时序**：原生 Death 仅 BeforeTriggers 收集触发（无 dieAfter 窗）；转阶技以 `Timing::Normal` priority **-10** 落在同窗晚段，并以 `death_cancelled` 拦截 DeathMachine（空洞印同款基建）。
2. **changeBoss 实体**：上游删旧建新；原生座位保留 + 清区摸 4。
3. **changeSeat 八座观战位移**：不建模（seat_ring 不变）。
4. **addBossFellow 座位插入**：固定队尾（B0 既有登记）。
5. **hide 视觉**：日志 + stage 标记；无 UI 观战/不可交互持久字段（ADR 0783 表现边界）。

## 版本与 golden

| 项 | 值 |
|---|---|
| `STANDARD_RULESET_VERSION` | 291→**292** |
| `ARCHIVE_FORMAT_VERSION` | **79**（不变） |
| HostBehavior 最高 tag | **414** |
| 版本回退探针 | 新代码 + 规则回退 292→291 + 旧 golden → 14 场全绿（[probe-291-newcode.log](probe-291-newcode.log)）——位移唯一来源=规则版本字段 |
| 14 golden | 程序化重录（[golden-v292-actuals.txt](golden-v292-actuals.txt)）；`replay_scenarios` 2/2（[replay-golden-v292.log](replay-golden-v292.log)） |

## 测试

| 口径 | 结果 |
|---|---|
| 定向 `d4_boss_b1_*` | **3/3** |
| 全量 | **825/0/1**——[engine-native-tests.log](engine-native-tests.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=.scratch/target-d4-b1-gate1b cargo check -p engine-native --all-targets --offline` **exit 0**——[gate1b-all-targets.log](gate1b-all-targets.log) |

独立 `CARGO_TARGET_DIR=.scratch/target-d4-b1`（全量）/ `.scratch/target-d4-b1-gate1b`（Gate）；未 `cargo clean` 共享 target。

## B2 建议

1. 以 `SixiangChainParams` 参数化落包白芒2/3、赤炎2/3、玄麟2/3（六技同构；赤炎2 额外全场回满+炎灵 fellow+复活挑战者已有臂可复用）。
2. 或先行 **bianshen4** 死亡换 Boss（ADR §6 B2），复用本批 `death_cancelled` + `reinit` 骨架与种子化候选池。
3. HostBehavior 自 **tag 415** 起顺序编号；无新存档字段则仅升规则版本。
