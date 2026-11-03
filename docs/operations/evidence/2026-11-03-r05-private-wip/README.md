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

## F 真包数据（2026-11-04，cf7c44d1）

| `cf7c44d1` F 真包数据 | xianding/sp2-upstream-noname 两包（tiqi/baoshu/dcbihuo 预编译模块 npack：DSL 壳→事件名改写→HOST_EFFECT_EXT(781/782/783) 体→logic_hash 重算；`r05f_real_packages` 5 测含 npack 再生回验）；映射定案：tiqi 只声明 `draw_phase_after`（主 draw>0 时 End 窗被 R 子帧替代、恒发射终窗=After；Skipped/Cancelled 留 F2） | 红 0/5（npack 移出）→绿 5/5；三 crate 1941/1/75（logs 见 /private/tmp/noname-r05/evidence/f-real-01.log＋f-real-report.md） |

四缺口：① tiqi phaseDrawSkipped/Cancelled 无原生触发载体（跳过路径不发窗）；② dcbihuo Current 槽无内核消费点（额定摸牌 ± 不生效）；③ dcbihuo 无 owner 角色门（旁观持有者也会被问）；④ 包 abi 按壳编译值 50/3（≤host 758，登记差异）。①-③ 由 F2 批闭。

## F2 批：缺口①-③闭合＋两文件拆行（2026-11-04，r05f2_semantic_gaps 7 测）

| 缺口 | 修法落点 | 核验 |
| --- | --- | --- |
| ② Current 消费（上游 sp2 sha `phaseDrawBegin2` `!numFixed → num+=storage`） | `kernel/draw.rs` DP::Begin2 增 `#r05:current` 消费循环（存在即消费含 0/负、pending 有符号、清槽＋helper；与 Combs 同门 `!num_fixed`）；num_fixed 守卫载体＝新 `Engine::debug_start_draw_phase_fixed` | 红 2（摸 2→1/3）＋守卫 1；见 /private/tmp/noname-r05/evidence/f2-report.md＋f2-01/02.log（红／全量同目录） |
| ① 跳过终窗（上游 tiqi global 含 phaseDrawSkipped） | `kernel/phase/draw.rs` 跳过路径 D 以 Before＋Skipped 入机；`kernel/draw.rs` DP::Before 预置终态只发 After（DrawPhaseAfter 恒发射，outcome 载荷区分）。**Cancelled 登记差异：原生无摸牌阶段取消生产者，面留待有生产者时接** | 红 1（兵粮寸断真实生产者路径，tiqi 无询问→两问＋Limit+2）；End/After 双落 DrawPhaseAfter 与 F 批定案一致 |
| ③ owner 角色门（上游 player:damageEnd＋source:damageSource） | `vm_effects/dcbihuo.rs` 凭据段：Source 面 owner==source、End 面 owner==victim（与 source 存在/存活/非自伤合取）；F/E 批 6 个既有 dcbihuo 测试改双持各答各面（End 取消路径仍覆盖） | 红 3（旁观被问→不问；受害者/来源各 [1,1]/[0,0]→[1]/[0]，±方向经摸牌数验证） |
| 行数门禁（machine_encode 1007／events 1070） | machine_encode.rs 拆 `machine_encode/{boss,skills}.rs`（机族分派臂保穷尽检查）；events.rs 拆 `events/turn.rs`（回合帧族，`pub(super) use` 兼容 `use events::*`） | 门禁 exit 0（9081 文件）；三 crate 1948/1/75（唯一失败仍既有 replay_scenarios golden，与 HEAD 同） |

测试：`crates/engine-native/tests/r05f2_semantic_gaps.rs`（登记 groups/misc.rs）7 测先红后绿；定向 r05 38/38。红／全量证据 `/private/tmp/noname-r05/evidence/f2-01.log`＋`f2-02.log`＋`f2-report.md`。

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
