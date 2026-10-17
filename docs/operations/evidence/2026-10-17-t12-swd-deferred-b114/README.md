# T12 swd deferred 消解批 114：锦鳞（字段批 v302/存档 v87，2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 461/462）——**150/184 技条目已映射**（deferred 33→32）
- **规则 v301→302、存档 v86→87**（`PlayerState` 尾部追加 `jinlin_used: bool`）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（jinlin 1480–1542 行、jinlin2 1543–1559 行逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 锦鳞 jinlin（司空蔷 `swd_situqiang`） | `enable:"phaseUse"` + `limited:true`（`awakenSkill`）；`filterTarget:true`、`selectTarget:[1,Infinity]`：对任意名角色各 `changeHujia(3)` 并 `addSkill("jinlin2")`、`storage.jinlin2 = 3` | `SwdJinlin`（tag 461）=主动技；`activate()` 门＝`!PlayerState.jinlin_used`；Act→`SelectPlayers`（1..全部存活，含自己）→TargetChoice（逐目标 `change_hujia(+3)` + 授予 `swd_jinlin2` 实例 `rounds=3`）；发动时置 `jinlin_used` |
| 锦鳞·退 jinlin2 | `trigger:{player:"phaseBegin"}`，silent：护甲 >0 则 -1；计数 -1；护甲 === 0 或计数 === 0 时移除本技 | `SwdJinlin2`（tag 462）=PhaseBegin（持有者任意阶段开始）：`change_hujia(-1)`（>0 时）+ `rounds-1`；`hujia == 0 || rounds == 0` → 移除实例 |

- 目录声明：包 `skills` 段新增 `swd_jinlin2 → host:jinlin2`（运行时授技的目标名）；`swd_situqiang` 的 `jinlin` 由 `null` 改挂 `host:jinlin`。
- 一次性门用 `PlayerState.jinlin_used`（与花好 `huahu_used`、贯日 `guanri_used` 同族），技能实例保留在册（上游 `awakenSkill` 只禁用不注销）。
- `rounds` 复用既有 `SkillInstance.rounds` 计数（食物倒计时同字段；本技不触发食物扣减路径）。

## 字段批

- `PlayerState.jinlin_used: bool`（state.rs 尾部）；入 `canonical_hash`（engine.rs 玩家字段尾部）。
- 存档：`archive.rs` 写/读尾部追加；`ARCHIVE_FORMAT_VERSION` 86→**87**；`STANDARD_RULESET_VERSION` 301→**302**。
- 14 golden 重录（[replay-golden-v302.log](replay-golden-v302.log)）；版本回退探针（302→301 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）证明位移含版本字段。
- `d4_boss_b0_archive_format_v79_rejects_v78_without_migration` 断言同步 86→87。
- 键表守护 `host_behavior_keys_bijective` 更新为 **459**（tag 461/462）；`ALL` 数组长度同步。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算。

## 验证

- **定向回归** `swd_deferred_jinlin_limited_hujia_and_phase_ticks`：发动选自己与 1 号 → 双方 +3 护甲、`rounds=3`，未选目标 0 护甲且无实例；再次发动 `InvalidChoice`（limited）；0 号阶段开始 → 护甲/计数各 -1，他人阶段不递减；1 号自己的阶段开始才递减；0 号两次阶段开始后护甲与计数同时归零、实例移除；移除后继续推进不再递减。1 passed / 0 failed。
- **swd 套件**：`cargo test --offline -p engine-native --test g1_shahai_swd` 108/0/0。
- **全量**：engine-native **899 passed / 0 failed / 1 ignored**（exit 0）——[engine-native-tests.log](engine-native-tests.log)。
- **装载面**：`cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast` **66/0/0**——[deps-tests.log](deps-tests.log)。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets`（独立 target 目录）exit 0——[gate1b-all-targets.log](gate1b-all-targets.log)。
- **14 golden**：`replay_scenarios` 2/2（按上述依据重录）。

## 结论

- **150/184 映射，deferred 33→32**；**司空蔷（`swd_situqiang`）整将闭环**（`fengze`+`lingyue`+`jinlin`）——79 将中 52 将全映射。
- `T12-SWD-DEFERRED-SKILLS` 仍开放（余 32 个未映射技名）。
