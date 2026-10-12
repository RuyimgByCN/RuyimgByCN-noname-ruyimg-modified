# 2026-10-11 T11 年兽系精觉片：精觉（继承胆术）（规则 v252）＋精觉年兽条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `24d97213`）。上游依据：`boss_jingjue`（8416：`inherit: "boss_danshu"` 同内容继承）；`boss_danshu`（8849：loseAfter+global loseAsync/gain/addJudge/equip/addToExpansionAfter——`getl(player).cards2` 非空、非当前回合、已受伤 → 判定红回 1）；携带者精觉年兽条目（`[boss_nianrui, boss_mengtai, boss_jingjue, boss_nbianshen]` hp12）。

## 建模

- **胆术+精觉**（双 SkillDef 查询型 Noop，共用 `commit_cards_discarded` 臂按名双查——ZhaoHuo 同构复用先例；胆术 def 供夜叉批条目复用）单臂：
  - **CardsDiscarded 面**（dispatch 新增臂 → commit_cards_discarded）：玩家失去手牌后（拆桥 from_hand/弃牌机器均发此事件）——非当前回合+已受伤 → 确定性判定（铁骑内联口径）红 → `ctx.recover(1)`。
- **精觉年兽条目**：`BOSS_GENERALS` 尾追加（def 索引 29，hp12）`[年瑞, 猛态, 精觉, 形态轮换]`。
- **登记差异**：上游覆盖 loseAsync/gain/addJudge/equip/addToExpansion 全卡牌移动事件族 → 原生仅 CardsDiscarded（拆手牌/弃牌阶段）触发，其余移动面（含 gain 摸牌后）登记；判定为铁骑内联口径无鬼才/天妒改判窗（frequent 面上游亦无询问）。

## 坑位实录

- **拆桥不造成伤害**——对照断言首版误写「3-1」期望（把弃牌当伤害），修正为 hp 不变+牌被拆走。
- **git checkout 回滚事故**：修卡名时误 `git checkout -- native_kernel.rs` 把整个测试文件未提交改动（含精觉测试）回滚——重加（卡名 = "dismantle"，name≠kind key 两套命名空间再次确认）。
- `debug_place` 前必清对应玩家初始手牌（v250 同坑）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_nianshou_jingjue_danshu_recover` | 四段：①拆手牌→非当前回合+已受伤判定红回 1（11→12）②满血对照（不超上限）③非持有者对照（拆桥无回血、牌被拆）④精觉实例存档往返+守恒 |
| 既有回归 | boss 全滤 48/48；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 251→252 全 14 位移；回退 251 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 252 全量重录 2/2 绿（行级程序化第三次，零手抄） |
| 全量 | **598 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 251→**252**；存档不变 v59（无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：夜叉批（胆术条目复用+魔道[准备摸 2]/魔剑[phaseUse 虚拟万箭]/魔眼+标准技 bazhen 混入核对）或 baonu 暴怒复杂面勘验。
