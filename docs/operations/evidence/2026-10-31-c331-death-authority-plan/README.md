# 共享 DeathFlow／provider 后续实施方案

核验2026-10-31，基于未变的Main d07／5fc死亡内核和固定两仓上游，以及只读C9冻结源码。实际VM KillPlayer先改HP／alive／dying／日志，后续旧DeathMachine又可能取消／替代／提交，存在两处authority冲突。

[方案原文](proposal.raw.txt)及[源码清单](source-inventory.json)提出复用既有栈的单一DeathFlow，保留正HP直接死亡、明确killer操作数和直接die与Damage／Tao救援的区别；真实provider／生命周期／operation LossEnd消费者必须实现，不用元数据壳替代。boss fellow removal不能自动视作death cue。

[独立复审](review.raw.txt)No issues／OK with notes：选定机制已在裁决29的DeathFlow、取消／复活、forceDie／forceOut、provider／lifetime范围内；裁决30的audio排除不会撤销29。不重复请求既有授权。实际出现未覆盖的opcode／操作数／无条件kill语义时仍逐项处理边界。

本目录只是已审实施计划，未实现或合入Main；当前Damage排序／恢复修复独立审查后再执行。331/106→332/106→R05 333/107及ABI759／WS7顺序不变；完整C331、I02或37来源不因此完成。当前状态见[进度主稿](../../../architecture/native-progress.md)。
