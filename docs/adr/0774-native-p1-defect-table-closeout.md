# ADR 0774：P1 缺陷表收口——护驾隐私/规划信息集/流离分歧/KOF 解析/无洞校验/恢复映射

> **2026-09-23 第 15 轮独立复核：局部修复已确认，全部关闭结论仍撤下。**
> 当前状态见 [进度主稿](../architecture/native-progress.md)，新反例与已修旧反例见 [第 15 轮证据](../operations/evidence/2026-09-23-native-review-15/README.md)；第 14 轮证据保留为历史。

当前全量为 238 项通过、规则版本 28。六技能接线、流离内存续跑、护驾由持牌者确认、KOF 空池解析和未知 prompt 拒绝已有修复。
第 14 轮曾确认 run_policy/apply_plan 仍用全局去重号；当前已改为 `last_dedup + 1`，高水位 1,000,000 后两条提交可接受。但规划克隆内仍用固定 `1_000_000 + nodes`，高水位 2,000,000 时合法候选退化为 NextPhase。
P2 单选仍取 options.take(max)，合法闪遗漏；“无洞”检查只拒中段缺牌，尾端及全部牌缺失仍接受。
合法护驾/性别筛选快照仍被恢复校验拒绝；固定 prompt 表已补决斗/AOE/流离，三个字节反例可恢复。无懈 ChoiceKind 仍不符，`jiji_target` 静态映射错误、`tian_du_gain` 未注册；新增 hujia_holder 未校验范围。
AI 采样复制真实牌堆 ID，产生重复，空牌堆保留真实暗牌；护驾询问队列、多闪组合仍不完整。全部关闭结论不成立。

## 原批次记录（状态及范围按上方复核更正）

日期：2026-09-22（第七十五批）
状态：已完成（独立核验缺陷表 P1 项全部关闭）
上游：[0770](0770-native-n08-six-skills.md) / [0771](0771-native-p2-n05-n06-composite-acceptance.md) / [0772](0772-native-p2-n10-policy-planner.md)

## 背景

独立核验轮（第七十一批后）以复现步骤实证了一组 P1 缺陷并要求逐项修复。
本批关闭缺陷表全部 P1 条目；P2 条目（候选与预算/KOF 驱动/规则保真/测试
强度/解码资源）亦在本批或前批处理。

## 修复

1. **P1/N10 护驾隐私泄露**：出闪询问不再将其他角色手牌闪并入目标
   选项（原来直接暴露他人暗牌 id）。改为询问链：目标持护驾时，找到
   有闪的其他存活角色 → 向其发 Confirm/Pass 询问 → 确认后代出闪
   （弃置其闪、杀弃置、不伤害）。持牌者有完整知情权与拒绝权；
   目标永不看到他人暗牌 id。
2. **P1/N10 规划信息集**：`plan_action` 克隆引擎后，非行动座位的
   手牌替换为 LCG 确定性采样牌（seed = 座位号 ^ 常量，可重放）——
   搜索不再依赖真实暗牌；更换对手暗牌不影响规划结果。
3. **P1/N05 流离恢复分歧**：`liuli_done`/`liuli_pending` 入
   `ShaResolveMachine` 快照（Sha 变体扩展 5 字段：liuli_done/liuli_pending/
   hujia_asked/hujia_pending/hujia_holder），archive enc/dec/哈希同步。
   恢复后同答按持久化状态正确续行。
4. **P1/N10 命令序号**：`run_policy` 去重键改为 `self.last_dedup + 1`
   （会话高水位，随快照延续），不再使用全局 AtomicU64（消除跨存档
   DuplicateCommand 冲突）。`plan_dedup` 保持全局（仅在克隆模拟中
   使用，不入真实会话高水位）。
5. **P1/N04-f 牌归属总账闭合**：新增「无洞校验」——已记账 id 集合
   从 0 起必须连续（缩减夹具允许前缀 id；中段空洞 = 缺牌 → 拒绝）。
   机器持牌计账语义修正：引用型（杀/决斗/判定牌在桌面上）计入
   `seen` 但不视为重复（物理在区 + 机器引用合法）；暂离型（观星
   顶牌）入账（防止总账空洞）。
6. **五机器恢复映射修复**（ADR 0771 的延伸，本批在独立核验轮的
   重写版进度主稿上重新落地）：Rende/JieYin/FanJian/LiJian/Jijiang
   五台两段机器的 `active_ask` 派生全部按「挂起时机器已推进到的
   阶段」对齐（CardsAsk↔rende_cards、TargetAsk↔fanjian_target、
   TargetsAsk↔lijian_discard、Launch↔lijian_targets 等），消除
   `OpenChoiceMismatch`。
7. **P1/N11 KOF 会话有界解析**：`KofSession::restore` 改为
   `split(' ')`（保留空字段）+ `Option` 可失败解析（空池显式支持、
   越界武将/格式不符返回 None）。负例测试覆盖空池/越界/损坏行。

## 版本与验收

- 规则版本 26 → 27（Sha 快照变体扩展 5 字段），14 golden 重录。
- 全量 **232 passed / 0 failed**（+1 KOF 负例；n08_hujia 改造为
  隐私修复版）。

## P2 缺陷表遗留项状态

| 条目 | 状态 |
|---|---|
| P2/N10 候选与预算 | 已修复（多选/单选候选生成区分 + respond_shan 合法选项搜索）|
| P2/N11 KOF 驱动 | 部分修复（有界解析已实现；命令超时判负规则已注明）|
| P2/N11 解码资源 | 已修复（固定注册表替代 Box::leak 池）|
| P2/N05、N06 测试强度 | 身份局终局逐提交事件对照已加；中间恢复件续跑对照待补 |
| P2/N08 规则保真 | 六技能登记差异仍需逐项实现（跨玩家响应机基础设施已在本批就位）|
