# R05 私有批工作证据（2026-11-03，进行中；分支 r05-private，未合入 main）

按裁决 28（v3＋v4＋v5 合同）／29／31 实施；目标版本 333／NGSA 107／ABI 759／WS8（升号在 J 步合入 main 时进行）。基线＝main `29712b2b`（332／106／758／WS7）。

## 检查点（r05-private 分支提交）

| 提交 | 内容 | 核验 |
| --- | --- | --- |
| `a605379c` A1 阶段基础设施 | `Phase::Prepare`（存档位 5／VM 索引 5，旧位不移）＋`EventKind/EventData::TurnBegin`（kind 42／data tag 26／hash 判别 27；九处接线）。回合更替改发 TurnBegin 窗（BeforeTriggers→Commit 压准备阶段 end/begin→Judge），每回合仍 5 次 NextPhase；phase_number 不计准备阶段 | `r05_turn_model` 3 测红→绿（[logs/a1-01-red.log] 先编译红；a1-03 绿）；三 crate 首跑 81 失败中 75 为私有树缺 `archive/upstream` 链接（环境性，a1-05/06 修复后）→ 真实影响 6 项 |
| `69f34575` B 骨架 | D/R/G 事件族（kind 43–51、hash 28–31）、`detached_cards` 状态区（暂不入档，106 布局逐字节不变）；**修 A1 暴露的既有缺陷**：rule-vm `check_caps` 双检查点（reference.rs＋vm.rs）不拒未知能力位——t8「拒收③」此前空转通过 | t8 38/38 绿（logs/b-03）；三 crate 回 1908/6/75 |
| `953213ab` B 主路径 | `begin_draw` 发 D 帧（跳过→D=Skipped；正常 Before→Begin（pending 有符号修正前窗，14 项同口径移植＋玉如意堆顶重排＋雪霁递减）→Begin1→Begin2→主 draw R 子树→End→After）；R 逐张入 held、双枯竭转 detached；G Filter→RecordHistory（D.actual_gain 折叠）→Transfer（held→手＋CardsDrawn 事件帧与日志双发）→After→Done | 三 crate **1909/5/75**（logs/b-10）：余 5＝热血/蛊舞（A2 审计）、golden（J 重录）、两个准备阶段登记差异锁面（A2 更新）。调试中修 3 个真缺陷：emit 误走 Commit 段（D 永不执行）、G 漏发 CardsDrawn 事件帧（mbqianlong +5 丢失）、玉如意/雪霁/白翼三处移植口径 |

## 已知待办（R05 内）

A2 阶段/回合触发审计（约 100 技能对照上游 phaseBegin/phaseZhunbei/phaseAfter）→ 热血/蛊舞迁移＋两锁面更新；C 191 caller 迁 `start_draw`；D Damage 凭据对照；E ABI759 三 compound＋四角色 helper；F 三技能包；G 存档 107（detached/D/R/G 入档）；H 投影 r05_effects/WS8；I 原字节锁链工厂；J 12 类场景＋golden 三步探针重录＋全门禁＋升号合入。

私有 target `/private/tmp/noname-r05/target`（裁决 23：批末退休）。

## A2 审计与下一步实施序（2026-11-03 14:55 更新，f81f1899）

审计两份独立交叉验证后定稿：168 条／需改 107／保持 60／待决策 1（REPORT.md＋audit.json 已入本目录）。**实施顺序（依赖序，先做生命周期 P0）**：

1. **K21**：新建「回合交接前」整回合结束窗（phaseEnd＋phaseAfter 两段；现有 TurnEnded 帧起步于 Commit 且 Commit 内先换人，触发窗晚于下一位整棵回合开始子树——不能复用）。落点 `kernel/events.rs::commit_turn_ended`（两处 `turn_opened = false` 前，825/798 行附近）。
2. **K25**：回合重置块（`kernel/phase/judge.rs:4-75` 的 `!turn_opened` 门：reset_turn_limits／cards_used_turn／turn_damage／忘忧／摄灵／射影／张目／红莲等）整体前移到发 TurnBegin **之前**——否则迁移到准备阶段的每回合限一次技能会被上回合标记挡住。
3. **K10**：翻面整回合跳过检测前移到发 TurnBegin 前（上游检测在 phaseBegin 前；A1 后翻面者会先跑完回合开始窗＋准备阶段才被跳过）。
4. **K22**：TurnBegin 窗补首回合发帧与 VM trigger_ctx_of 事件玩家臂。
5. 然后按审计分组迁移 107 条：TurnBegin 前 4／TurnBegin 窗 29＋两面 2／Phase{Prepare,begin} 29／整回合结束窗 phaseEnd 段 27＋phaseAfter 段 8／Play-end 精确锚 1（K56 虚失）／DSL 3（D01/D02/D06）。
6. 待拍板（不阻塞主体）：光塔 trigger/intro 矛盾按 trigger 记并登记；K24 phase_number 对齐或登记；D07 phase_any_end 是否含准备阶段。

三处 P0 与「任意阶段」过滤 6 次/回合问题修完后，热血（H42→TurnBegin 窗）与蛊舞（K01→phaseAfter 段）两个失败测试随迁移转绿。

## A2 生命周期 P0 实施状态（2026-11-03 18:50，K22 部分搁置）

| 项 | 状态 | 提交 |
| --- | --- | --- |
| K21 交接前整回合结束窗（phaseEnd/phaseAfter 两段，交接移至 phaseAfter 段 Commit） | ✅ 红→绿 | `3a0f98ee` |
| K25 回合重置前移 TurnBegin 前（含红莲效果与 reset 段；准备阶段 Boss 技段迁 `begin_prepare`） | ✅ 红→绿 | `3a0f98ee` |
| K10 翻面整回合跳过前移 turn_handoff 顶部 | ✅ 红→绿 | `2aef5772` |
| K22a `trigger_ctx_of` 补 TurnBegin/TurnEndWindow 事件玩家臂 | ✅ 保留（未提交，随下一波） | — |
| K22b 首回合发帧＋submit 前排空 | **搁置**：三方案（构造期排空／submit 前无界排空／有界排空）分别导致 d5 选将后建局、挂询问机死循环、或 **168 项测试红**（ai::redact 样本 stash 对照确认由该改动引起——首回合窗/阶段计数变化改变开局轨迹）。搁置=保留旧语义（首回合无 TurnBegin 序列），与 106 条迁移波次合并实施时一并处理其广泛测试影响。 | 证据 logs/a2-k22-01..06 | — |

三 crate 1912/5/75（余 5 已知：热血/蛊舞待 A2 迁移、golden 待 J、两准备阶段锁面待 A2 更新）。
