# 2026-10-17 T9 / D5：自动选将后的龙血 content 钩挂

核验日期：2026-10-17。规则 `STANDARD_RULESET_VERSION` **295**。存档 `ARCHIVE_FORMAT_VERSION` **80**。种子 `20261016`（mulberry32）。运行副本 `scratch/legacy-harness/liuli`（archive 只读）。本目录不覆盖 [2026-10-16 停在亮牌的样本](../2026-10-16-t9-lx-battle-trace/README.md)。

## 命令

```bash
cd scripts/legacy-harness
node sample-lx-battle.mjs
cargo test --offline -p engine-native --test t9_d5_surface -- --test-threads=8
```

`sample-lx-battle.mjs` 在脚本前写入 `mode=boss`、`continue_name_boss={boss:false,links:[mb_caomao,caocao,liubei]}`、`noname_0.9_boss={current:boss_zhugeliang}`、`noname_0.9_nodb=true`。这是上游「再战」使用的同一组配置，角色经 `player.init` 装配技能，不是事后 `addSkill`。`setContent` 在 `lib.element.event` 出现后包一层，记录解析后的 `content` 与当时的随机流计数。

## 结果

| 项 | 值 |
|---|---|
| 场上角色 | `mb_caomao`、`caocao`、`liubei`、`boss_zhugeliang` |
| 诸葛亮体力 | `Infinity` / `Infinity`（日志与 [mid-restore.json](mid-restore.json)） |
| 诸葛亮技能 | `xiangxing`、`yueyin`、`fengqi`、`gaiming`；`storage.xiangxing=7`，`xiangxing_count=0` |
| 曹髦技能 | `mbqianlong`；中途快照 `storage.mbqianlong=20` |
| `skill.content` 命中 | **17**（`_CMeffects`、`mbqianlong`、`mbqianlong_refresh`、`fengqi`、`mbqianlong_2`） |
| 钩子内命令 | `mbqianlong`→`addMark`（rng 332）、`fengqi`→`chooseButton`（rng 347）、`mbqianlong_2`→`addMark`（rng 396） |
| 中途事件 | 快照时 `useCard` step 8，玩家 `boss_zhugeliang` |
| 结束时随机计数 | 396 |
| `ylyg*` 运行时 | `lib.character.ylyg_pangtong` 不存在，钩子名 0。静态图仍是 10 hits |
| 装载器测试 | `t9_d5_surface` **3 passed / 0 failed**（[t9-loader-tests.log](t9-loader-tests.log)） |

原始日志：[sample-log.txt](sample-log.txt)、[runtime-trace.json](runtime-trace.json)、[commands.json](commands.json)。

停机时 `lib.storage.current` 已不是字符串。选将是否生效看场上四名角色：三名参战与 `continue_name_boss.links` 一致，BOSS 是 `boss_zhugeliang`。

## 与原生的对照（登记差异，不是已支持）

1. **体力 Infinity**。上游本局体力保持 Infinity，技能照常进入 `content`。原生 `generals[].hp` 只接受 1–255 的整数；`"Infinity"`、`null`、`0`、`0.5`、`256` 由 `bridge_generals` 拒绝（`t9_infinity_hp_is_rejected`）。没有把 Infinity 收成有限体力。
2. **四技与 ylyg\***。`mapping` 缺省时计入 `unmapped_skills`，不进入 `host_skills`，也不写入武将技能表（`t9_zhuge_and_ylyg_skills_stay_unmapped`，10 条）。未知键 `xiangxing` 返回 `UnknownBehaviorKey`，不是空操作（`t9_unknown_mapping_is_not_a_noop`）。
3. **命令**。上游在钩子内实际调用了 `addMark` 与 `chooseButton`，并走到 `useCard`。原生没有对应宿主行为，这些调用不会发生。
4. **事件**。上游事件名包括 `_CMeffects`、`mbqianlong`、`mbqianlong_refresh`、`fengqi`、`mbqianlong_2`。原生标准局面没有这些帧。
5. **随机流**。上游序列是种子 20261016 的 mulberry32，上表给出每次命令时的 `draws`。原生引擎 RNG 没有这条技能序列，不能把两个计数当成同分布。
6. **中途恢复**。快照含 Infinity 体力、`xiangxing` 星数和 `mbqianlong` 标记。原生存档没有这些字段；本目录不把该快照重放成原生对局。
7. **ylyg\***。琉璃运行副本的 `lib.character` 没有 `ylyg_pangtong`，自动选将无法装配。静态触发图来自 `scratch/legacy-harness/noname` 的 `offline_yanling.js`（10 hits）。原生侧与四技一样保持未映射。

`yueyin` 在上游没有 `content`（只有 `skipDamage` 表）。`xiangxing`/`gaiming` 的 `content` 本局没有进入（需要伤害或判定）。上面第 2 条仍把它们列为未映射，不记成已触发或已支持。

D5（2026-10-16）是立项迁移。本轮把迁移做到可对照的上游轨迹，并把无法等价的体力与技能面登记为差异。引擎全量、Gate 1b 与其余五门不在本目录。
