# ADR 0771：P2/N05、N06 复合状态验收扩展

> **2026-09-22 第 13 轮独立复核：本批局部交付保留，全部关闭结论不成立。**
> 当前状态以 [进度主稿](../architecture/native-progress.md) 与 [复现证据](../operations/evidence/2026-09-22-native-review-13/README.md) 为准。

新增测试实际通过，但其验收强度不足以关闭 N05/N06：流离原实例与恢复件从相同 hash 接受同一应答后分歧。
身份测试实际是同种子两次独立运行，未从中途恢复件继续；event_digests 只有日志长度与最后一条 Debug 字符数，非完整事件比较。
新选择测试主体覆盖观星/流离/反间，未覆盖注释列的全部六技能；当下 restore+hash 相等不保证续跑。
保留新增回归，补完整事件对照、真正中途恢复与剩余机器持牌死亡路径。

## 原批次记录（完成声明按上方复核更正）

日期：2026-09-22（第七十二批）
状态：已完成（N05/N06 行关闭；验收过程连带修复四处恢复缺陷）
上游：[0770](0770-native-n08-six-skills.md) / [0768](0768-native-p1-gap-closeout.md)

## 交付

1. **新选择点逐提交无重复守恒**（`p2_new_choice_points_per_submission_conservation`）：
   观星落位、流离转移、反间交牌、离间载体等新技能选择点，每次提交后
   执行「快照可恢复 + 恢复态哈希一致 + 牌归属多重集闭合」不变式。
2. **真实身份局终局逐提交事件对照**（`p2_identity_endgame_per_submission_parity`）：
   5 人身份局纯命令驱动至真实终局（无 debug 干预），逐提交状态哈希迹
   + 事件摘要迹；同种子两次独立推演逐哈希/逐事件一致、终态一致；
   胜负为合法阵营。
3. **死亡时剩余机器持牌路径**（`p2_death_with_machine_held_cards`）：
   闪电判定致死（判定机器持 trick/judge_card 于机）+ 濒死询问路径，
   死亡清区后逐提交恢复不变式成立。

## 验收过程连带修复（逐提交不变式抓到的真实缺陷）

1. **激活技挂起槽帧引用非法**：`ActivateSkill` 挂起槽硬编码
   `Slot::Intrinsic { frame: 0 }`（不存在的帧）——引入
   `resolution::NO_FRAME` 哨兵，恢复校验对哨兵豁免帧存在性检查，
   `finish_slot` 对哨兵短路。
2. **流离挂起快照不可恢复**：`active_ask` 缺 Sha 流离段臂——补
   `Sha { asked: false }` + 目标持流离的派生（选项=除来源/目标外的
   存活角色）。
3. **阶段→询问映射系统性错位**：Rende（TargetAsk↔rende_cards /
   TargetRead↔rende_target）、JieYin（同构）、FanJian（PickAsk↔
   fanjian_target / Consume↔fanjian_pick，选项=周瑜手牌）、
   LiJian（TargetsAsk↔lijian_discard / Launch↔lijian_targets）、
   Jijiang（RespondAsk↔jijiang_helper / TargetAsk↔jijiang_respond /
   Launch↔jijiang_target）——全部按「挂起时机器已推进到的阶段」
   重新对齐，并补齐缺失臂。
4. **急救对任意濒死者生效的语义确认**：急救红牌可救任意濒死角色
   （官方语义），驱动器据实应答。

## 验收

- 三个新用例全绿；全量 **226 passed / 0 failed**（native_kernel 143+3）。
- 规则版本 26 不变（验收用例与哨兵修复不改状态编码）。

## 影响

P2/N05、N06 行关闭。剩余 P2：N10 AI 基线与有界任务调度、
N09/N12 同负载性能与跨端证据（后续批次）；另轨平台项已剥离。
