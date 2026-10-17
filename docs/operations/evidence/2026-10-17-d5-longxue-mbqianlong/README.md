# D5 龙血玄黄批 117：潜龙道心值（`mbqianlong`）+ 卫统（`mbweitong`）（字段批 v305/存档 v90，2026-10-17）

- 日期：2026-10-17
- 范围：**龙血玄黄扩展的规则面**（`extension.js` 自注册的两技能之一 `mbqianlong`；`_CMeffects` 为纯显示面）+ 宿主武将手杀曹髦（`mb_caomao`，liuli `character/mobile.js:44`）的转换包
- **规则 v304→305、存档 v89→90**（`PlayerState` 尾部追加 `mbqianlong_marks`）
- 上游回读：`archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js`（可读化版 [LX.1](../2026-10-08-e01-longxue-lx1/extension.readable.js)：`lib.skill.mbqianlong` 581133 起、`_CMeffects` 536064 起）、`character/mobile.js`（`mb_caomao` 条目、`mbweitong` 17635–17659）

## 上游规则面与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 潜龙 `mbqianlong`（tag 469） | `trigger {global:'phaseBefore', player:'enterGame'}` + `filter event.name!='phase' \|\| game.phaseNumber==0`；`content`：`num=20`（另有魏势力持主公技卫统者 `num=60`）→ `addMark('mbqianlong', num)` 上限 99 → 刷新解锁 | `MbQianlong`（PhaseBegin + `extra_events [CardsDrawn, CardsMoved, Damage]`）：开局面 +20/+60；获得牌 +5；受伤 +10×伤害；造成伤害 +15×伤害；上限 99；随后 `mbqianlong_refresh` |
| 潜龙·获得牌（子技 1） | `gainEnd`：+5（`marks<99`） | 同一行为 `CardsDrawn{player==owner}` / `CardsMoved{to==owner}` 面（合并子技，登记差异：**开局摸牌不计**——上游 `gameDraw` 不触发 `gainEnd`，原生以「首个阶段帧之前不计」近似） |
| 潜龙·受伤（子技 2） | `damageEnd`：+`num*10` | `Damage{target==owner, amount>0, !prevented}` 面 |
| 潜龙·造成伤害（子技 3） | `damageSource`：+`num*15` | `Damage{source==owner}` 面 |
| 潜龙·刷新（子技 5） | `removeAdditionalSkill('mbqianlong')` 后按阈值 `addAdditionalSkill`：25 清正 / 50 酒诗 / 75 放逐 / 99 绝进（`if(!lib.skill[s]) return` 存在性守卫） | `mbqianlong_refresh`：逐阈值检查目录技能是否存在，存在才授予/撤销实例（外部包未转换时不虚构空操作） |
| 卫统 `mbweitong`（tag 470） | 主公技：开局（同 phaseNumber==0）且自己持潜龙、另有魏势力持本技者 → `game.log('修改了〖潜龙〗的初始道心值')`（+60 由潜龙内容判定） | `MbWeitong`（PhaseBegin 首阶段帧 + 魏势力/持技条件）：登记面，无机器效果 |
| `_CMeffects` | `gameStart`/`dieAfter` 建 DOM（cmarea/cmjian）+ Spine 动画，条件 `game.me.name` 为 mb_caomao×2 | **登记差异**：纯显示面不迁移（无 DOM/动画宿主） |
| `dieAnimation`（`$die` 包装） | 死亡时播放 Spine 动画 | **登记差异**：显示面不迁移 |

### 时点登记差异（开局面）

上游首个 `phaseBefore` 是准备阶段（`game.phaseNumber==0`）；原生建局时准备阶段开始为隐式提交（无帧），**游戏首个阶段帧是摸牌阶段开始**（`turn.phase_number` 由 0 → 1）。故开局面在首个阶段帧触发（时点后移一个阶段），并以 `phase_number == 1` 精确限定「只触发一次」。

## 转换包

`packages/extensions/longxue-upstream-liuli`（`upstream-liuli/longxue@1.0.0`）：

- 武将 `mb_caomao`（男/魏/3 体力/主公）：`mbqianlong → host:mbqianlong`、`mbweitong → host:mbweitong`（映射 2/2）。
- 包级 `skills` 段登记四个**外部依赖**（`mbcmqingzheng`/`mbcmjiushi`/`mbcmfangzhu`/`mbjuejin`，`mapping: null` + 理由）：它们定义在 liuli `character/mobile.js`（手杀包），不属于龙血玄黄扩展；道心值刷新按上游存在性守卫跳过，不虚构空操作。

## 验证

```
cargo test --offline -p engine-native --test d5_longxue_mbqianlong
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native-wasm --lib
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（龙血玄黄） | **5 passed / 0 failed**（[targeted.log](targeted.log)）：包装载与外部依赖登记；开局面 20/60（含另一魏势力持卫统）；累加面 +5/+10×/+15× 与上限 99、满值不再增长；阈值授予（目录存在才授予：20 未授予 → 25 授予）；卫统登记面 + 开局面只触发一次 + 存档往返 |
| engine-native 全量 | **914 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| wasm | **20/0**（[wasm-tests.log](wasm-tests.log)） |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| 14 golden | 重录 + 版本回退探针（305→304 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v305.log](replay-golden-v305.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 边界与剩余

- **外部技能面**：四个 `mbcm*`（清正/酒诗/放逐/绝进）与 `mb_epic_caomao` 属 liuli 手杀包（`character/mobile.js`，134/159/100/98 行），未转换——道心值的解锁目标在目录缺失时按上游守卫跳过；如需完整体验需另立手杀包来源批（该包不在 37 来源清单内，作为外部依赖登记）。
- **`ylyg*` 外部面**：属 `T11-BOSS-PANGTONG-EXTERNAL`（庞统涅槃/浴火），本批不动。
- **显示面**：`_CMeffects`/`dieAnimation`/进度条与剑特效不迁移（无 DOM/Spine 宿主），登记差异。
