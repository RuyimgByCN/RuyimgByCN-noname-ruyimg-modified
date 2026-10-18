# U08 英雄杀 批 1：反击（`yxs_fanji`）+ 反击·红杀直伤（`yxs_fanji2`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**U08 英雄杀**（`archive/upstream/noname/apps/core/extension/英雄杀/`，94 KB；G0 见 [余下 G0](../2026-10-17-remaining-sources-g0/README.md)）
- 本批：转换包 `packages/extensions/hero-kill-upstream-noname`（生成器 `gen_hero_kill_package.rs`）+ 2 个技能
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 台账：U08 由「G0 完成」→ **部分重构（批 1/N）**

## G0 补充（本批实测）

| 面 | 实测 | 结论 |
|---|---|---|
| `character/skill.js` | **仅 5 个 `yxs_*` 定义**：`yxs_fanji2`(241 B)/`yxs_menshen3`(406 B)/`yxs_fanji`(504 B)/`yxs_menshen`(748 B)/`yxs_menshen2`(**70 KB**) | 本扩展自带技能极少；主体为武将表 |
| `character/character.js` | **43 条武将**，技能引用大量**外部技能**（`nvquan`/`qiandu`/`yxsweiyi` 等，本仓他处定义） | 武将表**依赖未齐** → deferred（登记差异） |
| 配置键 | 0 | 无配置面 |

## 上游语义与原生接线

| 技能 | 上游 | 原生 |
|---|---|---|
| 反击 `yxs_fanji`（504 B） | `trigger {player:'damageEnd'}` direct——手牌有【杀】且伤害载体为【杀】/【决斗】→ `addTempSkill('yxs_fanji2','shaAfter')` + `chooseToUse({name:'sha'}, trigger.source)` | `YxsFanji`（tag 526）：`Damage`（自己为承受者）可选确认门 → 过滤（手牌有【杀】且载体为【杀】/【决斗】）→ 授予临时 `yxs_fanji2`（`expires_end_of_turn`）+ `SelectCards`（手牌中的【杀】）→ 发 `CardUsed`（目标=伤害来源） |
| 反击·红杀直伤 `yxs_fanji2`（241 B） | `trigger {player:'shaBegin'}` direct——`get.color(card)=='red'` → `trigger.directHit = true` | `YxsFanji2`（tag 527）：杀机器内联——持技者使用的**红色**【杀】跳过防具判定与出闪询问 |

> 测试要点：手动应答序列（闪窗 → `trigger_confirm` → `yxs_fanji_sha`）；首版因在出杀后立即 `drain` 而把两个窗口一并吃掉，改为逐窗应答后通过。

## 验证

```
cargo test --offline -p engine-native --test u08_hero_kill_batch1
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（批 1） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：1 号对 0 号出【杀】→ 0 号受伤后确认并选【杀】反击 → **来源受 1 点伤害**；对照（未持技无反击选牌窗） |
| engine-native 全量 | **1085 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **94/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 523（tag 526/527 追加） |

## 包与生成器

- 包：`packages/extensions/hero-kill-upstream-noname`（namespace `upstream-noname`，与其它 noname 来源一致；`assets/cards.json` + `assets/config.json`）。
- 生成器：`crates/content-runtime/examples/gen_hero_kill_package.rs`（可重复生成）。

## 登记差异

- 反击的临时技以 `expires_end_of_turn` 承载（上游 `shaAfter` = 本张杀结算后失效）。
- **武将表（43 条）未随包发布**：其技能引用本仓他处定义的外部技能（`nvquan`/`qiandu`/`yxsweiyi` 等），依赖未齐 → 依赖面登记；`yxs_menshen`/`yxs_menshen3`（门神标记机制）与 `yxs_menshen2`（70 KB 主体）待后续批次。
