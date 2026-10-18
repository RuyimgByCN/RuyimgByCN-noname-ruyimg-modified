# U08 英雄杀 批 2：门神（`yxs_menshen`）+ 门神·清除（`yxs_menshen3`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**U08 英雄杀**（`apps/core/extension/英雄杀/character/skill.js`）
- 本批：`yxs_menshen`（tag 528）+ `yxs_menshen3`（tag 529）+ `yxs_menshen2`（tag 530，标记占位）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化——`vm_aux.marks` 复用）
- 台账：U08「部分重构（批 2/N）」

## 上游语义与原生接线

| 技能 | 上游 | 原生 |
|---|---|---|
| 门神 `yxs_menshen`（748 B） | `trigger {player:'phaseEnd'}`（`group:'yxs_menshen3'`）——`players.length>1` 时选一名其他角色 → `target.storage.yxs_menshen2 = player` + `target.addSkill('yxs_menshen2')` | `YxsMenshen`（tag 528）：`PhaseEnd`（自己）可选确认门 → 过滤存活 >1 → `SelectPlayers`（其他角色）→ 目标获得 **`yxs_menshen2` 实例** + 标记 = 施予者座位 |
| 门神·清除 `yxs_menshen3`（406 B） | `trigger {player:['phaseBegin','dieBegin']}` silent——任一场上有 `yxs_menshen2` 时全场 `removeSkill` | `YxsMenshen3`（tag 529）：`PhaseBegin` + `Death` 锁定 → 过滤「场上存在门神实例」→ 移除全场实例与标记 |
| 门神主体 `yxs_menshen2`（**70 KB**） | 保护效果主体 | `YxsMenshen2`（tag 530）：**被动占位**（主体效果待后续批次，登记差异） |

## 验证

```
cargo test --offline -p engine-native --test u08_menshen_batch2
cargo test --offline -p engine-native --test u08_hero_kill_batch1    # 批 1 回归
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（批 2） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：0 号结束阶段选 1 号 → 1 号获得门神实例且**标记 = 0（施予者座位）**；清除——下个阶段开始时全场门神实例与标记被移除 |
| 定向（批 1 回归） | 2/2 通过（[targeted-regression.log](targeted-regression.log)） |
| engine-native 全量 | **1087 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **94/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 526（tag 528/529/530 追加） |

## 包与生成器

- 包：`packages/extensions/hero-kill-upstream-noname`（5 条技能声明：`yxs_fanji`/`yxs_fanji2`/`yxs_menshen`/`yxs_menshen3`/`yxs_menshen2`）。
- 生成器：`crates/content-runtime/examples/gen_hero_kill_package.rs`（deferred：武将表（外部技能依赖未齐）、其余技能）。

## 登记差异

- **门神主体（70 KB `yxs_menshen2`）未实现**：本批只落地「标记施予 + 清除」机制与其持有实例；保护效果（伤害转移/免伤等）待后续批次。
- 上游 `onremove: true`（移除技时清标记）未建模。
- 武将表（43 条）仍因外部技能依赖未齐而未随包发布。
