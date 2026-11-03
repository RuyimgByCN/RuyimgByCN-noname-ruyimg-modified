# A2 波3：23 个 TURN_END_WINDOW 宿主技迁移（子代理实施＋主代理验收）

子代理按审计（audit.json change_group==TURN_END_WINDOW 的 23 条 H*）完成三处同构迁移；主代理复跑全量确认（[a2-wave3-verify.log]，三 crate **1916/1/75**，唯余 golden＝J 步重录）。

- metadata（common/swd/wudian/xianjian 四文件）：23 处 `event: EventKind::PhaseEnd → EventKind::TurnPhaseEnd`（上游 player/global phaseEnd＝整回合结束，每回合一次；原「任意阶段结束」在准备阶段加入后每回合 6 次）。
- filtering（common/swd/xianjian）：过滤臂 `EventData::Phase { begin: false, .. }` → `EventData::TurnEndWindow { player, after: false }`（保留各自 player 约束）；step 文件引用同步。
- factory：目标提取臂 7 处同步。
- 测试：17 个锁旧「每阶段结束」时点的测试文件按整回合语义更新（22 处 A2 Hxx 注释）——hujia/datong/huanhun/hwendao/jiying/lingxin/pingxu/hujing/longfan/longkui/mei/shenqishuang/tuobiang/zhucheng_restore/trigger_order/hutian_deposit/menshen。

迁移技能：九尾、门神、轩辕、护天、鲤鱼汤、糖花冻、大同、幻灵、东皇钟、沐风、灵心、灵锋、倾城、机关鸢、筑城、光塔♠、紫玉、炮台、急冻、冰心、灵火、忘忧、神舞。
