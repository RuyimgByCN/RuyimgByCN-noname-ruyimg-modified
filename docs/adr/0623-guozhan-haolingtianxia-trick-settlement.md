# ADR 0623：国战锦囊「号令天下」（gz_haolingtianxia）结算

> 承接 [ADR 0622](0622-guozhan-wenheluanwu-trick-settlement.md)：国战 12 锦囊结算第十张——
> gz_haolingtianxia（号令天下）。全新 CardKind（51）＋ `HAOLINGTIANXIA_CUSTOM_CHOICE`
> 全员依次二选一链；`RULE_VM_ABI_VERSION` 不变（无新 EffectRequest 变体）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

guozhan.js:65 `gz_haolingtianxia`（黑桃 12，牌堆 1 张）：
- `filterTarget: !target.isMinHp()`——对一名**体力值不为全场最少**的其他角色使用。
- `content`：除目标外所有角色按座次依次（`sortBySeat`）二选一：
  1. **出杀**：非魏势力先弃一张牌（`chooseToDiscard("he", true)`），视为对目标使用一张
     【杀】；魏势力免弃牌。
  2. **弃/得**：弃置目标一张「he」牌（`discardPlayerCard`）；魏势力改为获得目标一张牌
     （`gainPlayerCard`）。
- 选项按可用性过滤：出杀需 `canUse(杀, 目标)` 且（魏或本人有可弃 he 牌）；弃/得需目标
  有 he 牌；两者皆不可用则跳过该角色；单选项仍可取消（`cancel2`）。
- `current.hasSkill("diaohulishan")` 跳过——引擎无按名技能查询，作为近似省略。

## 做了什么

### CardKind + CardKindTag + trick_effect（8 件套新增尾件）

- `state/cards.rs`：`CardKind::TrickHaolingtianxia = 51`（尾件，序数稳定）＋ from_tag 臂
  ＋ is_trick 列表。
- `engine-api/src/view.rs`：`CardKindTag::TrickHaolingtianxia` ＋ projection.rs 映射。
- `command/play.rs`：`trick_effect::HAOLINGTIANXIA = 27`。

### 分发（command/play.rs）

- 目标校验：单目标、非自己、存活、且 `hp > 全场存活最小 hp`（等价 `!isMinHp`）；
  经 `use_independent_trick_targets` 入队（含无懈窗口）。
- `skill_card_target_spec`：`select_one`（同 hp 过滤）。

### 结算：全员依次二选一链

链状态以**独立锦囊队列超集**持久化：
`[card, actor, kind, suit, HAOLINGTIANXIA, index, len, ...链角色(座次), 目标, ...原始args尾]`。
`advance_independent_trick_queue` 在 `index < len` 分支为 `HAOLINGTIANXIA` 特判——
不走无懈窗口，转 `advance_haolingtianxia_player`（跳过死亡/无可用选项角色，打开当前角色
的选项 choice）。链完成时经 `finish_independent_trick_target` 收尾原始锦囊队列。

- **Stage 1**（Custom，选项 [1,2]，可取消）：① 出杀 ② 弃/得目标牌。
  - ①：魏直接伤害；非魏先开 Stage 2（mode 1）选本人一张「he」牌作费用。
  - ②：开 Stage 2（mode 2 弃 / mode 3 魏得），选项集为目标可弃/可得的「he」牌。
- **Stage 2**（SelectCards）：mode 1 弃费用后伤害；mode 2 `DiscardCards{目标}`；
  mode 3 `MoveCards{→ 本人手牌}`。每步经 `run_effects_then_resume_trick_queue`
  （`IndependentTrickQueue` resume）推进链到下一角色——若效果挂起（伤害触发 choice），
  完成后由 `resume_interrupted_continuation` 续链。

魔数 `HAOLINGTIANXIA_CUSTOM_CHOICE = 0x484C_5458` 复用 `ContinuationKind::Custom`
分发（payload[0]），不新增 ContinuationKind 变体。

### ⚠️ 出杀近似（重要）

上游①为「视为使用一张【杀】」——目标可响应【闪】。引擎杀管线
（`handle_card_with_policies_and_flags` → `ResumePlayedSha`/`FinishPlayedSha`）**独占
resume 单槽**且无条件覆写调用方已挂的 `IndependentTrickQueue` resume（play.rs），
杀完成后无父级续链钩子（kefuzhongyuan ADR 0617 的杀分支同理：单目标局内队列静默停滞，
不可观测）。因此**号令天下①以对目标造成 1 点普通伤害近似**（无闪响应、不消耗杀使用
次数、不触发杀专属卡牌修饰）。已在 resolver 注释与本节双重记录。

### 牌堆映射 + AI + query

- `content/deck.rs`：`"gz_haolingtianxia"`（黑桃 12）由 `Other` →
  `CardKind::TrickHaolingtianxia`。
- `engine-ai/legal.rs`：存在 `hp > 全场最小` 的其他角色时给出合法目标。
- `query/mod.rs`：牌名长度 4（号令天下）＋ trick 列表并入。

### 单测

`crates/engine-core/tests/guozhan_trick_haolingtianxia.rs` 3 项，全部通过：
1. 非魏出杀：弃费用牌 → 目标掉 1 血，费用牌离手；
2. 魏获得：目标一张牌移入魏当前角色手牌；
3. 3 人局链按座次访问 actor → p2，出杀命中后 p2（魏）获得目标牌，链收尾。

## 验证

| 检查 | 结果 |
|---|---|
| 新单测 | **guozhan_trick_haolingtianxia 3 passed** ✅ |
| conformance | 全量重跑中（国战向量可能因牌堆 kind 变化漂移，如漂移则本域重钉） |
| 失败集合 | 与 claude-a-failures-whlw.txt（1：original_mobile_xinpingkou）comm 对比，须零新增 |
| `cargo build` | engine-core / engine-ai 无编译错误 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——新增 CardKind/trick_effect/自定义 choice
均为序数稳定尾件，无新 EffectRequest 变体；`advance_independent_trick_queue` 特判分支
不改队列格式。

## 后续

1. **chiling（叱咤）**：全局临时技能（待实现）。
2. **zhibi（知己知彼）**：纯信息揭示——建议维持元数据登记（ADR 0621 同判）。
3. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
4. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
