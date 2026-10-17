# D5 前置原语：无限体力（上游 `Infinity`，2026-10-17）

核验日期：2026-10-17。**规则 v302 / 存档 v87 不变**（无新字段——哨兵复用既有
`hp`/`max_hp` 字段）；14 golden **零扰动**（无版本变化、未重录）。

本目录交付 **D5 / `T11-BOSS-ZHUGE-LIANG` 的第一块前置原语**：上游 Boss 诸葛亮
（祭风卧龙）体力为 JS `Infinity`，装载器此前直接拒绝（[T9 差异登记](../2026-10-17-t9-lx-content-hooks/README.md) 第 1 条）。
本轮把「无限体力」做成原生可装载、可对局、可存档的语义；**D5 与 Boss 诸葛亮条目
仍开放**（余四技 `xiangxing`/`yueyin`/`fengqi`/`gaiming` 与 Boss 装配）。

## 语义与接线

| 面 | 上游（`Infinity`） | 原生 |
|---|---|---|
| 表示 | `player.maxHp = player.hp = Infinity` | `state::INFINITE_HP = i32::MAX as u32`（2147483647）哨兵；`PlayerState::infinite_hp()` = `max_hp == INFINITE_HP` |
| 装载 | `boss_zhugeliang:['male','shen',Infinity,[...]]` | `bridge_generals` 接受 `"hp": "Infinity"` 或哨兵数值 2147483647 → `ExtensionGeneral.max_hp = INFINITE_HP`；`null`/`0`/`0.5`/`256`/`"infinity"`/`"∞"` 仍拒绝 |
| 伤害 | `hp -= num`（Infinity 不变） | `commit_damage`：伤害照常结算与记录，**体力不变**（`hp` 保持 `== max_hp` ⇒「已受伤」类判据恒假，与上游一致） |
| 失去体力 | 同上 | `commit_lose_hp`：同上 |
| 回复 | `hp += num`（Infinity 不变） | `Ctx::recover`：不改体力（不产生 `HpRecovered` 日志/`Recover` 事件） |
| 体力上限减 | Infinity − n = Infinity | `Ctx::lose_max_hp`：不减（记 `MaxHpLost{amount:0}`） |
| 濒死 | `hp <= 0` 永不成立 | 濒死判定读 `hp` ⇒ 永不进入；救援机不询问 |
| 手牌上限 | 基数 = 体力（Infinity） | `hand_limit` 基数取哨兵值（等效无上限，弃牌阶段不要求弃牌） |
| 投影 | UI 显示 ∞ | 视图直接暴露哨兵数值（`hp`/`max_hp`）；宿主可映射为 ∞——**登记差异**：不新增视图字段（避免扩大快照/协议面） |
| 存档 | — | 哨兵随 `hp`/`max_hp` 自然往返；读档校验与既有规则版本门禁不变 |

代码：`crates/engine-native/src/state.rs`（哨兵 + `infinite_hp()`）、
`crates/engine-native/src/kernel.rs`（`commit_damage`/`commit_lose_hp` 门控）、
`crates/engine-native/src/machine.rs`（`recover`/`lose_max_hp` 门控）、
`crates/content-native-loader/src/lib.rs`（`bridge_generals` 接受 `Infinity`）。

## 验证

```
cargo test --offline -p engine-native --test t9_d5_surface
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（T9/D5 面） | **5 passed / 0 failed**（[targeted.log](targeted.log)）：`t9_infinity_hp_maps_to_sentinel`（字符串与哨兵数值两种写法）、`t9_invalid_hp_values_are_rejected`（6 个非法值仍拒绝）、`t9_zhuge_and_ylyg_skills_stay_unmapped`（10 技仍暂缓）、`t9_unknown_mapping_is_not_a_noop`、`infinite_hp_survives_damage_and_recovery`（内核正反例） |
| engine-native 全量 | **901 passed / 0 failed / 1 ignored**（exit 0；1 ignored＝既有诊断探针）——[engine-native-tests.log](engine-native-tests.log) |
| 装载三包 | **66/0/0**——[deps-tests.log](deps-tests.log) |
| 14 golden | `replay_scenarios` **2/2**，零扰动（无版本变化、未重录）——[replay-golden.log](replay-golden.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

内核回归要点（`infinite_hp_survives_damage_and_recovery`）：无限角色受 3 点伤害后
`hp == INFINITE_HP`、`max_hp` 不变、`alive` 且未濒死；回复 5 点体力不变；手牌上限
基数 > 10⁶；**对照**普通角色体力 1 受 3 点伤害照常阵亡；投影与存档往返保持哨兵；
AI 规划器面对哨兵不溢出且两次结果一致。

## 剩余工作（D5 / `T11-BOSS-ZHUGE-LIANG`）

1. **四技迁移**：`xiangxing`（禳星，含 `hlss_xiangxing1..3`/`_hp` 子技与「3 星」存储）、
   `yueyin`（隐月 + `skipDamage.x1..x3` 表）、`fengqi`（起风）、`gaiming`（改命）——
   定义在 liuli `character/extra.js`（可读源码），带跨技存储与 RNG 分布，属字段批 + 多机批。
2. **Boss 装配**：`boss_zhugeliang` 条目 + Boss 关卡流程接入（D4 `BossStageState`）+ 禁将/翼
   登记；`ylyg*`/`mbcm*` 外部依赖按台账继续登记差异。
3. **龙血玄黄本体**：`mbqianlong`/`_CMeffects`（可读版 [LX.1](../2026-10-08-e01-longxue-lx1/extension.readable.js)）与 `mb_caomao` 装配。

## 影响

- 登记表 `D5-LX-MIGRATION` 与 `T11-BOSS-ZHUGE-LIANG` **保持 open**（前置原语已交付，
  条目描述与备注补记本轮进度）；无规则/存档版本变化，无 golden 变化。
- 台账 L29 行补记原语落地与剩余面；进度主稿新增切片行。
