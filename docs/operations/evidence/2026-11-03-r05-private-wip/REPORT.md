# A2 审计：原生阶段/回合触发 vs 上游 noname 触发名语义

> 只读审计产物。未改动仓库文件，未编译、未运行测试。机器可读版本：同目录 `audit.json`（168 条，字段与本表一致）。

## 0. 核验信息

| 项 | 值 |
|---|---|
| 核验日期 | 2026-11-03 |
| 仓库 | `/private/tmp/noname-r05/worktree`（noname-next 私有分支 r05-private） |
| 基线 | main `29712b2b` ＋ A1 检查点 `a605379c`（Phase::Prepare＋EventKind::TurnBegin） |
| HEAD 漂移 | 审计期间 HEAD 前进到 `953213ab`（r05-b：摸牌阶段 D/R/G 机）。`git diff --quiet a605379c HEAD -- <审计范围>` 通过：`vocabulary/`、`engine-native-content/`、`content-native-loader/`、`kernel/phase.rs`、`phase/{judge,end,play,discard}.rs`、`kernel/events.rs` 与 A1 逐字节相同；`resolution.rs`/`machine.rs`/`phase/draw.rs` 有改动，本文这几处行号按 `a605379c` 引用 |
| 上游 | `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/`（noname/apps/core 与 noname-liuli/noname）。注意 `noname/apps/core/extension/` 被上游 .gitignore 忽略，`rg` 须加 `--no-ignore` |
| 方法 | 解析 metadata 全部 `EventKind::PhaseBegin/PhaseEnd`（主事件与 extra_events）条目 → 逐条提取 filtering 过滤臂 → 按 source_order.rs 注释与 keys.rs 的 host 键在上游定位定义并读 trigger 原文；标准技能、内核直写效果、DSL 装载器同法 |
| 失败证据（他人所跑，非本次执行） | `/private/tmp/noname-r05/evidence/b-10.log`：`g1_wudian::anu::anu_anwugu_afflict_and_limits`（:69）、`g1_shahai_swd::rexue::swd_deferred_rexue_and_liuli`（:298，hp 2≠3）、`nc08_prepare_phase_mapping`（:1193）、`nc08_upstream_identity_start`（:1405）、golden（:846）——与本文 K01／H42 的结论吻合 |

路径缩写：`meta/`=crates/engine-native-kernel/src/vocabulary/host/metadata/，`filt/`=crates/engine-native-content/src/content/host/filtering/，`host/`=crates/engine-native-content/src/content/host/，`std/`=crates/engine-native-content/src/content/standard/，`kern/`=crates/engine-native-kernel/src/kernel/，`nk/`=crates/engine-native-kernel/src/，`loader/`=crates/content-native-loader/src/；上游 `杀海/`=archive/upstream/noname/apps/core/extension/杀海拾遗/，`ext/`=…/apps/core/extension/，`core/`=…/noname/apps/core/，`liuli/`=archive/upstream/noname-liuli/noname/。

## 1. 判定基准

上游 `phase` 事件就是**整回合**（`core/noname/library/element/content.ts:4663-4924`）：

```
phaseBefore(4667) → game.phaseNumber++/回合统计入栈(4670-4736) → phaseBeforeStart → phaseBeforeEnd
→ 翻面检测(4750，命中则 event.cancel()，整回合无 phaseBegin) → phaseBeginStart(4836) → phaseBegin(4841)
→ [phaseZhunbei → phaseJudge → phaseDraw → phaseUse → phaseDiscard → phaseJieshu]（各自 Before/Begin/End/After）
→ phaseEnd(4913) → phaseAfter(4917)
```

因此 `phaseBegin`/`phaseEnd`/`phaseAfter` 每回合各一次；`addTempSkill(x)` 不带 expire 时默认 `{global:["phaseAfter","phaseBeforeStart"]}`（`core/noname/library/element/player.js:11966`）。`player:`/`global:` 决定是谁的回合。

## 2. A1 后原生回合时序（与审计结论直接相关）

```
NextPhase@End → PhaseEnd(End) [Commit: commit_phase(End,false) → emit TurnEnded]
  TurnEnded [Commit: commit_turn_ended — 清旗标/换人/turn.phase=Judge/turn_opened=false → emit TurnBegin(next)]
    TurnBegin [BeforeTriggers：宿主+VM 一起派发 → Commit: phase=Prepare，压 Prepare end、Prepare begin → after 静默]
      Prepare begin [VM 前置窗 → Commit（无入场） → 宿主 after 窗]
      Prepare end   [Commit: commit_phase(Prepare,false)（anwugu/筑城/光塔/护天/灵武…均在此执行）→ emit Judge begin]
        Judge begin [VM 前置窗 → Commit: begin_judge（turn_opened 块：reset_turn_limits、turn_damage=0、一批 Boss 回合开始技、废墟）→ 宿主 after 窗 → Intrinsic: 判定区结算]
      Prepare end 的 AfterTriggers（宿主/VM 的 Phase{Prepare,end} 技在**判定区结算之后**才派发）
    TurnEnded 的 AfterTriggers（上一位角色的 turn_ended 技，此时 turn.current 已是下一位）
  PhaseEnd(End) 的 AfterTriggers（上一位角色的「结束阶段结束」技）
```

据此得出的结构性结论（详见 K21/L1、K55/L2、K22/L3、K10、K25）：

1. **TurnEnded 帧不等价于 phaseEnd，也不等价于 phaseAfter**：它从 Commit 起步，Commit 里先换人、发 TurnBegin；TurnBegin 子树压在其上，所以 TurnEnded 的触发窗要等下一位角色 TurnBegin→准备开始/结束→判定开始（含判定区结算）全部结算完才开。它的 Commit 内联清理（`expire_temporary_skills`、`minwan_targets` 等，events.rs:702-825）时点≈phaseAfter，可保留；但不能拿它的触发窗承载 TURN_END/TURN_AFTER 技。**需要新建在交接前派发的回合结束窗**（phaseEnd、phaseAfter 两段）。
2. **PhaseEnd 帧一律「先提交后触发」**：commit_phase 先压下一阶段 begin（或 TurnEnded），该帧的 AfterTriggers 要等子树结算完（resolution.rs:610-630 @a605379c）。所以 Phase{Prepare,end} 的技能会晚于判定区结算，Phase{End,end} 的技能会晚于下一位角色的回合开始。本次未发现必须落到 ZHUNBEI_END 的条目，但 A2 若要用 Phase{X,end} 或回合结束窗，须先处理这一点。
3. **回合重置仍在判定开始**：`reset_turn_limits`、`cards_used_turn`、`turn_damage` 等在 `begin_judge` 的 turn_opened 块里清零（judge.rs:4-25），晚于 TurnBegin 和准备阶段。TurnBegin/准备阶段里看到的是上一回合的值：once_used 未清、cards_used_turn 还是上一位角色的计数、准备阶段造成的伤害会在判定开始被清掉。**ZHUNBEI_BEGIN 技能迁到准备阶段之前，须先把这批重置挪到 TurnBegin 之前**，否则观星这类 once_per_turn 技会被上回合的 once_used 挡住。
4. **翻面跳过仍在判定开始**（phase.rs:21-33）：A1 后翻面角色会先跑完 TurnBegin 窗和准备阶段开始/结束，再被翻回、跳过。上游的翻面检测在 phaseBegin 之前，命中后整回合没有任何时机。属于 A1 引入的回归（P0）。
5. **TurnBegin 窗期间 `turn.phase==Judge`**（events.rs:767/800 写入，TurnBegin Commit 才改 Prepare）；`rvm_bridge::trigger_ctx_of` 没有 TurnBegin/TurnEnded 臂（rvm_bridge.rs:194 default），VM 技能在这两种帧上拿不到 event_player；首回合建局直接 Judge＋turn_opened=true（engine/construction.rs:282/295），不发 TurnBegin/准备阶段。
6. **灵舞（lingwu）在准备阶段结束提交时会读到上一位角色的 cards_used_turn**（phase.rs:261-282；cards_used_turn 到 begin_judge 才清零）。可能在判定前插入一个出牌阶段，Play 之后 `phase.next()` 是 Discard，判定、摸牌和 turn_opened 回合重置会被整体跳过。属于 A1 引入的风险（P0）。
7. 旧口径更正：native-progress.md 第 575–622 轮把上游 `phaseBegin` 读成「任意阶段开始」（G14），把 `phaseEnd` 读成「每个阶段结束」（metadata 注释 common.rs:15、wudian.rs:106、`GujianFoodLiyutang` 臂注释，以及 `scripts/report_phase_arms.py:366-399` 的等价判定）。按上游源码，两者都是**每回合一次**。由此改出来的「省略 phase」写法（如 `SwdJinlin2` 被当作「G14 正确范本」）在 A1 前已经是每回合 5 次，A1 后变成 6 次。`report_phase_arms.py` 里「判定开始＝回合开始＝准备阶段开始」的假设在 A1 后也不再成立。

## 3. 统计

### 3.1 分节与是否需改

| 分节 | 条目 | 需改 | 保持 | 待决策 |
|---|---:|---:|---:|---:|
| 宿主技元数据（PhaseBegin 48＋PhaseEnd 43 条，按行为拆为 91） | 91 | 52 | 39 | 0 |
| 标准技能 | 14 | 7 | 7 | 0 |
| 内核直写效果／生命周期 | 56 | 45 | 11 | 0 |
| DSL 装载器映射 | 7 | 3 | 3 | 1 |
| **合计** | **168** | **107** | **60** | **1** |

宿主元数据口径：metadata 中 `event: EventKind::PhaseBegin` 47 个分支（`BossQingyi`、`BossChiyi` 合为一个分支，按行为计 48）＋ `SwdHuajin2` 的 extra_events PhaseBegin；`event: EventKind::PhaseEnd` 42 个分支＋`Kunlunjing` 的 extra_events PhaseEnd。按行为去重后为 91 条（Kunlunjing 计一次）。内核一节把 judge.rs turn_opened 块里的 24 个 Boss 技逐条列出；end.rs 中 22 个与上游 phaseJieshuBegin 相符的效果合为 K54 一条。

### 3.2 上游语义分类计数（多面条目按组合原样计）

| 上游分类 | 宿主 | 标准 | 内核 | DSL | 合计 |
|---|---:|---:|---:|---:|---:|
| TURN_END | 32 | 0 | 5 | 0 | 37 |
| TURN_BEGIN | 21 | 1 | 8 | 1 | 31 |
| ZHUNBEI_BEGIN | 4 | 6 | 19 | 1 | 30 |
| PHASE_SPECIFIC_OTHER | 12 | 0 | 8 | 1 | 21 |
| TURN_AFTER | 2 | 0 | 7 | 1 | 10 |
| DRAW_BEGIN | 5 | 4 | 0 | 0 | 9 |
| JIESHU_BEGIN | 2 | 3 | 1 | 1 | 7 |
| PLAY_END | 2 | 0 | 3 | 0 | 5 |
| DISCARD_END | 2 | 0 | 2 | 0 | 4 |
| PLAY_BEGIN | 4 | 0 | 0 | 0 | 4 |
| TURN_BEGIN+TURN_END | 1 | 0 | 0 | 0 | 1 |
| JUDGE_BEGIN | 0 | 0 | 1 | 0 | 1 |
| DRAW_END/PLAY_END/DISCARD_END | 0 | 0 | 0 | 1 | 1 |
| TURN_END+TURN_AFTER | 0 | 0 | 1 | 0 | 1 |
| ZHUNBEI_END/JIESHU_END/TURN_END（结构性） | 0 | 0 | 1 | 0 | 1 |
| DISCARD_BEGIN | 1 | 0 | 0 | 0 | 1 |
| DRAW_BEGIN+TURN_END | 1 | 0 | 0 | 0 | 1 |
| PLAY_END+TURN_END | 1 | 0 | 0 | 0 | 1 |
| JUDGE_BEGIN/DRAW_BEGIN/PLAY_BEGIN/DISCARD_BEGIN | 0 | 0 | 0 | 1 | 1 |
| TURN_BEGIN+TURN_AFTER | 1 | 0 | 0 | 0 | 1 |

按单一分类拆面汇总（组合条目每面各计一次；族/结构性条目单列）：

| 分类 | 计数 |
|---|---:|
| TURN_END | 41 |
| TURN_BEGIN | 33 |
| ZHUNBEI_BEGIN | 30 |
| PHASE_SPECIFIC_OTHER | 21 |
| TURN_AFTER | 12 |
| DRAW_BEGIN | 10 |
| JIESHU_BEGIN | 7 |
| PLAY_END | 5 |
| PLAY_BEGIN | 4 |
| DISCARD_END | 4 |
| （结构性/族条目） | 3 |
| DISCARD_BEGIN | 1 |
| JUDGE_BEGIN | 1 |

### 3.3 改动类型计数

| 改动类型 | 条目 | P0 | P1 | P2 |
|---|---:|---:|---:|---:|
| 改到 TurnBegin 之前（回合交接时、发 TurnBegin 前） | 4 | 2 | 2 | 0 |
| 改挂 TurnBegin 窗 | 29 | 7 | 21 | 1 |
| 改挂 Phase{Prepare,begin} | 29 | 0 | 29 | 0 |
| 改挂整回合结束窗·phaseEnd 段（需新建） | 27 | 20 | 5 | 2 |
| 改挂整回合结束窗·phaseAfter 段（需新建） | 8 | 4 | 0 | 4 |
| 两面分别改挂 TurnBegin 窗＋phaseAfter 段 | 1 | 1 | 0 | 0 |
| 两面分别改挂 TurnBegin 窗＋phaseEnd 段 | 1 | 0 | 1 | 0 |
| 改挂 Phase{Play,end}（精确锚） | 1 | 0 | 0 | 1 |
| 内核生命周期／口径（前置依赖） | 4 | 1 | 2 | 1 |
| DSL 装载器重映射 | 3 | 0 | 3 | 0 |
| 待决策 | 1 | 0 | 1 | 0 |
| 保持 | 60 | 0 | 0 | 60 |

优先级：**P0**＝A1 已造成多触发或流程回归（任意阶段开始/结束类、翻面、回合重置、灵舞、巫蛊、热血等），**P1**＝锚点语义漂移（判定开始≠回合开始/准备阶段）或迁移的前置依赖，**P2**＝时点近似、口径登记、可选优化。

## 4. 需改清单（按改动类型分组；名字｜上游触发｜原生现状｜改成）

### 改到 TurnBegin 之前（回合交接时、发 TurnBegin 前）（4）

