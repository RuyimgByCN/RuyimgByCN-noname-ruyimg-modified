# ADR 0759：N08 官方迁移第六十批——诸葛亮/陆逊

日期：2026-09-22（第六十批）
状态：已采纳
上游：[0758](0758-native-n08-ganglie-tuxi-luoyi.md)

## 决策

1. **观星（诸葛亮，可选触发，每回合限一次）**：回合开始（判定阶段占位，
   官方「准备阶段」的原生等位）观看牌库顶 X 张（X=存活数且至多 5），
   `guanxing_arrange` 询问（SelectCards，min 0、不可取消）中所选牌按应答
   顺序置顶（先选先摸），未选按原序置底。询问挂起可存档续行
   （`active_ask` 的 GuanXing/Place 态臂派生真实选项）。
2. **空城（诸葛亮，锁定门控）**：无 SkillDef 触发（NoopBehavior 注册，
   debug_grant/has_skill 通道），出牌校验侧——杀/决斗目标无手牌时
   `IllegalTargets`。
3. **谦逊（陆逊，锁定门控）**：乐不思蜀目标门控；上游目录无【顺手牵羊】，
   该半边挂账覆盖清单。
4. **连营（陆逊，可选触发）**：失去最后一张手牌后摸一张。触发覆盖
   出牌（CardUsed）/响应（CardResponded）/给出（CardsMoved）三条路径，
   filter 期判 `hand.len() ≤ 1`、机器步内复核空手；弃牌阶段批量弃置
   不产生事件暂不触发（挂账）。**新增目录能力**：`TriggerMeta.extra_events`
   ——单个 SkillDef 可分发到多种事件类别（本批唯一结构性扩展）。

## 验收

- `zhugeliang_guanxing_reorders_deck_top`：落位顺序（先选先摸/未选置底）、
  存档续行哈希一致。
- `zhugeliang_kongcheng_blocks_sha_and_duel_targets`：无手牌拒绝杀/决斗
  目标；有手牌放行并正常结算。
- `luxun_qianxun_blocks_le_target`、`luxun_lianying_draws_after_losing_last_card`
  （含非最后一张不触发）。
- 全量 **201 passed / 0 failed**；规则版本 21 → 22，14 golden 重录。

## 后果

标准包剩余：刘备、马超（下批收尾）。`extra_events` 为目录结构唯一
增量，历史 SkillDef 构造点已全部补默认空值。
