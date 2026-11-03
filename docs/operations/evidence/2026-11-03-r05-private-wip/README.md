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