- K03 qianxing 解除（隐身 tempHide） [P1]｜player.tempHide() → addTempSkill("qianxing", { player: "phaseBeginStart" }) @杀海/character/ow/skill.js:718｜判定阶段开始解除｜TurnBegin 之前（回合交接时、发 TurnBegin 前解除；上游 qianxing 至 phaseBeginStart，先于 phaseBegin）
- K10 翻面跳过整回合（E03） [P0]｜phase 事件 step⑤ 翻面检测 isTurnedOver→turnOver+event.cancel()（content.ts:4750），先于 phaseBeginStart（4836）/phaseBegin（4841）；命中则整回合无 phaseBegin/准备阶段 @core/noname/library/element/content.ts:4750｜判定阶段开始时检测：A1 后翻面角色先跑完 TurnBegin 窗与准备阶段开始/结束，再在判定开始被翻回跳过｜TurnBegin 之前（commit_turn_ended 发 TurnBegin 前检测翻面并直达回合收束，不发 TurnBegin/准备阶段帧）
- K25 reset_turn_limits 时点 [P0]｜phase 事件 step②：game.phaseNumber++、各角色 actionHistory/stat 入栈（content.ts:4669-4736），先于 phaseBeginStart（4836）/phaseBegin（4841） @core/noname/library/element/content.ts:4669｜判定阶段开始重置每回合限一次｜TurnBegin 之前（commit_turn_ended 发 TurnBegin 前；连同 turn_opened 块中的 turn_damage/cards_used_turn/sheling 等回合统计清零）
- K30 形态轮换（boss_nbianshen） [P1]｜{ player: "phaseBefore" } @core/mode/boss.js:8210｜判定开始锚换将｜TurnBegin 之前（换将须先于 TurnBegin/准备阶段，使新形态的回合开始技生效）

### 改挂 TurnBegin 窗（29）

- H06 YxsMenshen3（门神（yxs_menshen3 清除面）） [P0]｜{ player: ["phaseBegin", "dieBegin"] } @ext/英雄杀/character/skill.js:37｜任意阶段开始（每回合 5→6 次）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） 且 player==owner（补本人门）；Death 面限持有者本人
- H12 Mailun（脉轮） [P0]｜{ player: "phaseBegin" } @杀海/character/swd/skill.js:7558｜任意（自己）阶段开始｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H17 YunchouDiyguhuo（蛊惑（diyguhuo）） [P1]｜{ player: "phaseBegin" } @杀海/character/yunchou/skill.js:372｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H19 GujianFoodYougeng（酉羹） [P1]｜{ player: "phaseBegin" }（intro 文案「准备阶段」） @杀海/card/gujian/skill.js:289｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H22 GujianFoodZiyangdan（紫阳丹） [P2]｜{ player: "phaseBegin" }（silent，每回合开始护甲-1） @杀海/card/gujian/skill.js:6｜自己结束阶段开始（上游=每回合开始；登记差异）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H24 HuanleXuelunyangCard（写轮眼） [P1]｜{ player: "phaseBegin" } @ext/欢乐卡牌/card/skill.js:72｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H25 SwdHyunshen2（云身（hyunshen2）） [P1]｜{ player: "phaseBegin" } @杀海/character/swd/skill.js:1375｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H26 SwdDaixing2（代形（daixing2）） [P1]｜{ player: "phaseBegin" } @杀海/character/swd/skill.js:5330｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H28 SwdJikong（亟空） [P0]｜{ player: ["loseEnd", "phaseBegin"] } usable:1 @杀海/character/swd/skill.js:5619｜任意（自己）阶段开始（每回合限一次）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H30 SwdJinlin2（金鳞（jinlin2）） [P0]｜{ player: "phaseBegin" } @杀海/character/swd/skill.js:1544｜任意（自己）阶段开始（范本式省略 phase）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H32 SwdQinglongzhiguiHeld（青龙之圭） [P1]｜g_qinglongzhigui: { player: "phaseBegin" } @杀海/card/swd/skill.js:514｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H33 SwdHuanglinzhicongHeld（黄麟之琮） [P1]｜g_huanglinzhicong: { player: "phaseBegin" } @杀海/card/swd/skill.js:584｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H35 SwdHujing（壶境） [P1]｜{ player: "phaseBegin" } @杀海/character/swd/skill.js:721｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H39 SwdZhenjiu2（针灸（zhenjiu2）） [P1]｜{ player: "phaseBegin" } @杀海/character/swd/skill.js:7943｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H40 SwdHuanxing（幻形） [P1]｜{ player: "phaseBegin" } @杀海/character/swd/skill.js:3040｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H41 SwdHuajin2（化金（huajin2/huajin3）） [P1]｜huajin3: { player: "phaseBegin" }（自清面）；huajin2: damageBegin @杀海/character/swd/skill.js:2201｜判定阶段开始自清｜extra_events 中 PhaseBegin→TurnBegin（自清面）；伤害前置面保持
- H42 SwdRexue（热血） [P0]｜{ global: "phaseBegin" } @杀海/character/swd/skill.js:4413｜任意（他人）阶段开始 → 每回合 5→6 次机会｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（global：player≠owner）
- H43 SwdFengyinState（机关蜂·封印） [P0]｜addTempSkill("fengyin", { player: "phaseBegin" }) @杀海/card/swd/card.js:952｜任意（自己）阶段开始解除｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H44 SwdGouhunluoSub（勾魂锣·倒计时） [P0]｜{ global: "phaseBegin" } @杀海/card/swd/skill.js:458｜来源的任意阶段开始 -1｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（global：仅来源 player 的回合）
- H45 WudianGeju（割据） [P1]｜{ player: "phaseBegin" } @ext/玩点论杀/character/skill.js:303｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- H49 XianjianQianfang（千方） [P1]｜{ player: "phaseBegin" } @杀海/character/xianjian/skill.js:3473｜自己判定阶段开始（钉死 Judge=G14 族）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- S14 BOSS_SHEDU（蛇毒） [P1]｜{ player: "phaseBegin" } @core/mode/boss.js:4487｜自己判定阶段开始（钉 Judge）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- K02 qianggu2 解除（强固） [P1]｜addTempSkill("qianggu2", { player: "phaseBegin" }) @杀海/character/ow/skill.js:142｜判定阶段开始解除｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） 的 Commit 前/BeforeTriggers（解除类无需触发窗，可在 TurnBegin 提交前内联）
- K04 hslingjian_chaofeng 解除（嘲讽） [P1]｜addTempSkill("hslingjian_chaofeng", { player: "phaseBegin" }) @杀海/card/swd/card.js:1506｜判定阶段开始解除｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（同上，解除内联）
- K09 pingzhang_count 转计数（屏障） [P1]｜pingzhang_count: { player: "phaseBegin" } silent @杀海/character/ow/skill.js:489｜判定阶段开始转计数｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（同上，内联转计数）
- K23 mtg_feixu_skill（废墟地图） [P1]｜mtg_feixu_skill: { player: "phaseBegin" } silent @杀海/card/mtg/skill.js:119｜判定阶段开始置牌｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（上游 player phaseBegin silent；现每次判定阶段开始执行，含额外判定阶段）
- K46 封冻（boss_fengdong） [P1]｜{ player: "phaseBegin" } @core/mode/boss.js:4788｜判定开始锚授封印｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）
- K47 巡游（boss_xunyou） [P1]｜{ global: "phaseBegin" } filter player!==event.player @core/mode/boss.js:4799｜他人判定开始锚｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（global）
- K48 炎狱（boss_yanyu） [P1]｜{ global: "phaseBegin" } filter player!==event.player @core/mode/boss.js:4765｜他人判定开始锚｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（global）

### 改挂 Phase{Prepare,begin}（29）

- H02 HlssFengqi（起风） [P1]｜{player:['phaseZhunbeiBegin']}（注：core/mode/boss.js:7586 的同名 fengqi 为 ['phaseZhunbeiBegin','phaseJieshuBegin'] 两值，但不是本 host 键的来源） @liuli/character/extra.js:458｜自己判定阶段开始（替位准备阶段）｜Phase{Prepare,begin}
- H09 Xiuluo（修罗） [P1]｜{ player: "phaseZhunbeiBegin" } @杀海/character/swd/skill.js:7073｜自己判定阶段开始（替位准备阶段）｜Phase{Prepare,begin}
- H14 BossQingyi（青翼） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:6989｜自己判定阶段开始（替位准备阶段）｜Phase{Prepare,begin}
- H15 BossChiyi（赤翼） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:6797｜自己判定阶段开始（替位准备阶段）｜Phase{Prepare,begin}
- S03 GUAN_XING（观星） [P1]｜{ player: "phaseZhunbeiBegin" } @core/character/standard/skill.js:1329｜自己判定阶段开始（替位准备阶段）｜Phase{Prepare,begin}
- S05 LUO_SHEN（洛神） [P1]｜{ player: "phaseZhunbeiBegin" } @core/character/standard/skill.js:839｜自己判定阶段开始（替位准备阶段）｜Phase{Prepare,begin}
- S09 SHEN_ZHI（神智） [P1]｜{ player: "phaseZhunbeiBegin" } @core/character/sp/skill.js:44739｜自己判定阶段开始（登记差异：无准备阶段以判定开始锚）｜Phase{Prepare,begin}
- S11 WANG_ZUN（妄尊） [P1]｜{ global: "phaseZhunbeiBegin" } @core/character/sp/skill.js:43352｜他人判定阶段开始（替位准备阶段）｜Phase{Prepare,begin}（global：player≠owner）
- S12 BOSS_FUTAI（覆泰） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:5417｜自己判定阶段开始（注释自称「神智同口径」）｜Phase{Prepare,begin}
- S13 BOSS_LUANCHANG（乱常·南蛮面） [P1]｜subSkill: { player: "phaseZhunbeiBegin" } @core/mode/boss.js:5445｜自己判定阶段开始（万箭面在 kernel Phase::End 臂=end.rs:436）｜Phase{Prepare,begin}（南蛮面）；万箭面 phaseJieshuBegin 已由 kernel/phase/end.rs:436-462 正确承载
- K26 红莲（Boss 毕方 boss_honglianx） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:4705｜回合开始（判定开始锚）获红牌+火伤｜Phase{Prepare,begin}
- K27 绝红（boss_juehong） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:4898｜判定开始锚｜Phase{Prepare,begin}
- K28 夹击（boss_jiazu） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:5095｜判定开始锚｜Phase{Prepare,begin}
- K29 造异异变（boss_zaoyi_hp） [P1]｜subSkill hp: { player: "phaseZhunbeiBegin" } @core/mode/boss.js:4939｜判定开始锚｜Phase{Prepare,begin}
- K31 自润（boss_zirun） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:4882｜判定开始锚｜Phase{Prepare,begin}
- K32 魔道（boss_modao） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:8504｜判定开始锚｜Phase{Prepare,begin}
- K33 轮回（boss_lunhui） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:3957｜判定开始锚｜Phase{Prepare,begin}
- K34 猛态·始（boss_mengtai_begin） [P1]｜subSkill begin: { player: "phaseZhunbeiBegin" } @core/mode/boss.js:8157｜判定开始锚置双旗标｜Phase{Prepare,begin}
- K35 睿智（boss_ruizhi） [P1]｜{ global: "phaseZhunbeiBegin" } filter event.player!==player && (h>1||e>1)；content 只作用 trigger.player @core/mode/boss.js:8065｜有持有者即在每名角色判定开始对**全体**角色执行（目标过宽，既有差异）｜Phase{Prepare,begin}（global，仅当前回合角色且≠持有者）
- K37 公正（boss_gongzheng） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:3679｜判定开始锚｜Phase{Prepare,begin}
- K38 吸行（boss_xixing） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:8544｜判定开始锚｜Phase{Prepare,begin}
- K39 雷舟（boss_leizhou） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:3744｜判定开始锚｜Phase{Prepare,begin}
- K40 苦酒（boss_kujiu） [P1]｜{ global: "phaseZhunbeiBegin" } @core/mode/boss.js:3805｜他人判定开始锚｜Phase{Prepare,begin}（global，持有者≠当前回合角色）
- K41 不佛（boss_bufo，准备面） [P1]｜{ player: ["damageBegin4","phaseZhunbeiBegin"] } @core/mode/boss.js:4195｜判定开始锚｜Phase{Prepare,begin}
- K42 无量（boss_wuliang，准备面） [P1]｜{ global:"gameDrawAfter", player:["phaseZhunbeiBegin","phaseJieshuBegin","enterGame"] } @core/mode/boss.js:4222｜判定开始锚回复至 3｜Phase{Prepare,begin}
- K43 炙焚（boss_zhifen） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:3875｜判定开始锚｜Phase{Prepare,begin}
- K44 锁足（boss_suozu） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:3911｜判定开始锚｜Phase{Prepare,begin}
- K45 往事（boss_wangshi） [P1]｜{ global: "phaseZhunbeiBegin" } filter 敌方 @core/mode/boss.js:4050｜他人判定开始锚｜Phase{Prepare,begin}（global）
- K49 中毒（boss_zhongdu） [P1]｜{ player: "phaseZhunbeiBegin" } @core/mode/boss.js:6957｜判定开始锚｜Phase{Prepare,begin}

### 改挂整回合结束窗·phaseEnd 段（需新建）（27）

