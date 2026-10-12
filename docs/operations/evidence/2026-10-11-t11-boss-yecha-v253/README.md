# 2026-10-11 T11 夜叉批：魔道+魔剑+胆术复用（规则 v253）＋夜叉条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续批次链；精觉 v252 后夜叉批）。上游依据：`boss_yecha`（1231：shen/hp11/skills `[boss_modao, boss_mojian, bazhen, boss_danshu]`）；`boss_modao`（8882：phaseZhunbeiBegin draw(2)）；`boss_mojian`（8890：phaseUseBegin useCard 虚拟万箭打全部异侧存活[canUse 过滤同口径]）。**勘误**：魔眼（boss_moyan）属罗刹（boss_luocha）不在夜叉表——夜叉批无魔眼。

## 建模

- **魔道**（查询型 Noop，`BOSS_MODAO`）：Judge begin 臂自摸 2（藏目/年瑞同位）。
- **魔剑**（查询型 Noop，`BOSS_MOJIAN`）：Play begin 臂——异侧存活全目标 `use_virtual_card(holder, ArcheryAttack, targets)`（乱常 End 臂万箭同款虚拟牌机器直发；空手敌方不询问直接受伤=AOE 跳过语义挂账清单口径）。
- **胆术复用**：夜叉条目含 v252 胆术 def（CardsDiscarded 臂零改动）。
- **夜叉条目**：`BOSS_GENERALS` 尾追加（def 索引 30，hp11）`[魔道, 魔剑, 胆术]`。
- **bazhen 混入核对**：八阵=标准装备八卦阵（native Armor 判定机已承载，v208–v252 全量装备面回归覆盖）——由装备承载、不入条目技能表，登记。

## 登记差异

- 魔剑上游 useCard 走 canUse 过滤（含距离/锦囊面）——native 虚拟牌机器直发天然不经 PlayCard 校验（乱常先例同口径），异侧存活过滤保留。
- bazhen 标准装备承载（见上）。
- 势力 shen→群目录近似（先例族）。
- 魔眼属罗刹批（勘误）。

## 坑位实录

- **万箭打空手敌方不询问直接受伤**（AOE 跳过语义挂账清单口径）——首版测试期待 aoe_response 询问挂起，实际全部直接结算；断言改为直接血量校验。
- 时序：魔剑在 Play begin 触发——advance_to_play 之后才清手会错过触发窗口；改为 advance 前清全场手牌。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_yecha_modao_mojian_danshu` | 四段：①魔道（准备摸 2：手牌 4=魔道 2+摸牌 2）②魔剑（敌方 0/2/3 各 4→3、本体不受、全无异常）③胆术复用（掉血后拆手牌判定红回 1：10→11）④双技实例存档往返+守恒 |
| 既有回归 | boss 全滤 49/49；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 252→253 全 14 位移；回退 252 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 253 全量重录 2/2 绿（行级程序化第四次） |
| 全量 | **599 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 252→**253**；存档不变 v59（无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：罗刹批（魔道共用/boss_yushou 御兽[虚拟南蛮]/魔眼/标准技 yizhong 混入）或牛头马面批（爆敛/蛮甲/枭首+niepan 混入）或 baonu 暴怒复杂面勘验。