- H50 PhaseEndRecoverOrDraw（九尾（jiuwei）） [P0]｜{ player: "phaseEnd" } @ext/欢乐卡牌/card/skill.js:131｜自己任意阶段结束（每回合 5 次）｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H51 YxsMenshen（门神） [P0]｜{ player: "phaseEnd" } @ext/英雄杀/character/skill.js:55｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H53 Xuanyuan（轩辕） [P0]｜{ player: "phaseEnd" } @杀海/character/swd/skill.js:3255｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H54 Hutian（护天） [P0]｜{ player: "phaseEnd" }（寄存主面） @杀海/character/swd/skill.js:2915｜自己任意阶段结束（寄存/取回由 phase.rs:221 阶段收束内联，跨一个阶段边界）｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（寄存主面）；取回面见内核 hutian 条（phase.rs:217-223）
- H57 GujianFoodLiyutang（鲤鱼汤） [P0]｜{ player: "phaseEnd" }（intro：结束阶段获得护甲） @杀海/card/gujian/skill.js:263｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H62 SwdDatong（大同） [P0]｜{ global: "phaseEnd" } @杀海/character/swd/skill.js:399｜任意玩家任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global）
- H63 SwdHuanling（幻灵） [P0]｜{ player: "phaseEnd" } @杀海/character/swd/skill.js:7813｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H64 SwdDonghuangzhongCard（东皇钟） [P0]｜{ player: "phaseEnd" } @杀海/card/swd/skill.js:952｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H66 SwdMufengHeld（沐风） [P0]｜{ player: "phaseEnd" } @杀海/character/swd/skill.js:3876｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H67 SwdLingxin（灵心） [P0]｜{ player: "phaseEnd" } @杀海/character/swd/skill.js:5411｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H69 SwdLingfeng（凌锋） [P2]｜{ player: "phaseEnd" } @杀海/character/swd/skill.js:1150｜自己出牌阶段结束（钉死 Play=错位族）｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H70 SwdLqingcheng（倾城） [P0]｜{ player: "phaseEnd" } @杀海/character/swd/skill.js:5362｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H71 SwdJiguanyuanGain（机关鸢·待获得） [P0]｜jiguanyuan: { player: "phaseEnd" } @杀海/card/swd/skill.js:503｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H73 WudianZhucheng（筑城） [P0]｜{ player: "phaseEnd" } @ext/玩点论杀/character/skill.js:161｜自己任意阶段结束（kernel phase.rs:128-151 在每阶段收束直推 OptionalGate）｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H74 WudianGuangshuSpade（光塔（guangshu_spade）） [P1]｜guangshu_spade: { player: "phaseEnd" }（intro：下个结束阶段受雷伤） @杀海/character/ow/skill.js:3115｜自己任意阶段结束（kernel phase.rs:183-201 内联在 PhaseEnd 收束）｜整回合结束窗（与宿主面同步；若按 intro 语义则 Phase{End,end}——需在规则基准登记取舍）
- H75 WudianZiyu（自愈） [P1]｜{ global: "phaseEnd" } + game.phaseNumber%4===0 @杀海/character/ow/skill.js:3123｜任意阶段结束且 phase_number%4==0（phase_number 每非准备阶段+1）｜整回合结束窗；同时登记 phaseNumber 口径差异：上游 game.phaseNumber 每回合 +1（content.ts:4670），原生 phase_number 每回合 +5（phase.rs:109-114 豁免准备阶段后仍 +5）→ %4 命中节奏不同
- H76 WudianPaotaiFire（炮台（paotai2）） [P0]｜paotai2: { player: "phaseEnd" }（概率门） @杀海/character/ow/skill.js:896｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H78 WudianJidong（急冻） [P0]｜{ global: "phaseEnd" } @杀海/character/ow/skill.js:1858｜任意玩家任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global）
- H80 XianjianBinxin（冰心） [P0]｜{ global: "phaseEnd" } @杀海/character/xianjian/skill.js:1520｜任意玩家任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global）
- H81 XianjianLingyu（灵愈） [P0]｜{ player: "phaseEnd" } @杀海/character/xianjian/skill.js:2652｜自己任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- H83 XianjianLinghuo（灵火） [P1]｜{ global: "phaseEnd" } filter event.player!==player @杀海/character/xianjian/skill.js:576｜他人任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global）
- H85 XianjianWangyou（忘忧） [P0]｜{ global: "phaseEnd" } @杀海/character/xianjian/skill.js:3091｜他人任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global）
- H87 XianjianShenwu（神舞） [P1]｜{ global: "phaseEnd" }（觉醒技） @杀海/character/xianjian/skill.js:27｜任意玩家任意阶段结束｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global）
- K05 hslingjian_xingtigaizao 重置（星体改造） [P2]｜hslingjian_xingtigaizao: { player: "phaseEnd" }（移除+清零在「其下一结束阶段」） @杀海/card/swd/skill.js:2204｜本回合开始重置（登记差异：重置时点=下一回合开始而非上一结束）｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）
- K13 guangshu_spade 雷伤（光塔） [P1]｜{ player: "phaseEnd" }（intro=下个结束阶段） @杀海/character/ow/skill.js:3115｜任意阶段收束（宿主元数据同款问题）｜整回合结束窗（与 WudianGuangshuSpade 宿主条同步取舍；若按 intro 则 Phase{End,end}）
- K14 zhucheng 确认门（筑城） [P0]｜{ player: "phaseEnd" } optional @ext/玩点论杀/character/skill.js:161｜每阶段收束确认｜整回合结束窗（与 WudianZhucheng 条联动）
- K17 hutian3 寄存取回（护天） [P0]｜hutian3: { player: ["phaseEnd","dieBegin"] } @杀海/character/swd/skill.js:3004｜跨任一阶段边界归还（登记：上游跨回合边界）｜整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（取回须加「非寄存当回合」守卫=上游 hutian4 临时技）

### 改挂整回合结束窗·phaseAfter 段（需新建）（8）

- H60 GwentFengyinState（阻魔手铐·封印） [P0]｜addTempSkill("fengyin", { player: "phaseAfter" }) @杀海/card/gwent/card.js:1625｜自己任意阶段结束解除封印｜整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）
- H72 WudianFuchou2（负仇（fuchou2）） [P0]｜{ global: "phaseAfter" } @ext/玩点论杀/character/skill.js:553｜任意玩家任意阶段结束｜整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）（global）
- K01 anwugu2 递减（巫蛊；原生注释称「蛊舞」） [P0]｜anwugu2: { player: "phaseAfter" } silent（intro「剩余$回合」=按回合递减） @杀海/character/xianjian/skill.js:501｜每阶段结束递减（每回合 5→6 次）｜整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）（递减一次/回合）
- K18 lingwu 额外阶段（灵舞） [P0]｜lingwu: { player: "phaseAfter" } frequent @杀海/character/swd/skill.js:5440｜每阶段收束判定（上游=回合结束后一次）｜整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）；效果亦应为 insertPhase=额外回合（原生为额外出牌阶段，既有差异）
- K50 封冻解除（end.rs） [P2]｜current.addTempSkill("fengyin")（无 expire→默认 {global:["phaseAfter","phaseBeforeStart"]}，player.js:11966） @core/mode/boss.js:4793｜结束阶段开始全清｜整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）
- K51 反噬 terra 清零（end.rs） [P2]｜addTempSkill("boss_zlfanshi_terra")（默认回合结束后过期） @core/mode/boss.js:4013｜结束阶段开始全清（另 judge.rs:5-7 与 events.rs:706-708 也各清一次）｜保留 events.rs:706-708（回合收束≈phaseAfter），删去结束阶段开始与判定开始两处
- K52 往事封禁清零（end.rs） [P2]｜trigger.player.addTempSkill("boss_wangshi2")（默认回合结束后过期） @core/mode/boss.js:4062｜结束阶段开始全清｜整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）
- K53 巨源记体力（end.rs） [P2]｜{ player: "phaseAfter" } @core/mode/boss.js:4594｜结束阶段开始记 hp｜整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）

### 两面分别改挂 TurnBegin 窗＋phaseAfter 段（1）

- H11 Kunlunjing（幻镜（kunlunjing）） [P0]｜kunlunjing2: { player: "phaseAfter" }（快照面，swd/skill.js:4635）；kunlunjing1: { player: "phaseBegin" }（比对面，swd/skill.js:4536） @杀海/character/swd/skill.js:4635｜任意阶段结束记快照 + 任意阶段开始比对（step 按帧 begin 位分流 skills_status.rs:324-328）｜比对面→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）；快照面→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）

### 两面分别改挂 TurnBegin 窗＋phaseEnd 段（1）

- H38 SwdLexue（乐学） [P1]｜lexue1: { player: "phaseBegin" } @杀海/character/swd/skill.js:5954｜任意（自己）判定阶段开始（钉死 Judge）｜TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（lexue1 面，限本人回合）；lexue2 面（上游 phaseEnd）→ 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）

### 改挂 Phase{Play,end}（精确锚）（1）

- K56 BOSS_XUSHI 虚失·自翻面 [P2]｜{ player: ["phaseUseEnd","turnOverEnd"] } @liuli/mode/boss.js:4068（本条只覆盖 phaseUseEnd 面；turnOverEnd 面在 commit_turn_over_before 尾内联，不涉阶段锚）｜弃牌阶段开始自翻面（begin_discard 首臂 @kern/phase/discard.rs:15-26；注释所据『原生无 PhaseEnd 帧』在 R05 后已不成立）｜改挂 Phase{Play,begin:false}。迁移需复核两条路径：跳过出牌阶段（乐不思蜀/贪欲类）时上游 phaseUse 以 Skipped 收场、phaseUseEnd 是否仍触发需对齐；现锚在 begin_discard 内，跳过弃牌阶段（入梦/贪欲/庆祝）时虚失不翻面——改挂后该行为变化需登记

### 内核生命周期／口径（前置依赖）（4）

- K21 回合收束 commit_turn_ended [P0]｜event.trigger("phaseEnd") → step11 phaseAfter @core/noname/library/element/content.ts:4913｜TurnEnded 帧有 AfterTriggers 窗，但起步于 Commit：commit_turn_ended 先换人并发 TurnBegin，TurnBegin 压在其上，故 TurnEnded 的触发要等下一位角色 TurnBegin→准备开始/结束→判定开始（含判定区结算）整棵子树结算完才派发，且 turn.current 已切换；其 Commit 内联清理（expire_temporary_skills 等）时点≈phaseAfter｜新建回合结束窗：在交接前派发 phaseEnd/phaseAfter 两段（例：TurnEnded 改 BeforeTriggers 起步、触发在 Commit 前；或 TurnEnded 提交只做清理，TurnBegin 推迟到其 after 段完成后再发）
- K22 commit_turn_begin [P1]｜event.trigger("phaseBegin")（回合开始后⑨，阶段列表前） @core/noname/library/element/content.ts:4841｜TurnBegin BeforeTriggers→Commit→静默 after；提交步自动接准备阶段→判定｜①发 TurnBegin 前把 turn.phase 置 Prepare（或独立的回合开始态），避免 TurnBegin 窗读到 Judge；②rvm_bridge::trigger_ctx_of 补 TurnBegin（及 TurnEnded）臂给 event_player；③首回合（建局直接 Judge+turn_opened=true）补发 TurnBegin/准备阶段，或在规则基准登记首回合差异
- K24 phase_number 计数 [P2]｜game.phaseNumber++（每回合 phase 事件内 +1，另有 player.phaseNumber） @core/noname/library/element/content.ts:4670｜每回合 +5（准备阶段豁免）｜与上游 game.phaseNumber（每回合+1）对齐或明确登记差异；WudianZiyu %4 门依赖此数
- K55 L2 PhaseEnd 帧触发窗时序（PhaseEnd 帧触发窗时序） [P1]｜上游 phaseXEnd 在本阶段事件内结算完毕后才进入下一阶段；phaseEnd/phaseAfter 在全部阶段之后、交接之前 @core/noname/library/element/content.ts:4913｜PhaseEnd 触发晚于下一阶段开始子树：Prepare,end 的触发在判定开始（含判定区结算）之后；End,end 的触发在下一位角色 TurnBegin→准备→判定开始之后｜A2 若要把任何技能落到 Phase{Prepare,end} 或整回合结束窗，须先让 PhaseEnd 帧「先触发后提交」（或另开前置窗），否则 ZHUNBEI_END 技会晚于判定阶段

### DSL 装载器重映射（3）

- D01 phase_begin [P1]｜旧线 DomainEvent::TurnBegan→EventKind::PhaseBegin（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:841）；编译器 hover「fires once at the start of a turn, immediately before phase_prepare」（crates/content-compiler/src/lsp/hover_docs/skill.rs:226-228）＝上游 phaseBegin｜映射为「判定阶段开始」｜EventKind::TurnBegin（去掉 AtPhase(Judge) 门）+ 补 trigger_ctx_of TurnBegin 臂（event_player）；首回合缺帧仍需登记或补发
- D02 phase_prepare [P1]｜旧线 TurnStateChanged{Prepare}→EventKind::PhasePrepare（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:867）＝上游 phaseZhunbeiBegin｜拒装（73 个包 732 处不可用）｜Phase{Prepare,begin}（EventKind::PhaseBegin+AtPhase(Prepare)）——A1 已有真实准备阶段帧，可解除拒装
- D06 turn_ended [P1]｜旧线 PhaseEnded{End}→EventKind::TurnEnded（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:917）；LSP 补全「After a player's whole turn ends」（crates/content-compiler/src/lsp/completions/trigger_event.rs:114-117）＝上游 phaseEnd/phaseAfter 一族｜映射 EventKind::TurnEnded：触发窗在下一位角色 TurnBegin→准备→判定开始子树之后才派发；且 trigger_ctx_of 无 TurnEnded 臂（rvm_bridge.rs:194 default）→ event_player=None，依赖持有者=事件玩家的 VM 过滤恒假｜映射到新建回合结束窗（phaseAfter 段；编译器补全文案为「After a player's whole turn ends」）并补 trigger_ctx_of 的 TurnEnded/TurnBegin 臂

### 待决策（1）

- D07 phase_any_end/phase_draw_skipped 等旧线事件（phase_any_end 等） [P1]｜旧线 "phase_any_end"→EventKind::PhaseAnyEnd（crates/engine-core/src/content/registry/default_view_as_card_count.rs:620）；hover「fires after any concrete Prepare, Judge, Draw, Play, Discard, or End phase」｜拒装｜可选：AtPhase 无门 PhaseEnd（任意阶段结束，含准备阶段）或单独 PhaseAnyEnd kind；22 个包 300 处 phase_any_end 目前仅旧线可用

## 5. 全量对照表

### 5.1 宿主技元数据（metadata PhaseBegin/PhaseEnd）

| ID | 原生名（中文） | 原生 event＋阶段约束（file:line） | 上游 file:line｜trigger | 分类 | 原生现状 | 需改→改成 | 准备阶段影响/风险 |
|---|---|---|---|---|---|---|---|
| H01 | YunchouXiongzi（雄姿） | PhaseBegin @meta/common.rs:39；自己摸牌阶段开始（Draw,begin,player==owner） @filt/common.rs:516 | 杀海/character/yunchou/skill.js:144｜{ player: "phaseDrawBegin" } | DRAW_BEGIN | 自己摸牌阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H02 | HlssFengqi（起风） | PhaseBegin @meta/common.rs:106；自己判定阶段开始（Judge,begin） @filt/common.rs:43 | liuli/character/extra.js:458｜{player:['phaseZhunbeiBegin']}（注：core/mode/boss.js:7586 的同名 fengqi 为 ['phaseZhunbeiBegin','phaseJieshuBegin'] 两值，但不是本 host 键的来源） | ZHUNBEI_BEGIN | 自己判定阶段开始（替位准备阶段） | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| H03 | MbQianlong（潜龙） | PhaseBegin(+CardsDrawn/CardsMoved/Damage) @meta/common.rs:118；Phase begin 任意阶段（phase_number==1 门）+ 获得牌/伤害 @filt/common.rs:51-73 | core/character/mobile/skill.js:13793｜group mbqianlong_begin: { global: "phaseBefore", player:"enterGame" } + filter phaseNumber==0 | PHASE_SPECIFIC_OTHER | 开局首个阶段帧（phase_number==1）等价 gameStart/回合开始前 | 否→保持 [P2] | phase_number 只在非 Prepare 阶段 +1（phase.rs:112 已豁免准备阶段），首个非准备阶段帧仍为 phase_number==1 → 行为不变；若未来把准备阶段计入计数则此面会漂移 |
| H04 | MbcmQingzheng（清正） | PhaseBegin @meta/common.rs:140；自己出牌阶段开始（Play,begin） @filt/common.rs:414 | core/character/mobile/skill.js:14068｜{ player: "phaseUseBegin" } | PLAY_BEGIN | 自己出牌阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H05 | XiguaXueliangFanbei（血量翻倍） | PhaseBegin @meta/common.rs:173；任意阶段开始（player==owner，标记门一次性） @filt/common.rs:166-186 | liuli/extension/标记补充/extension.js:2263｜_xigua_xueliangfanbei_: { global: "gameStart" } | PHASE_SPECIFIC_OTHER | 首次阶段开始近似 gameStart（登记差异） | 否→保持（可选：改挂 TurnBegin 窗=更贴近 gameStart） [P2] | 无变化：过滤匹配首个阶段帧且标记一次性——首回合不发准备阶段帧（建局 turn_opened=true），首触发仍是首轮摸牌阶段开始 |
| H06 | YxsMenshen3（门神（yxs_menshen3 清除面）） | PhaseBegin(+Death) @meta/common.rs:191；任意阶段开始（**无 player 门**——上游为 player 自己回合开始）+任意死亡 @filt/common.rs:130-142 | ext/英雄杀/character/skill.js:37｜{ player: ["phaseBegin", "dieBegin"] } | TURN_BEGIN | 任意阶段开始（每回合 5→6 次） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） 且 player==owner（补本人门）；Death 面限持有者本人 [P0] | 过滤为任意角色任意阶段开始（无本人门）：A1 后每个角色回合多一次清除点（准备阶段开始）；上游只在门神持有者本人回合开始/死亡时清除 |
| H07 | Zaibian（灾变） | PhaseBegin @meta/common.rs:213；自己出牌阶段开始（Play,begin） @filt/common.rs:189 | liuli/extension/末日乱斗/extension.js:368｜{player:'phaseUseBegin'} | PLAY_BEGIN | 自己出牌阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H08 | NewMeibu（魅步） | PhaseBegin @meta/common.rs:244；任意角色出牌阶段开始（Play,begin,player!=owner） @filt/common.rs:88-101 | core/character/sp/skill.js:35049｜{ global: "phaseUseBegin" } | PLAY_BEGIN | 任意角色出牌阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H09 | Xiuluo（修罗） | PhaseBegin @meta/common.rs:296；自己判定阶段开始（Judge,begin）+判定区/手牌非空 @filt/common.rs:222-227 | 杀海/character/swd/skill.js:7073｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 自己判定阶段开始（替位准备阶段） | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| H10 | Miedao（灭道） | PhaseBegin @meta/common.rs:305；自己弃牌阶段开始（Discard,begin）+已受伤 @filt/common.rs:230-234 | 杀海/character/swd/skill.js:4065｜miedao2: { player: "phaseDiscardBegin" } | DISCARD_BEGIN | 自己弃牌阶段开始（miedao1 摸牌面在 D 机前窗内联） | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H11 | Kunlunjing（幻镜（kunlunjing）） | PhaseBegin(+PhaseEnd) @meta/common.rs:327；PhaseEnd 任意阶段收束记快照（begin:false）/ PhaseBegin 任意阶段开始比快照 @filt/common.rs:249-264；联动：host/step/skills_status.rs:324-328（step 以 Phase{begin} 区分两面） | 杀海/character/swd/skill.js:4635｜kunlunjing2: { player: "phaseAfter" }（快照面，swd/skill.js:4635）；kunlunjing1: { player: "phaseBegin" }（比对面，swd/skill.js:4536） | TURN_BEGIN+TURN_AFTER | 任意阶段结束记快照 + 任意阶段开始比对（step 按帧 begin 位分流 skills_status.rs:324-328） | 是→比对面→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）；快照面→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚） [P0] | 两面都是任意阶段过滤：A1 后每回合快照 6 次、比对 6 次（上游各 1 次） |
| H12 | Mailun（脉轮） | PhaseBegin @meta/common.rs:412；任意阶段开始（player==owner） @filt/common.rs:369-371 | 杀海/character/swd/skill.js:7558｜{ player: "phaseBegin" } | TURN_BEGIN | 任意（自己）阶段开始 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P0] | 任意阶段开始过滤：A1 后每回合 6 次选择（上游 1 次） |
| H13 | MbWeitong（卫统） | PhaseBegin @meta/common.rs:480；任意阶段开始（phase_number==1 门） @filt/common.rs:418-442 | core/character/mobile/skill.js:13895｜{ player: "mbqianlong_beginBegin" }（mbqianlong_begin 为 global phaseBefore+enterGame，phaseNumber==0） | PHASE_SPECIFIC_OTHER | 开局首阶段帧（phase_number==1） | 否→保持 [P2] | phase_number 豁免准备阶段 → 首个非准备阶段帧仍 ==1，行为不变 |
| H14 | BossQingyi（青翼） | PhaseBegin @meta/common.rs:585；自己判定阶段开始（Judge,begin,轮次门 3/5/7） @filt/common.rs:853+boss.rs:23-52 | core/mode/boss.js:6989｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 自己判定阶段开始（替位准备阶段） | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| H15 | BossChiyi（赤翼） | PhaseBegin @meta/common.rs:585；自己判定阶段开始（Judge,begin,轮次门 3/5/7） @filt/common.rs:854+boss.rs:23-52 | core/mode/boss.js:6797｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 自己判定阶段开始（替位准备阶段） | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| H16 | BossBaiyi（白翼（弃牌面）） | PhaseBegin @meta/common.rs:603；自己判定阶段开始（Judge,begin,round==5 门） @filt/common.rs:858-865 | core/mode/boss.js:5030｜boss_baiyi_discard: { global: "roundStart" } filter roundNumber===5 | PHASE_SPECIFIC_OTHER | 第 5 轮判定阶段开始近似 roundStart | 否→保持（可选：TurnBegin 窗 + 本轮首个回合门，更贴近 roundStart） [P2] | 不多触发；A1 后落点在 Boss 本人 TurnBegin/准备阶段之后（上游 roundStart 先于该轮首个回合的 phaseBegin，Boss 非轮首座位时原生本就偏晚） |
| H17 | YunchouDiyguhuo（蛊惑（diyguhuo）） | PhaseBegin @meta/common.rs:648；自己判定阶段开始（Judge,begin） @filt/common.rs:477-482；联动：host/factory.rs:222（按 Phase 帧取目标，需加 TurnBegin 臂） | 杀海/character/yunchou/skill.js:372｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H18 | D3Moyan（末焱） | PhaseBegin(+DamageBefore) @meta/common.rs:693；判定阶段开始（Judge,begin,一轮一次门） @filt/common.rs 无独立臂（support.rs:764 moyan_filter） | ext/3D精选/character/skill.js:474｜{ global: "roundStart" } | PHASE_SPECIFIC_OTHER | 每轮首个判定阶段开始近似 roundStart | 否→保持（可选：TurnBegin 窗 + 本轮首次门，更贴近 roundStart） [P2] | 不多触发；A1 后落点在本轮首个回合的 TurnBegin/准备阶段之后（上游 roundStart 先于 phaseBegin） |
| H19 | GujianFoodYougeng（酉羹） | PhaseBegin @meta/common.rs:748；自己判定阶段开始（Judge,begin） @filt/common.rs:626-628 | 杀海/card/gujian/skill.js:289｜{ player: "phaseBegin" }（intro 文案「准备阶段」） | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H20 | GujianFoodMapodoufu（麻婆豆腐） | PhaseBegin @meta/common.rs:774；自己结束阶段开始（End,begin） @filt/common.rs:614-616 | 杀海/card/gujian/skill.js:406｜{ player: "phaseJieshuBegin" } | JIESHU_BEGIN | 自己结束阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H21 | GujianFoodXiajiao（虾饺） | PhaseBegin @meta/common.rs:793；自己摸牌阶段开始（Draw,begin） @filt/common.rs:607-609 | 杀海/card/gujian/skill.js:63｜{ player: ["phaseUseBefore", "phaseEnd"] }（倒计时，每回合一次）+ 子技 xiajiao_draw { player: "phaseDrawBegin" }（gujian/skill.js:89） | DRAW_BEGIN+TURN_END | 自己摸牌阶段开始（倒计时在此扣减） | 否→保持（摸牌面相符；倒计时并入摸牌阶段开始=既有登记差异，跳过摸牌阶段时原生不扣倒计时） [P2] | 无（Draw 过滤不匹配准备阶段） |
| H22 | GujianFoodZiyangdan（紫阳丹） | PhaseBegin @meta/common.rs:807；自己结束阶段开始（End,begin） @filt/common.rs:629-631 | 杀海/card/gujian/skill.js:6｜{ player: "phaseBegin" }（silent，每回合开始护甲-1） | TURN_BEGIN | 自己结束阶段开始（上游=每回合开始；登记差异） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P2] | 现挂结束阶段开始（既有错位），不受准备阶段影响但与上游 phaseBegin 不符 |
| H23 | GwentCiguState（刺骨寒霜） | PhaseBegin @meta/common.rs:860；自己摸牌阶段开始（Draw,begin） @filt/common.rs:637-640 | 杀海/card/gwent/skill.js:226｜gw_ciguhanshuang: { player: "phaseDrawBegin" } | DRAW_BEGIN | 自己摸牌阶段开始（D 机前窗扣 1） | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H24 | HuanleXuelunyangCard（写轮眼） | PhaseBegin @meta/common.rs:898；自己判定阶段开始（Judge,begin,装备门） @filt/common.rs:658-675 | ext/欢乐卡牌/card/skill.js:72｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H25 | SwdHyunshen2（云身（hyunshen2）） | PhaseBegin @meta/swd.rs:17；自己判定阶段开始（Judge,begin,计数>0） @filt/swd.rs:28-30 | 杀海/character/swd/skill.js:1375｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H26 | SwdDaixing2（代形（daixing2）） | PhaseBegin @meta/swd.rs:35；自己判定阶段开始（Judge,begin,计数>0） @filt/swd.rs:38-40 | 杀海/character/swd/skill.js:5330｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H27 | SwdZhuhai（诛害） | PhaseBegin @meta/swd.rs:113；任意角色结束阶段开始（End,begin,player!=owner） @filt/swd.rs:78-97 | liuli/character/refresh.js:17056｜{global:'phaseJieshuBegin'} | JIESHU_BEGIN | 任意角色结束阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H28 | SwdJikong（亟空） | PhaseBegin(+CardsLost) @meta/swd.rs:124；任意阶段开始（player==owner）/ 失手牌后手牌空 @filt/swd.rs:100-123 | 杀海/character/swd/skill.js:5619｜{ player: ["loseEnd", "phaseBegin"] } usable:1 | TURN_BEGIN | 任意（自己）阶段开始（每回合限一次） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P0] | 任意阶段开始过滤（每回合限一次缓解）：A1 后首个可发动点变为准备阶段开始，且 once_used 要到判定开始 reset_turn_limits 才清零（准备阶段读到上回合残留） |
| H29 | SwdRumeng（入梦） | PhaseBegin @meta/swd.rs:160；任意角色出牌阶段开始（Play,begin,player!=owner） @filt/swd.rs:155-171 | 杀海/character/swd/skill.js:2278｜{ global: "phaseUseBefore" } | PHASE_SPECIFIC_OTHER | 任意角色出牌阶段开始（近似 Before=Begin，G15 既有差异） | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H30 | SwdJinlin2（金鳞（jinlin2）） | PhaseBegin @meta/swd.rs:181；任意阶段开始（player==owner） @filt/swd.rs:74-75 | 杀海/character/swd/skill.js:1544｜{ player: "phaseBegin" } | TURN_BEGIN | 任意（自己）阶段开始（范本式省略 phase） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P0] | 任意阶段开始过滤：A1 后每回合护甲-1/轮数-1 共 6 次（上游 1 次）；旧文档把本条当「G14 正确范本」系误读 phaseBegin |
| H31 | SwdXielv（谐率） | PhaseBegin @meta/swd.rs:343；自己结束阶段开始（End,begin,同色手牌门） @filt/swd.rs:345-368 | 杀海/character/swd/skill.js:6564｜{ player: "phaseDiscardEnd" } | DISCARD_END | 结束阶段开始帧近似 phaseDiscardEnd（相邻帧） | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H32 | SwdQinglongzhiguiHeld（青龙之圭） | PhaseBegin @meta/swd.rs:374；自己判定阶段开始（Judge,begin,手牌门） @filt/swd.rs:389-394 | 杀海/card/swd/skill.js:514｜g_qinglongzhigui: { player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H33 | SwdHuanglinzhicongHeld（黄麟之琮） | PhaseBegin @meta/swd.rs:382；自己判定阶段开始（Judge,begin,无护甲门） @filt/swd.rs:461-467 | 杀海/card/swd/skill.js:584｜g_huanglinzhicong: { player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H34 | SwdWangchenHeld（忘尘） | PhaseBegin @meta/swd.rs:429；自己结束阶段开始（End,begin,弃基本牌门） @filt/swd.rs:443-448 | 杀海/character/swd/skill.js:4194｜{ player: "phaseDiscardEnd" } | DISCARD_END | 结束阶段开始帧近似 phaseDiscardEnd（相邻帧） | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H35 | SwdHujing（壶境） | PhaseBegin @meta/swd.rs:487；自己判定阶段开始（Judge,begin,弃堆有炼妖壶） @filt/swd.rs:993-1000 | 杀海/character/swd/skill.js:721｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H36 | SwdShangxi（伤袭） | PhaseBegin @meta/swd.rs:735；自己出牌阶段开始（Play,begin） @filt/swd.rs:947-951 | 杀海/character/swd/skill.js:2505｜{ player: "phaseUseBegin" } | PLAY_BEGIN | 自己出牌阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H37 | SwdReyingzi（英姿（reyingzi）） | PhaseBegin @meta/swd.rs:808；自己摸牌阶段开始（Draw,begin） @filt/swd.rs:655-657 | core/character/refresh/skill.js:16088｜{ player: "phaseDrawBegin2" } | DRAW_BEGIN | 自己摸牌阶段开始（Begin2 档=摸牌前修正窗，原生以 D 机前窗/Draw begin 等价承载） | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H38 | SwdLexue（乐学） | PhaseBegin @meta/swd.rs:824；任意判定阶段开始（存活门；机器按当前回合分流） @filt/swd.rs:669-671 | 杀海/character/swd/skill.js:5954｜lexue1: { player: "phaseBegin" } | TURN_BEGIN+TURN_END | 任意（自己）判定阶段开始（钉死 Judge） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（lexue1 面，限本人回合）；lexue2 面（上游 phaseEnd）→ 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P1] | 过滤无 player 门（任意角色判定开始，机器内按当前回合分流）；A1 后换技晚于准备阶段 |
| H39 | SwdZhenjiu2（针灸（zhenjiu2）） | PhaseBegin @meta/swd.rs:835；自己判定阶段开始（Judge,begin,off_court 门） @filt/swd.rs:673-675 | 杀海/character/swd/skill.js:7943｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H40 | SwdHuanxing（幻形） | PhaseBegin @meta/swd.rs:881；自己判定阶段开始（Judge,begin） @filt/swd.rs:699-701 | 杀海/character/swd/skill.js:3040｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H41 | SwdHuajin2（化金（huajin2/huajin3）） | DamageBefore(+PhaseBegin) @meta/swd.rs:986；伤害前置 +1 / 判定阶段开始自清（Judge,begin） @filt/swd.rs:817-823；联动：filt/swd.rs:817-823、host/step/swd_damage_reactions.rs:195-204（step 内也按 Phase{Judge,begin} 判定自清） | 杀海/character/swd/skill.js:2201｜huajin3: { player: "phaseBegin" }（自清面）；huajin2: damageBegin | TURN_BEGIN | 判定阶段开始自清 | 是→extra_events 中 PhaseBegin→TurnBegin（自清面）；伤害前置面保持 [P1] | A1 后 +1 伤害效果在准备阶段仍有效（上游回合开始即移除） |
| H42 | SwdRexue（热血） | PhaseBegin @meta/swd.rs:1013；任意阶段开始（begin:true 无 phase 门，player!=owner） @filt/swd.rs:194-213；联动：host/factory.rs:234（按 Phase 帧取目标，需加 TurnBegin 臂） | 杀海/character/swd/skill.js:4413｜{ global: "phaseBegin" } | TURN_BEGIN | 任意（他人）阶段开始 → 每回合 5→6 次机会 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（global：player≠owner） [P0] | 任意阶段开始过滤：A1 后他人每回合 5→6 次发动机会（上游 1 次）——g1_shahai_swd::rexue 已红（evidence b-10.log:298，hp 2≠3） |
| H43 | SwdFengyinState（机关蜂·封印） | PhaseBegin @meta/swd.rs:1099；任意阶段开始（player==owner） @filt/swd.rs:1064-1065 | 杀海/card/swd/card.js:952｜addTempSkill("fengyin", { player: "phaseBegin" }) | TURN_BEGIN | 任意（自己）阶段开始解除 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P0] | 任意阶段开始过滤：解除点由判定开始提前到准备阶段开始（仍单次）；上游=持有者下一回合开始 |
| H44 | SwdGouhunluoSub（勾魂锣·倒计时） | PhaseBegin @meta/swd.rs:1153；任意阶段开始（gouhunluo_source==player） @filt/swd.rs:1099-1101 | 杀海/card/swd/skill.js:458｜{ global: "phaseBegin" } | TURN_BEGIN | 来源的任意阶段开始 -1 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（global：仅来源 player 的回合） [P0] | 任意阶段开始过滤：A1 后来源每回合扣倒计时 6 次（上游 1 次/回合） |
| H45 | WudianGeju（割据） | PhaseBegin @meta/wudian.rs:56；自己判定阶段开始（Judge,begin） @filt/common.rs:691-693 | ext/玩点论杀/character/skill.js:303｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H46 | WudianOwliyong（力涌） | PhaseBegin @meta/wudian.rs:211；自己摸牌阶段开始（Draw,begin,pingzhang>0） @filt/common.rs:722-724 | 杀海/character/ow/skill.js:513｜{ player: "phaseDrawBegin" } | DRAW_BEGIN | 自己摸牌阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H47 | XianjianZhaoyao（招摇） | PhaseBegin @meta/xianjian.rs:332；他人摸牌阶段开始（Draw,begin,player!=owner） @filt/xianjian.rs:381-385 | 杀海/character/xianjian/skill.js:2232｜{ global: "phaseDrawBegin" } | DRAW_BEGIN | 他人摸牌阶段开始 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H48 | XianjianFenglue（风掠） | PhaseBegin @meta/xianjian.rs:359；自己出牌阶段开始（Play,begin） @filt/xianjian.rs:143-145 | 杀海/character/xianjian/skill.js:1250｜{ player: "phaseUseBefore" } | PHASE_SPECIFIC_OTHER | 自己出牌阶段开始（近似 Before=Begin，G15 既有差异） | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H49 | XianjianQianfang（千方） | PhaseBegin @meta/xianjian.rs:408；自己判定阶段开始（Judge,begin,悬凝门） @filt/xianjian.rs:236-244 | 杀海/character/xianjian/skill.js:3473｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉死 Judge=G14 族） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |
| H50 | PhaseEndRecoverOrDraw（九尾（jiuwei）） | PhaseEnd @meta/common.rs:14；自己任意阶段结束（begin:false,player==owner） @filt/common.rs:543-544 | ext/欢乐卡牌/card/skill.js:131｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束（每回合 5 次） | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合：每回合多回/摸一次） |
| H51 | YxsMenshen（门神） | PhaseEnd @meta/common.rs:182；自己任意阶段结束（begin:false,存活>1 门） @filt/common.rs:124-127 | ext/英雄杀/character/skill.js:55｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合） |
| H52 | Xiefang（撷芳） | PhaseEnd(+Death) @meta/common.rs:265；任意死亡 / 自己摸牌阶段结束（Draw,end） @filt/common.rs:205-212 | liuli/extension/选将美化/extension.js:1164｜xiefang_mark: { global: ["dieEnd","gameDrawEnd"] }（原生自加摸牌阶段结束面近似标记刷新） | PHASE_SPECIFIC_OTHER | 死亡 + 自己摸牌阶段结束（刷新女性数标记） | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H53 | Xuanyuan（轩辕） | PhaseEnd @meta/common.rs:356；自己任意阶段结束（begin:false,黑桃门） @filt/common.rs:276-288 | 杀海/character/swd/skill.js:3255｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合） |
| H54 | Hutian（护天） | PhaseEnd @meta/common.rs:448；自己任意阶段结束（begin:false,手牌门） @filt/common.rs:394-398；联动：kern/phase.rs:217-223 | 杀海/character/swd/skill.js:2915｜{ player: "phaseEnd" }（寄存主面） | TURN_END | 自己任意阶段结束（寄存/取回由 phase.rs:221 阶段收束内联，跨一个阶段边界） | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（寄存主面）；取回面见内核 hutian 条（phase.rs:217-223） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合：主面多一次寄存确认） |
| H55 | BossHuanren（还人） | PhaseEnd @meta/common.rs:567；无事件约束（仅 boss 模式持有者 && 关卡 kind==None）——任意 PhaseEnd 帧均命中，触发后关卡状态改变即不再命中 @filt/common.rs:835-843 | core/mode/boss.js:3428｜boss_newhuanren: { global: "gameStart" } | PHASE_SPECIFIC_OTHER | 任意阶段结束即开局（原生无 gameStart 帧，登记差异） | 否→保持（可选：TurnBegin 窗承载 gameStart 语义） [P2] | 一次性：首回合无准备阶段帧，首个 PhaseEnd 仍是首回合判定阶段结束 → 不受 A1 影响（若日后首回合补发准备阶段，首触发点会变为准备阶段结束） |
| H56 | GujianFoodTanhuadong（昙花冻） | PhaseEnd @meta/common.rs:740；自己出牌阶段结束（Play,end） @filt/common.rs:623-625 | 杀海/card/gujian/skill.js:396｜tanhuadong_draw: { player: "phaseUseEnd" }（主面 phaseEnd=回合结束仅倒计时） | PLAY_END+TURN_END | 自己出牌阶段结束（摸牌+倒计时都在此扣） | 否→保持 [P2] | 无（Play 过滤）；倒计时并入出牌阶段结束（上游主面 phaseEnd=回合结束），跳过出牌阶段时原生不扣——既有登记差异 |
| H57 | GujianFoodLiyutang（鲤鱼汤） | PhaseEnd @meta/common.rs:783；自己任意阶段结束（begin:false） @filt/common.rs:610-613 | 杀海/card/gujian/skill.js:263｜{ player: "phaseEnd" }（intro：结束阶段获得护甲） | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合：每回合多获得一次护甲判定） |
| H58 | GwentQinpenState（倾盆大雨） | PhaseEnd @meta/common.rs:844；自己弃牌阶段结束（Discard,end） @filt/common.rs:647-649 | 杀海/card/gwent/skill.js:190｜gw_qinpendayu_clear: { player: "phaseDiscardAfter" } | PHASE_SPECIFIC_OTHER | 自己弃牌阶段结束移除天气 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H59 | GwentBirinState（蔽日浓雾） | PhaseEnd @meta/common.rs:852；自己出牌阶段结束（Play,end） @filt/common.rs:634-636 | 杀海/card/gwent/skill.js:217｜gw_birinongwu_clear: { player: "phaseUseAfter" } | PHASE_SPECIFIC_OTHER | 自己出牌阶段结束移除天气 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H60 | GwentFengyinState（阻魔手铐·封印） | PhaseEnd @meta/common.rs:881；自己任意阶段结束（begin:false） @filt/common.rs:678-679 | 杀海/card/gwent/card.js:1625｜addTempSkill("fengyin", { player: "phaseAfter" }) | TURN_AFTER | 自己任意阶段结束解除封印 | 是→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚） [P0] | 一次性解除：过滤为自己任意阶段结束 → A1 后在目标下一回合的准备阶段结束即解除（A1 前为判定阶段结束）；上游持续到其整回合结束后——封印几乎不覆盖目标自己的回合 |
| H61 | SwdDaixing（代形） | PhaseEnd @meta/swd.rs:26；自己结束阶段结束（End,end,有牌门） @filt/swd.rs:32-36 | 杀海/character/swd/skill.js:5280｜{ player: "phaseEnd" }（direct） | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |
| H62 | SwdDatong（大同） | PhaseEnd @meta/swd.rs:202；任意角色任意阶段结束（begin:false,极差门） @filt/swd.rs:254-263 | 杀海/character/swd/skill.js:399｜{ global: "phaseEnd" } | TURN_END | 任意玩家任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（所有玩家回合：触发次数×6/回合） |
| H63 | SwdHuanling（幻灵） | PhaseEnd @meta/swd.rs:312；自己任意阶段结束（begin:false） @filt/swd.rs:332-334 | 杀海/character/swd/skill.js:7813｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合） |
| H64 | SwdDonghuangzhongCard（东皇钟） | PhaseEnd @meta/swd.rs:365；自己任意阶段结束（begin:false,红牌门） @filt/swd.rs:982-991 | 杀海/card/swd/skill.js:952｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合） |
| H65 | SwdBaishouzhihuHeld（白兽之琥） | PhaseEnd @meta/swd.rs:400；自己弃牌阶段结束（Discard,end） @filt/swd.rs:412-420 | 杀海/card/swd/skill.js:528｜g_baishouzhihu: { player: "discardEnd" } | PHASE_SPECIFIC_OTHER | 自己弃牌阶段结束 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H66 | SwdMufengHeld（沐风） | PhaseEnd @meta/swd.rs:420；自己任意阶段结束（begin:false,用基本牌门） @filt/swd.rs:438-441 | 杀海/character/swd/skill.js:3876｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合） |
| H67 | SwdLingxin（灵心） | PhaseEnd @meta/swd.rs:647；自己任意阶段结束（begin:false） @filt/swd.rs:579-581 | 杀海/character/swd/skill.js:5411｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合） |
| H68 | SwdHuajian（化剑） | PhaseEnd @meta/swd.rs:744；自己出牌阶段结束（Play,end） @filt/swd.rs:953-957 | 杀海/character/swd/skill.js:5570｜{ player: "phaseUseEnd" } | PLAY_END | 自己出牌阶段结束 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H69 | SwdLingfeng（凌锋） | PhaseEnd @meta/swd.rs:778；自己出牌阶段结束（Play,end） @filt/swd.rs:942-944 | 杀海/character/swd/skill.js:1150｜{ player: "phaseEnd" } | TURN_END | 自己出牌阶段结束（钉死 Play=错位族） | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 现挂出牌阶段结束（既有错位，上游整回合结束）；不受准备阶段影响 |
| H70 | SwdLqingcheng（倾城） | PhaseEnd @meta/swd.rs:1060；自己任意阶段结束（begin:false） @filt/swd.rs:797-799 | 杀海/character/swd/skill.js:5362｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合） |
| H71 | SwdJiguanyuanGain（机关鸢·待获得） | PhaseEnd @meta/swd.rs:1108；自己任意阶段结束（begin:false,off_court 门） @filt/swd.rs:1066-1069 | 杀海/card/swd/skill.js:503｜jiguanyuan: { player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 一次性获得：过滤为自己任意阶段结束 → A1 后提前到准备阶段结束获得（A1 前为判定阶段结束）；上游在其整回合结束 |
| H72 | WudianFuchou2（负仇（fuchou2）） | PhaseEnd @meta/wudian.rs:20；任意玩家任意阶段结束（begin:false,标记存活门） @filt/common.rs:23-30 | ext/玩点论杀/character/skill.js:553｜{ global: "phaseAfter" } | TURN_AFTER | 任意玩家任意阶段结束 | 是→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）（global） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（所有玩家回合：触发次数×6/回合） |
| H73 | WudianZhucheng（筑城） | PhaseEnd @meta/wudian.rs:37；自己任意阶段结束（begin:false,无标记门） @filt/common.rs:773-777；联动：kern/phase.rs:128-152（每阶段收束直推 OptionalGate） | ext/玩点论杀/character/skill.js:161｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束（kernel phase.rs:128-151 在每阶段收束直推 OptionalGate） | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合：每回合多一次筑城确认） |
| H74 | WudianGuangshuSpade（光塔（guangshu_spade）） | PhaseEnd @meta/wudian.rs:84；自己任意阶段结束（begin:false） @filt/common.rs:601-602；联动：kern/phase.rs:183-201（每阶段收束内联雷伤并移除——内核直推会先于宿主触发窗执行） | 杀海/character/ow/skill.js:3115｜guangshu_spade: { player: "phaseEnd" }（intro：下个结束阶段受雷伤） | TURN_END | 自己任意阶段结束（kernel phase.rs:183-201 内联在 PhaseEnd 收束） | 是→整回合结束窗（与宿主面同步；若按 intro 语义则 Phase{End,end}——需在规则基准登记取舍） [P1] | 一次性（结算后移除）：宿主过滤为任意阶段结束 + 内核每阶段收束内联结算（phase.rs:183-201，先于宿主窗）→ 获得后首个阶段收束即受伤；A1 后可在准备阶段结束就结算（更早），不多触发 |
| H75 | WudianZiyu（自愈） | PhaseEnd @meta/wudian.rs:92；任意玩家任意阶段结束（phase_number%4==0 门） @filt/common.rs:598-600 | 杀海/character/ow/skill.js:3123｜{ global: "phaseEnd" } + game.phaseNumber%4===0 | TURN_END | 任意阶段结束且 phase_number%4==0（phase_number 每非准备阶段+1） | 是→整回合结束窗；同时登记 phaseNumber 口径差异：上游 game.phaseNumber 每回合 +1（content.ts:4670），原生 phase_number 每回合 +5（phase.rs:109-114 豁免准备阶段后仍 +5）→ %4 命中节奏不同 [P1] | 任意阶段结束多一帧（准备阶段结束）；%4 门按 phase_number 计数——豁免准备阶段后每回合仍 +5，与上游每回合 +1 的节奏差异属既有问题，A1 未改变 |
| H76 | WudianPaotaiFire（炮台（paotai2）） | PhaseEnd @meta/wudian.rs:105；自己任意阶段结束（begin:false,paotai_lv 门） @filt/common.rs:804-805 | 杀海/character/ow/skill.js:896｜paotai2: { player: "phaseEnd" }（概率门） | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合：每回合最多 6 次概率结算） |
| H77 | WudianYuedong（乐动） | PhaseEnd @meta/wudian.rs:123；自己出牌阶段结束（Play,end） @filt/common.rs:767-769 | 杀海/character/ow/skill.js:2840｜{ player: "phaseUseEnd" } | PLAY_END | 自己出牌阶段结束 | 否→保持 [P2] | 准备阶段已是独立帧；过滤仅匹配具体阶段，行为不变 |
| H78 | WudianJidong（急冻） | PhaseEnd @meta/wudian.rs:133；任意玩家任意阶段结束（begin:false,hp==1 门） @filt/common.rs:711-714 | 杀海/character/ow/skill.js:1858｜{ global: "phaseEnd" } | TURN_END | 任意玩家任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（所有玩家回合） |
| H79 | WudianXiyang（吸氧） | PhaseEnd @meta/wudian.rs:143；自己结束阶段结束（End,end） @filt/common.rs:762-766 | 杀海/character/ow/skill.js:580｜{ player: "phaseEnd" }（原生注释=回复面钉 End 结束） | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |
| H80 | XianjianBinxin（冰心） | PhaseEnd @meta/xianjian.rs:14；任意玩家任意阶段结束（begin:false,hp==1 门） @filt/xianjian.rs:394-397；联动：host/factory.rs:138（按 Phase 帧取目标） | 杀海/character/xianjian/skill.js:1520｜{ global: "phaseEnd" } | TURN_END | 任意玩家任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（所有玩家回合） |
| H81 | XianjianLingyu（灵愈） | PhaseEnd @meta/xianjian.rs:122；自己任意阶段结束（begin:false,有人受伤门） @filt/xianjian.rs:95-101 | 杀海/character/xianjian/skill.js:2652｜{ player: "phaseEnd" } | TURN_END | 自己任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（自己回合） |
| H82 | XianjianLonghuo（龙火） | PhaseEnd @meta/xianjian.rs:156；自己结束阶段结束（End,end,姜城门） @filt/xianjian.rs:223-227 | 杀海/character/xianjian/skill.js:1771｜{ player: "phaseEnd" }（原生钉 End 结束） | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |
| H83 | XianjianLinghuo（灵火） | PhaseEnd @meta/xianjian.rs:177；他人任意阶段结束（begin:false,player!=owner,turn_damage 门） @filt/xianjian.rs:209-213；联动：host/factory.rs:151（按 Phase 帧取目标） | 杀海/character/xianjian/skill.js:576｜{ global: "phaseEnd" } filter event.player!==player | TURN_END | 他人任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global） [P1] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（他人回合：判定条件 turn_damage 于准备阶段结束时恒 0=无害但语义脏） |
| H84 | XianjianShuiyun（水蕴） | PhaseEnd @meta/xianjian.rs:188；自己结束阶段结束（End,end,藏牌门） @filt/xianjian.rs:367-379 | 杀海/character/xianjian/skill.js:2963｜{ player: "phaseEnd" }（原生钉 End 结束） | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |
| H85 | XianjianWangyou（忘忧） | PhaseEnd @meta/xianjian.rs:204；他人任意阶段结束（begin:false,player!=owner,wangyou3 门） @filt/xianjian.rs:305-313 | 杀海/character/xianjian/skill.js:3091｜{ global: "phaseEnd" } | TURN_END | 他人任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（他人回合） |
| H86 | XianjianLingquan（灵泉） | PhaseEnd @meta/xianjian.rs:220；自己结束阶段结束（End,end,觉醒门） @filt/xianjian.rs:214-220 | 杀海/character/xianjian/skill.js:6｜{ player: "phaseEnd" }（觉醒技） | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |
| H87 | XianjianShenwu（神舞） | PhaseEnd @meta/xianjian.rs:228；任意玩家任意阶段结束（begin:false,shuiyun>=3 觉醒门） @filt/xianjian.rs:277-283 | 杀海/character/xianjian/skill.js:27｜{ global: "phaseEnd" }（觉醒技） | TURN_END | 任意玩家任意阶段结束 | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（global） [P1] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次（所有玩家回合；觉醒门缓解） |
| H88 | XianjianXjzhimeng（织梦） | PhaseEnd @meta/xianjian.rs:268；自己结束阶段结束（End,end） @filt/xianjian.rs:317-322 | 杀海/character/xianjian/skill.js:3857｜{ player: "phaseEnd" } | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |
| H89 | XianjianHusha（虎煞） | PhaseEnd @meta/xianjian.rs:300；自己结束阶段结束（End,end,标记门） @filt/xianjian.rs:152-157 | 杀海/character/xianjian/skill.js:1621｜{ player: "phaseEnd" } | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |
| H90 | XianjianChangnian2（追思（changnian2）） | PhaseEnd @meta/xianjian.rs:316；自己结束阶段结束（End,end） @filt/xianjian.rs:118-120 | 杀海/character/xianjian/skill.js:3201｜{ player: "phaseEnd" } | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |
| H91 | XianjianLiguang（离光） | PhaseEnd @meta/xianjian.rs:340；自己结束阶段结束（End,end,可移动牌门） @filt/xianjian.rs:182-207 | 杀海/character/xianjian/skill.js:834｜{ player: "phaseEnd" } | TURN_END | 自己结束阶段结束 | 否→保持 Phase{End,end}（单帧近似可接受）；整回合结束窗建成后宜迁 整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 不多触发（另：PhaseEnd{End} 触发窗在 Commit 之后，而 Commit 发 TurnEnded→TurnBegin，故实际派发在下一位角色 TurnBegin→准备→判定开始子树之后，见 L2） |

### 5.2 标准技能

| ID | 原生名（中文） | 原生 event＋阶段约束（file:line） | 上游 file:line｜trigger | 分类 | 原生现状 | 需改→改成 | 准备阶段影响/风险 |
|---|---|---|---|---|---|---|---|
| S01 | TU_XI（突袭） | @std/standard_skills.rs:128；自己摸牌阶段开始（Draw,begin） @std/skills/tu_xi.rs:9-16 | core/character/standard/skill.js:608｜{ player: "phaseDrawBegin1" } | DRAW_BEGIN | 自己摸牌阶段开始 | 否→保持 [P2] | 准备阶段不匹配 Draw |
| S02 | LUO_YI（裸衣） | @std/standard_skills.rs:144；自己摸牌阶段开始（Draw,begin） @std/skills/luo_yi.rs:11-18 | core/character/standard/skill.js:649｜{ player: "phaseDrawBegin2" } | DRAW_BEGIN | 自己摸牌阶段开始 | 否→保持 [P2] | 同上 |
| S03 | GUAN_XING（观星） | @std/standard_skills.rs:160；自己判定阶段开始（Judge,begin） @std/skills/guan_xing.rs:10-18 | core/character/standard/skill.js:1329｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 自己判定阶段开始（替位准备阶段） | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| S04 | BI_YUE（闭月） | @std/standard_skills.rs:376；自己结束阶段开始（End,begin） @std/skills/bi_yue.rs:8-14 | core/character/standard/skill.js:2308｜{ player: "phaseJieshuBegin" } | JIESHU_BEGIN | 自己结束阶段开始 | 否→保持 [P2] | 无 |
| S05 | LUO_SHEN（洛神） | @std/standard_skills.rs:389；自己判定阶段开始（Judge,begin） @std/skills/luo_shen.rs:9-17 | core/character/standard/skill.js:839｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 自己判定阶段开始（替位准备阶段） | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| S06 | YING_ZI（英姿） | @std/standard_skills.rs:422；自己摸牌阶段开始（Draw,begin） @std/skills/ying_zi.rs:8-14 | core/character/standard/skill.js:1716｜{ player: "phaseDrawBegin2" } | DRAW_BEGIN | 自己摸牌阶段开始 | 否→保持 [P2] | 无 |
| S07 | JU_SHOU（据守） | @std/standard_skills.rs:478；自己结束阶段开始（End,begin） @std/skills/jushou.rs:11-18 | core/character/shenhua/skill.js:7735｜{ player: "phaseJieshuBegin" } | JIESHU_BEGIN | 自己结束阶段开始 | 否→保持 [P2] | 无 |
| S08 | XIAO_GUO（骁果） | @std/standard_skills.rs:491；他人结束阶段开始（End,begin,player!=owner） @std/skills/xiao_guo.rs:10-20 | core/character/sp/skill.js:44338｜{ global: "phaseJieshuBegin" } | JIESHU_BEGIN | 他人结束阶段开始 | 否→保持 [P2] | 无 |
| S09 | SHEN_ZHI（神智） | @std/standard_skills.rs:546；自己判定阶段开始（Judge,begin,手牌门） @std/skills/shen_zhi.rs:10-14 | core/character/sp/skill.js:44739｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 自己判定阶段开始（登记差异：无准备阶段以判定开始锚） | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| S10 | XUN_XUN（恂恂） | @std/standard_skills.rs:559；自己摸牌阶段开始（Draw,begin） @std/skills/xun_xun.rs:11-15 | core/character/refresh/skill.js:17339｜{ player: "phaseDrawBegin1" } | DRAW_BEGIN | 自己摸牌阶段开始 | 否→保持 [P2] | 无 |
| S11 | WANG_ZUN（妄尊） | @std/standard_skills.rs:614；他人判定阶段开始（Judge,begin,体力门） @std/skills/wang_zun.rs:11-18 | core/character/sp/skill.js:43352｜{ global: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 他人判定阶段开始（替位准备阶段） | 是→Phase{Prepare,begin}（global：player≠owner） [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| S12 | BOSS_FUTAI（覆泰） | @std/boss_skills.rs:60；自己判定阶段开始（Judge,begin） @std/skills/futai.rs:8-11 | core/mode/boss.js:5417｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 自己判定阶段开始（注释自称「神智同口径」） | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| S13 | BOSS_LUANCHANG（乱常·南蛮面） | @std/boss_skills.rs:119；自己判定阶段开始（Judge,begin） @std/skills/luan_chang.rs:10-12 | core/mode/boss.js:5445｜subSkill: { player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 自己判定阶段开始（万箭面在 kernel Phase::End 臂=end.rs:436） | 是→Phase{Prepare,begin}（南蛮面）；万箭面 phaseJieshuBegin 已由 kernel/phase/end.rs:436-462 正确承载 [P1] | 现挂判定阶段开始：A1 后真实准备阶段已存在，本技晚于准备阶段窗口；迁到准备阶段前须先把回合重置（K25：reset_turn_limits/turn_damage/cards_used_turn）移到 TurnBegin 之前，否则 once_per_turn 技（观星）在准备阶段读到上回合 once_used 而被挡 |
| S14 | BOSS_SHEDU（蛇毒） | @std/boss_skills.rs:195；自己判定阶段开始（Judge,begin,标记门） @std/skills/shed_u.rs:9-13 | core/mode/boss.js:4487｜{ player: "phaseBegin" } | TURN_BEGIN | 自己判定阶段开始（钉 Judge） | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始：A1 后 TurnBegin 窗与准备阶段开始/结束都先于判定开始结算 → 本技晚于全部回合开始/准备阶段技能（上游 phaseBegin 先于 phaseZhunbei）；不多触发 |

### 5.3 内核直写效果与生命周期

| ID | 原生名（中文） | 原生 event＋阶段约束（file:line） | 上游 file:line｜trigger | 分类 | 原生现状 | 需改→改成 | 准备阶段影响/风险 |
|---|---|---|---|---|---|---|---|
| K01 | anwugu2 递减（巫蛊；原生注释称「蛊舞」） | !begin（任意阶段收束）时 anwugu_count-1，归 0 移除 @kern/phase.rs:55-60 | 杀海/character/xianjian/skill.js:501｜anwugu2: { player: "phaseAfter" } silent（intro「剩余$回合」=按回合递减） | TURN_AFTER | 每阶段结束递减（每回合 5→6 次） | 是→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）（递减一次/回合） [P0] | 准备阶段结束帧额外递减一次 → 剩余回合数消耗×6/回合（g1_wudian::anu 测试已红：b-10.log:69） |
| K02 | qianggu2 解除（强固） | begin && Judge 时清除 qianggu_active @kern/phase.rs:34-38 | 杀海/character/ow/skill.js:142｜addTempSkill("qianggu2", { player: "phaseBegin" }) | TURN_BEGIN | 判定阶段开始解除 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） 的 Commit 前/BeforeTriggers（解除类无需触发窗，可在 TurnBegin 提交前内联） [P1] | 解除时点从回合起点漂移到判定阶段开始（差一个准备阶段宽度） |
| K03 | qianxing 解除（隐身 tempHide） | begin && Judge 时清除 yinshen_active @kern/phase.rs:39-43 | 杀海/character/ow/skill.js:718｜player.tempHide() → addTempSkill("qianxing", { player: "phaseBeginStart" }) | PHASE_SPECIFIC_OTHER | 判定阶段开始解除 | 是→TurnBegin 之前（回合交接时、发 TurnBegin 前解除；上游 qianxing 至 phaseBeginStart，先于 phaseBegin） [P1] | 同上漂移 |
| K04 | hslingjian_chaofeng 解除（嘲讽） | begin && Judge 时清除 chaofeng_active @kern/phase.rs:44-48 | 杀海/card/swd/card.js:1506｜addTempSkill("hslingjian_chaofeng", { player: "phaseBegin" }) | TURN_BEGIN | 判定阶段开始解除 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（同上，解除内联） [P1] | 同上 |
| K05 | hslingjian_xingtigaizao 重置（星体改造） | begin && Judge 时清零 xingtigaizao_count @kern/phase.rs:49-54 | 杀海/card/swd/skill.js:2204｜hslingjian_xingtigaizao: { player: "phaseEnd" }（移除+清零在「其下一结束阶段」） | TURN_END | 本回合开始重置（登记差异：重置时点=下一回合开始而非上一结束） | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21） [P2] | 既有登记差异+新漂移：Judge 锚在 A1 后不再等于回合开始 |
| K06 | yuexing2 清除（越行） | !begin && Play 时清距离交换对 @kern/phase.rs:61-71 | 杀海/character/xianjian/skill.js:246｜addTempSkill("yuexing2","phaseUseAfter") | PHASE_SPECIFIC_OTHER | 出牌阶段结束清除 | 否→保持 [P2] | Play,end 过滤不受准备阶段影响 |
| K07 | zhuyue2 清白名单（逐月） | !begin && Play 时清 zhuyue_targets @kern/phase.rs:72-76 | 杀海/character/xianjian/skill.js:3675｜zhuyue2: { player: "phaseUseEnd" } silent | PLAY_END | 出牌阶段结束清除 | 否→保持 [P2] | 无 |
| K08 | chengxin2 清除（澄心） | !begin && End 时清 chengxin_active @kern/phase.rs:77-81 | 杀海/character/xianjian/skill.js:732｜addTempSkill("chengxin2", { player: "phaseAfter" }) | TURN_AFTER | 结束阶段结束清除（登记差异：上游存活至最近回合结束） | 否→保持（整回合结束窗建成后可迁 phaseAfter 段） [P2] | 既有差异；End,end 与 phaseAfter 相差一个收束段，A1 未扩大此差 |
| K09 | pingzhang_count 转计数（屏障） | begin && Judge 时两标志转计数并清零 @kern/phase.rs:82-92 | 杀海/character/ow/skill.js:489｜pingzhang_count: { player: "phaseBegin" } silent | TURN_BEGIN | 判定阶段开始转计数 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（同上，内联转计数） [P1] | 转计数时点漂移一个阶段（应用技在随后摸牌阶段读——仍先于摸牌，功能不破但时点偏离上游） |
| K10 | 翻面跳过整回合（E03） | begin && Judge && face_down → TurnOverBefore+直达回合收束 @kern/phase.rs:21-33 | core/noname/library/element/content.ts:4750｜phase 事件 step⑤ 翻面检测 isTurnedOver→turnOver+event.cancel()（content.ts:4750），先于 phaseBeginStart（4836）/phaseBegin（4841）；命中则整回合无 phaseBegin/准备阶段 | PHASE_SPECIFIC_OTHER | 判定阶段开始时检测：A1 后翻面角色先跑完 TurnBegin 窗与准备阶段开始/结束，再在判定开始被翻回跳过 | 是→TurnBegin 之前（commit_turn_ended 发 TurnBegin 前检测翻面并直达回合收束，不发 TurnBegin/准备阶段帧） [P0] | A1 回归：翻面角色在上游整回合无任何时机，现在会触发 TurnBegin/准备阶段技能（迁移 ZHUNBEI/TURN_BEGIN 技后影响扩大） |
| K11 | huanxia_gain 回收（幻霞） | !begin && End 时收回未造成伤害的还侠杀 @kern/phase.rs:153-172 | 杀海/character/swd/skill.js:1653｜huanxia_gain: { player: "phaseEnd" } | TURN_END | 结束阶段结束回收（登记差异：上游特殊区/回合结束） | 否→保持 [P2] | End,end 不受准备阶段影响 |
| K12 | mtg_shuimomuxue_skill（水没墓穴） | Discard 收束弃牌数门摸 1 @kern/phase.rs:173-182 | 杀海/card/mtg/skill.js:99｜mtg_shuimomuxue_skill: { player: "phaseDiscardEnd" } | DISCARD_END | 弃牌阶段结束 | 否→保持 [P2] | 无 |
| K13 | guangshu_spade 雷伤（光塔） | 任意阶段收束（!begin）持有即受 1 雷伤并移除 @kern/phase.rs:183-201 | 杀海/character/ow/skill.js:3115｜{ player: "phaseEnd" }（intro=下个结束阶段） | TURN_END | 任意阶段收束（宿主元数据同款问题） | 是→整回合结束窗（与 WudianGuangshuSpade 宿主条同步取舍；若按 intro 则 Phase{End,end}） [P1] | 准备阶段收束提前结算雷伤 |
| K14 | zhucheng 确认门（筑城） | 每阶段收束（PhaseEnd 帧）直推确认门 @kern/phase.rs:128-152 | ext/玩点论杀/character/skill.js:161｜{ player: "phaseEnd" } optional | TURN_END | 每阶段收束确认 | 是→整回合结束窗（与 WudianZhucheng 条联动） [P0] | 任意阶段结束过滤会额外匹配准备阶段结束帧 → 多触发一次 |
| K15 | songci_draw（颂词组技） | !begin && Discard 时全记录则摸 1 @kern/phase.rs:226-247 | core/character/sp/skill.js:45641｜songci_draw: { player: "phaseDiscardEnd" } forced | DISCARD_END | 弃牌阶段收束 | 否→保持 [P2] | 无 |
| K16 | hxunzhi2 死亡（殉志） | !begin && Play 时持有「殉」者死亡 @kern/phase.rs:202-216 | 杀海/character/swd/skill.js:1266｜hxunzhi2: { player: "phaseUseEnd" } forced | PLAY_END | 出牌阶段收束 | 否→保持 [P2] | 无 |
| K17 | hutian3 寄存取回（护天） | !begin 任意阶段收束归还寄存（跨一个阶段边界） @kern/phase.rs:217-223 | 杀海/character/swd/skill.js:3004｜hutian3: { player: ["phaseEnd","dieBegin"] } | TURN_END | 跨任一阶段边界归还（登记：上游跨回合边界） | 是→整回合结束窗·phaseEnd 段（需新建：在回合交接前派发；现有 TurnEnded 帧不等价，见 L1=K21）（取回须加「非寄存当回合」守卫=上游 hutian4 临时技） [P0] | 任意阶段收束归还：A1 后准备阶段结束即归还，寄存期更短（上游跨一整轮至持有者下一回合结束） |
| K18 | lingwu 额外阶段（灵舞） | !begin 任意阶段收束判定插额外 Play @kern/phase.rs:261-282 | 杀海/character/swd/skill.js:5440｜lingwu: { player: "phaseAfter" } frequent | TURN_AFTER | 每阶段收束判定（上游=回合结束后一次） | 是→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚）；效果亦应为 insertPhase=额外回合（原生为额外出牌阶段，既有差异） [P0] | A1 回归：准备阶段收束提交时 cards_used_turn 尚未清零（begin_judge 才清）而是上一位角色的计数 → 可能在判定前插入出牌阶段，Play 之后 phase.next()=Discard，判定/摸牌与 turn_opened 回合重置被整体跳过 |
| K19 | 乐不思蜀跳 Play | Draw 收束 skip_play_phase → 直进 Discard @kern/phase.rs:248-260 | 延迟锦囊判定效果（非技能） | PHASE_SPECIFIC_OTHER | 摸牌阶段结束跳过 | 否→保持 [P2] | 无 |
| K20 | 额外阶段队列 | 阶段收束弹 extra_phases @kern/phase.rs:283-300 | insertPhase 原语 | PHASE_SPECIFIC_OTHER | 阶段推进 | 否→保持 [P2] | 无（灵武条单独列） |
| K21 | 回合收束 commit_turn_ended [L1] | 清旗标/换人/置 phase=Judge → A1 后发 TurnBegin @kern/events.rs:702-825 | core/noname/library/element/content.ts:4913｜event.trigger("phaseEnd") → step11 phaseAfter | TURN_END+TURN_AFTER | TurnEnded 帧有 AfterTriggers 窗，但起步于 Commit：commit_turn_ended 先换人并发 TurnBegin，TurnBegin 压在其上，故 TurnEnded 的触发要等下一位角色 TurnBegin→准备开始/结束→判定开始（含判定区结算）整棵子树结算完才派发，且 turn.current 已切换；其 Commit 内联清理（expire_temporary_skills 等）时点≈phaseAfter | 是→新建回合结束窗：在交接前派发 phaseEnd/phaseAfter 两段（例：TurnEnded 改 BeforeTriggers 起步、触发在 Commit 前；或 TurnEnded 提交只做清理，TurnBegin 推迟到其 after 段完成后再发） [P0] | 现有 TurnEnded 既不等价 phaseEnd 也不等价 phaseAfter；所有 TURN_END/TURN_AFTER 迁移都依赖此项先落地 |
| K22 | commit_turn_begin [L3] | TurnBegin Commit：置 phase=Prepare、压 Prepare end+begin 帧 @kern/events.rs:682-700 | core/noname/library/element/content.ts:4841｜event.trigger("phaseBegin")（回合开始后⑨，阶段列表前） | TURN_BEGIN | TurnBegin BeforeTriggers→Commit→静默 after；提交步自动接准备阶段→判定 | 是→①发 TurnBegin 前把 turn.phase 置 Prepare（或独立的回合开始态），避免 TurnBegin 窗读到 Judge；②rvm_bridge::trigger_ctx_of 补 TurnBegin（及 TurnEnded）臂给 event_player；③首回合（建局直接 Judge+turn_opened=true）补发 TurnBegin/准备阶段，或在规则基准登记首回合差异 [P1] | TurnBegin BeforeTriggers 段时 turn.phase 仍为 commit_turn_ended 写入的 Judge（events.rs:767/800），读 turn.phase 的过滤（含 VM phase_is 回退 rvm_named.rs:721-725）会误判为判定阶段；trigger_ctx_of（rvm_bridge.rs:194 default 臂）不给 TurnBegin 帧 event_player → VM 技能挂 TurnBegin 拿不到事件玩家；首回合（construction.rs:282/295 建局即 Judge+turn_opened）不发 TurnBegin/准备阶段 |
| K23 | mtg_feixu_skill（废墟地图） | 每次 Judge begin（含首帧外的每次判定阶段开始）随机置牌 @kern/phase/judge.rs:746-756 | 杀海/card/mtg/skill.js:119｜mtg_feixu_skill: { player: "phaseBegin" } silent | TURN_BEGIN | 判定阶段开始置牌 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（上游 player phaseBegin silent；现每次判定阶段开始执行，含额外判定阶段） [P1] | 时点漂移一个阶段 |
| K24 | phase_number 计数 | 每非 Prepare 阶段开始 +1（注释：保持每回合五阶段口径） @kern/phase.rs:109-114 | core/noname/library/element/content.ts:4670｜game.phaseNumber++（每回合 phase 事件内 +1，另有 player.phaseNumber） | PHASE_SPECIFIC_OTHER | 每回合 +5（准备阶段豁免） | 是→与上游 game.phaseNumber（每回合+1）对齐或明确登记差异；WudianZiyu %4 门依赖此数 [P2] | 上游 %4=每 4 个回合，原生 %4=每 4/5 回合错拍——既有差异，A1 未改但需在报告中登记 |
| K25 | reset_turn_limits 时点 | 首帧 Judge begin 时清 once_used（turn_opened 门） @nk/machine.rs:831-837（a605379c；HEAD 953213ab 为 839-845）＋kern/phase/judge.rs:4-25,742-744 | core/noname/library/element/content.ts:4669｜phase 事件 step②：game.phaseNumber++、各角色 actionHistory/stat 入栈（content.ts:4669-4736），先于 phaseBeginStart（4836）/phaseBegin（4841） | PHASE_SPECIFIC_OTHER | 判定阶段开始重置每回合限一次 | 是→TurnBegin 之前（commit_turn_ended 发 TurnBegin 前；连同 turn_opened 块中的 turn_damage/cards_used_turn/sheling 等回合统计清零） [P0] | TurnBegin/准备阶段窗内 once_used、cards_used_turn（仍是上一位角色的）、turn_damage 均为陈旧值；准备阶段造成的伤害会在判定开始被 turn_damage=0 抹掉（影响诛害/灵火/延妒等按本回合伤害判定的技能） |
| K26 | 红莲（Boss 毕方 boss_honglianx） | kernel 直写（无触发窗） @kern/phase/judge.rs:26-80 | core/mode/boss.js:4705｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 回合开始（判定开始锚）获红牌+火伤 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K27 | 绝红（boss_juehong） | kernel 直写（无触发窗） @kern/phase/judge.rs:81-117 | core/mode/boss.js:4898｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K28 | 夹击（boss_jiazu） | kernel 直写（无触发窗） @kern/phase/judge.rs:118-154 | core/mode/boss.js:5095｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K29 | 造异异变（boss_zaoyi_hp） | kernel 直写（无触发窗） @kern/phase/judge.rs:155-183 | core/mode/boss.js:4939｜subSkill hp: { player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K30 | 形态轮换（boss_nbianshen） | kernel 直写（无触发窗） @kern/phase/judge.rs:184-237 | core/mode/boss.js:8210｜{ player: "phaseBefore" } | PHASE_SPECIFIC_OTHER | 判定开始锚换将 | 是→TurnBegin 之前（换将须先于 TurnBegin/准备阶段，使新形态的回合开始技生效） [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口；旧形态的准备阶段技会先触发再换将 |
| K31 | 自润（boss_zirun） | kernel 直写（无触发窗） @kern/phase/judge.rs:238-259 | core/mode/boss.js:4882｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K32 | 魔道（boss_modao） | kernel 直写（无触发窗） @kern/phase/judge.rs:260-268 | core/mode/boss.js:8504｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K33 | 轮回（boss_lunhui） | kernel 直写（无触发窗） @kern/phase/judge.rs:269-299 | core/mode/boss.js:3957｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K34 | 猛态·始（boss_mengtai_begin） | kernel 直写（无触发窗） @kern/phase/judge.rs:300-311 | core/mode/boss.js:8157｜subSkill begin: { player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚置双旗标 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K35 | 睿智（boss_ruizhi） | kernel 直写（无触发窗） @kern/phase/judge.rs:312-356 | core/mode/boss.js:8065｜{ global: "phaseZhunbeiBegin" } filter event.player!==player && (h>1\|\|e>1)；content 只作用 trigger.player | ZHUNBEI_BEGIN | 有持有者即在每名角色判定开始对**全体**角色执行（目标过宽，既有差异） | 是→Phase{Prepare,begin}（global，仅当前回合角色且≠持有者） [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口；且原生对全体角色执行（上游只对当前回合角色） |
| K36 | 鬼计（boss_guiji） | kernel 直写（无触发窗） @kern/phase/judge.rs:357-369 | core/mode/boss.js:8693｜{ player: "phaseJudgeBegin" } | JUDGE_BEGIN | 判定开始（注释误称「准备阶段」） | 否→保持（可选：移出 turn_opened 块） [P2] | 无 |
| K37 | 公正（boss_gongzheng） | kernel 直写（无触发窗） @kern/phase/judge.rs:370-384 | core/mode/boss.js:3679｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K38 | 吸行（boss_xixing） | kernel 直写（无触发窗） @kern/phase/judge.rs:385-410 | core/mode/boss.js:8544｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K39 | 雷舟（boss_leizhou） | kernel 直写（无触发窗） @kern/phase/judge.rs:411-430 | core/mode/boss.js:3744｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K40 | 苦酒（boss_kujiu） | kernel 直写（无触发窗） @kern/phase/judge.rs:431-454 | core/mode/boss.js:3805｜{ global: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 他人判定开始锚 | 是→Phase{Prepare,begin}（global，持有者≠当前回合角色） [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K41 | 不佛（boss_bufo，准备面） | kernel 直写（无触发窗） @kern/phase/judge.rs:455-479 | core/mode/boss.js:4195｜{ player: ["damageBegin4","phaseZhunbeiBegin"] } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K42 | 无量（boss_wuliang，准备面） | kernel 直写（无触发窗） @kern/phase/judge.rs:480-497 | core/mode/boss.js:4222｜{ global:"gameDrawAfter", player:["phaseZhunbeiBegin","phaseJieshuBegin","enterGame"] } | ZHUNBEI_BEGIN | 判定开始锚回复至 3 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K43 | 炙焚（boss_zhifen） | kernel 直写（无触发窗） @kern/phase/judge.rs:498-541 | core/mode/boss.js:3875｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K44 | 锁足（boss_suozu） | kernel 直写（无触发窗） @kern/phase/judge.rs:542-572 | core/mode/boss.js:3911｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K45 | 往事（boss_wangshi） | kernel 直写（无触发窗） @kern/phase/judge.rs:573-595 | core/mode/boss.js:4050｜{ global: "phaseZhunbeiBegin" } filter 敌方 | ZHUNBEI_BEGIN | 他人判定开始锚 | 是→Phase{Prepare,begin}（global） [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K46 | 封冻（boss_fengdong） | kernel 直写（无触发窗） @kern/phase/judge.rs:596-611 | core/mode/boss.js:4788｜{ player: "phaseBegin" } | TURN_BEGIN | 判定开始锚授封印 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}） [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K47 | 巡游（boss_xunyou） | kernel 直写（无触发窗） @kern/phase/judge.rs:612-673 | core/mode/boss.js:4799｜{ global: "phaseBegin" } filter player!==event.player | TURN_BEGIN | 他人判定开始锚 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（global） [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K48 | 炎狱（boss_yanyu） | kernel 直写（无触发窗） @kern/phase/judge.rs:674-711 | core/mode/boss.js:4765｜{ global: "phaseBegin" } filter player!==event.player | TURN_BEGIN | 他人判定开始锚 | 是→TurnBegin 窗（EventKind::TurnBegin，BeforeTriggers 段；过滤改匹配 EventData::TurnBegin{player}）（global） [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K49 | 中毒（boss_zhongdu） | kernel 直写（无触发窗） @kern/phase/judge.rs:712-741 | core/mode/boss.js:6957｜{ player: "phaseZhunbeiBegin" } | ZHUNBEI_BEGIN | 判定开始锚 | 是→Phase{Prepare,begin} [P1] | 现挂判定阶段开始（turn_opened 一次性块，每回合首个判定开始执行一次）：A1 后晚于 TurnBegin 与准备阶段开始/结束窗口 |
| K50 | 封冻解除（end.rs） | kernel 直写（无触发窗） @kern/phase/end.rs:349-352 | core/mode/boss.js:4793｜current.addTempSkill("fengyin")（无 expire→默认 {global:["phaseAfter","phaseBeforeStart"]}，player.js:11966） | TURN_AFTER | 结束阶段开始全清 | 是→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚） [P2] | 封印在结束阶段内提前失效（被封者的结束阶段/他人结束阶段技可误发） |
| K51 | 反噬 terra 清零（end.rs） | kernel 直写（无触发窗） @kern/phase/end.rs:353-357 | core/mode/boss.js:4013｜addTempSkill("boss_zlfanshi_terra")（默认回合结束后过期） | TURN_AFTER | 结束阶段开始全清（另 judge.rs:5-7 与 events.rs:706-708 也各清一次） | 是→保留 events.rs:706-708（回合收束≈phaseAfter），删去结束阶段开始与判定开始两处 [P2] | 结束阶段内的第二次受伤不再计入「本回合首次受伤后」 |
| K52 | 往事封禁清零（end.rs） | kernel 直写（无触发窗） @kern/phase/end.rs:358-362 | core/mode/boss.js:4062｜trigger.player.addTempSkill("boss_wangshi2")（默认回合结束后过期） | TURN_AFTER | 结束阶段开始全清 | 是→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚） [P2] | 结束阶段内封禁提前解除 |
| K53 | 巨源记体力（end.rs） | kernel 直写（无触发窗） @kern/phase/end.rs:363-373 | core/mode/boss.js:4594｜{ player: "phaseAfter" } | TURN_AFTER | 结束阶段开始记 hp | 是→整回合结束窗·phaseAfter 段（需新建，同上；TurnEnded 的 Commit 动作位置≈phaseAfter 后，但其触发窗过晚） [P2] | 结束阶段内的体力变化不计入记录 |
| K54 | 结束阶段开始族（end.rs 22 项：爆敛/强征/枭首/索命/天狱/拘魂/无量·结束面/炼狱/血池/胜负/雷斧/红莲·周瑜/芳华/枭雄·伤/伤逝·失/鬼谋/猛态·终/兵先/兽九·补/贪欲·失/乱常·万箭/延妒） | kernel 直写（无触发窗） @kern/phase/end.rs:4-348,374-494 | 均为 phaseJieshuBegin：boss.js 8589/8958/8596/8560/9451/3351/4222/8677/3690/3632/3758/9422/9523/7320(global)/9685/7108/8195/4581(global)/4524/5550/5459/5405(global) | JIESHU_BEGIN | 结束阶段开始 | 否→保持 [P2] | 无（兵先原生注释「回合结束时」实为结束阶段开始，与上游相符） |
| K55 | L2 PhaseEnd 帧触发窗时序（PhaseEnd 帧触发窗时序） [L2] | PhaseEnd 帧起步 Commit：commit_phase 先压下一阶段开始帧（或 TurnEnded→TurnBegin），其 AfterTriggers 要等子树结算完 @nk/resolution.rs:610-630（a605379c 行号；Commit 后才进 AfterTriggers）+ kernel/phase.rs:301-315（收束提交即压下一阶段 begin / TurnEnded） | core/noname/library/element/content.ts:4913｜上游 phaseXEnd 在本阶段事件内结算完毕后才进入下一阶段；phaseEnd/phaseAfter 在全部阶段之后、交接之前 | ZHUNBEI_END/JIESHU_END/TURN_END（结构性） | PhaseEnd 触发晚于下一阶段开始子树：Prepare,end 的触发在判定开始（含判定区结算）之后；End,end 的触发在下一位角色 TurnBegin→准备→判定开始之后 | 是→A2 若要把任何技能落到 Phase{Prepare,end} 或整回合结束窗，须先让 PhaseEnd 帧「先触发后提交」（或另开前置窗），否则 ZHUNBEI_END 技会晚于判定阶段 [P1] | 既有结构（非 A1 新增），但 A1 新增的准备阶段结束与 TurnBegin 窗使「晚到」的跨度变大 |
| K56 | BOSS_XUSHI 虚失·自翻面（出牌阶段结束面） | kernel 直写（无触发窗）：begin_discard 首臂发射 TurnOverBefore @kern/phase/discard.rs:15-26（注释自认『上游 phaseUseEnd turnOver 同口径；锚=弃牌阶段开始[原生无 PhaseEnd 帧]』） | liuli/mode/boss.js:4068｜{ player: ["phaseUseEnd","turnOverEnd"] } forced（turnOverEnd 面=翻回正面全场伤，原生在 commit_turn_over_before 尾内联，非本条范围） | PLAY_END | 弃牌阶段开始自翻面（相邻阶段近似；跳过弃牌阶段路径不翻面） | 是→改挂 Phase{Play,begin:false}（R05 已有该帧）；复核跳过出牌/跳过弃牌两条路径的行为变化并登记 [P2] | 无（具体阶段锚） |

### 5.4 DSL 装载器映射

| ID | 原生名（中文） | 原生 event＋阶段约束（file:line） | 上游 file:line｜trigger | 分类 | 原生现状 | 需改→改成 | 准备阶段影响/风险 |
|---|---|---|---|---|---|---|---|
| D01 | phase_begin | → EventKind::PhaseBegin + AtPhase(Judge)（注释：旧线=DomainEvent::TurnBegan=回合开始，每回合一次；首回合 Judge begin 帧不发出→首回合不触发[PA12 批②登记]） @loader/lib.rs:333-340 | 旧线 DomainEvent::TurnBegan→EventKind::PhaseBegin（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:841）；编译器 hover「fires once at the start of a turn, immediately before phase_prepare」（crates/content-compiler/src/lsp/hover_docs/skill.rs:226-228）＝上游 phaseBegin | TURN_BEGIN | 映射为「判定阶段开始」 | 是→EventKind::TurnBegin（去掉 AtPhase(Judge) 门）+ 补 trigger_ctx_of TurnBegin 臂（event_player）；首回合缺帧仍需登记或补发 [P1] | 61 个包 298 处使用：语义从「判定阶段开始」回到「回合开始」；准备阶段不再重复触发 |
| D02 | phase_prepare | → 拒装（BridgeError：原生无准备阶段帧[登记差异]） @loader/lib.rs:293-298（注释）,360-364（other=>拒装） | 旧线 TurnStateChanged{Prepare}→EventKind::PhasePrepare（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:867）＝上游 phaseZhunbeiBegin | ZHUNBEI_BEGIN | 拒装（73 个包 732 处不可用） | 是→Phase{Prepare,begin}（EventKind::PhaseBegin+AtPhase(Prepare)）——A1 已有真实准备阶段帧，可解除拒装 [P1] | 解除后 732 处 DSL 触发面可用；需补 AtPhase(Prepare) 与编译器/旧线 PhasePrepare 语义对齐验证 |
| D03 | phase_end | → EventKind::PhaseBegin + AtPhase(End)（=进入结束阶段！） @loader/lib.rs:301-304 | 旧线 TurnStateChanged{End}→EventKind::PhaseEnd（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:872，即「进入结束阶段」）＝上游 phaseJieshuBegin；不是上游 phaseEnd | JIESHU_BEGIN | 进入结束阶段 | 否→保持；但建议在文档明确 DSL phase_end≠上游 phaseEnd，上游整回合结束应新增 DSL 事件（如 turn_after） [P2] | 100 个包 2313 处：不受准备阶段影响（AtPhase(End) 门） |
| D04 | phase_X（judge/draw/play/discard） | → PhaseBegin+AtPhase(X) @loader/lib.rs:305-320 | 旧线 TurnStateChanged{Judge/Draw/Play/Discard}（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:868-871）＝上游 phaseJudgeBegin/phaseDrawBegin/phaseUseBegin/phaseDiscardBegin | JUDGE_BEGIN/DRAW_BEGIN/PLAY_BEGIN/DISCARD_BEGIN | 各阶段开始 | 否→保持 [P2] | 无 |
| D05 | phase_X_end（draw/play/discard） | → PhaseEnd+AtPhase(X) @loader/lib.rs:321-332 | 旧线 PhaseEnded{Draw/Play/Discard}（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:914-916）＝上游 phaseDrawEnd/phaseUseEnd/phaseDiscardEnd | DRAW_END/PLAY_END/DISCARD_END | 各阶段结束 | 否→保持 [P2] | 无 |
| D06 | turn_ended | → EventKind::TurnEnded @loader/lib.rs:357 | 旧线 PhaseEnded{End}→EventKind::TurnEnded（crates/engine-core/src/trigger/dispatch/ddd_shixing_female_available/event_contexts.rs:917）；LSP 补全「After a player's whole turn ends」（crates/content-compiler/src/lsp/completions/trigger_event.rs:114-117）＝上游 phaseEnd/phaseAfter 一族 | TURN_AFTER | 映射 EventKind::TurnEnded：触发窗在下一位角色 TurnBegin→准备→判定开始子树之后才派发；且 trigger_ctx_of 无 TurnEnded 臂（rvm_bridge.rs:194 default）→ event_player=None，依赖持有者=事件玩家的 VM 过滤恒假 | 是→映射到新建回合结束窗（phaseAfter 段；编译器补全文案为「After a player's whole turn ends」）并补 trigger_ctx_of 的 TurnEnded/TurnBegin 臂 [P1] | 与宿主 phaseEnd 族同源的系统性缺口 |
| D07 | phase_any_end/phase_draw_skipped 等旧线事件（phase_any_end 等） | 未映射 → BridgeError::InvalidField「VM 技能事件名 … 未映射（增量 3 严格面）」 @loader/lib.rs:360-364（other => 拒装） | 旧线 "phase_any_end"→EventKind::PhaseAnyEnd（crates/engine-core/src/content/registry/default_view_as_card_count.rs:620）；hover「fires after any concrete Prepare, Judge, Draw, Play, Discard, or End phase」 | PHASE_SPECIFIC_OTHER | 拒装 | 待定→可选：AtPhase 无门 PhaseEnd（任意阶段结束，含准备阶段）或单独 PhaseAnyEnd kind；22 个包 300 处 phase_any_end 目前仅旧线可用 [P1] | 若映射为无门 PhaseEnd，准备阶段结束会额外触发——定义时须决定是否排除准备阶段 |

## 6. A2 实施顺序建议（依赖关系）

1. **生命周期先行（P0/P1）**：K10（翻面跳过移到 TurnBegin 之前）、K25（回合统计/once_used 重置移到 TurnBegin 之前）、K22/L3（TurnBegin 窗期间 turn.phase、`trigger_ctx_of` 补臂、首回合）。这三项不先做，后面的迁移会让 once_per_turn 技在准备阶段被挡，翻面角色也会触发回合开始技。
2. **新建整回合结束窗（K21/L1）**，并决定 PhaseEnd 帧时序（K55/L2）。TURN_END（27 条）和 TURN_AFTER（8 条）的迁移、DSL `turn_ended`，以及两面条目 Kunlunjing、SwdLexue，都依赖这一步。窗口落地前，可以先把「任意阶段结束」过滤临时收窄为 `Phase{End,end}`，止住 A1 后每回合 6 次的多触发（K01 巫蛊、筑城、光塔、护天取回、灵舞这几项内核直推需要同步收窄）。
3. **TurnBegin 窗迁移（29 条＋两面条目 2 条）**：优先处理「任意阶段开始」类 P0（Mailun、SwdJikong、SwdJinlin2、SwdRexue、SwdFengyinState、SwdGouhunluoSub、YxsMenshen3、Kunlunjing 比对面），再处理钉在判定开始的 P1。注意联动点：`host/factory.rs:222/234`（Diyguhuo/Rexue 按 Phase 帧取目标）、`host/step/swd_damage_reactions.rs:195-204`（Huajin2 自清）、`host/step/skills_status.rs:324-328`（Kunlunjing 按 begin 位分两面）。
4. **Phase{Prepare,begin} 迁移（29 条）**：标准技 6 条、宿主 4 条，以及 judge.rs turn_opened 块中 19 个 ZHUNBEI_BEGIN Boss 技（落到 `commit_phase` 的 `Phase::Prepare => {}` 入场臂，phase.rs:118）。其中鬼计（K36）上游是 phaseJudgeBegin，留在判定开始；形态轮换（K30，phaseBefore）移到 TurnBegin 之前；封冻、巡游、炎狱（phaseBegin）改挂 TurnBegin 窗。
5. **DSL**：D01 `phase_begin` 改映射 TurnBegin；D02 `phase_prepare` 解除拒装（73 个包 732 处）；D06 `turn_ended` 接到新的回合结束窗。D07 `phase_any_end` 由后续决策处理。
6. golden 与锁面：`nc08_prepare_phase_mapping`、`nc08_upstream_identity_start` 两个登记差异锁面需要按新语义改写；golden 哈希变化先查原因，确认语义后再重录（AGENTS.md 第 4 条）。

## 7. 不确定／需决策项

- **WudianGuangshuSpade / K13 光塔**：上游 trigger 写 `phaseEnd`（整回合结束），同一技能的 intro 却写「下个结束阶段受到 1 点…」。按任务规则以 trigger 字段为准，记 TURN_END；如果按 intro 语义取 Phase{End,end}，需要在规则基准里登记这一取舍。
- **HlssFengqi**：host 键 `host:hlss_fengqi` 和包 `boss-zhugeliang-upstream-liuli` 对应 liuli `character/extra.js:456`，只有 `phaseZhunbeiBegin`。core `mode/boss.js:7585` 的同名 `fengqi` 是准备＋结束两值，不是本条来源。旧文档（第 609 轮）的「两值」结论来自后者。
- **D07 `phase_any_end` 等旧线事件**：22 个包 300 处只在旧线可用。是否给原生加映射、加的话是否把准备阶段算作「任意阶段」，需要先决策。
- **K24 phase_number 口径**：上游 `game.phaseNumber` 每回合 +1，原生每回合 +5（准备阶段已豁免）。影响 WudianZiyu 的 %4 门、潜龙/卫统的 ==1 判定和十殿 ≤50 判定。属于既有差异，A1 没有改变它，但需要决定是对齐上游还是登记差异。
- **阶段级近似保持项**（Before≈Begin、After≈End、DiscardEnd≈End 开始、Begin2≈Begin、roundStart/gameStart 近似）不受准备阶段影响，本次标为「保持」；它们仍属已登记的差异，不代表与上游规则等价。
