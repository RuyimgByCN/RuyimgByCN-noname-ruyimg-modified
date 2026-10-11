# 2026-10-11 T11 boss 内容第六批：书精催毒+中毒（规则 v220）——书精全技

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `354d107f`）。上游依据：`mode/boss.js` `boss_cuidui`（7272：source damageEnd——目标无 boss_zhongdu → addSkill+木神勾芒摸 1）与 `boss_zhongdu`（7293：phaseZhunbeiBegin 判定——黑桃保留/红桃无伤/其余伤 1+失去技能）。

## 建模

- `BOSS_GENERALS` 增**书精**（女/hp2/`[催毒]`——势力暂 Shu 近似上游 group "shen"（原生 Faction 无神族，登记）；`groupInGuozhan`/`isHiddenBoss` 面未建模）。
- **催毒**：`commit_damage` 尾查询臂——来源持催毒 ∧ 目标非己存活 ∧ 目标无中毒实例 → `skills.insert` 中毒实例（enabled，**运行时授技**=濒死救援通道 skills.insert 首例同模式）+SkillToggled；**登记**：木神勾芒联动（boss_mushengoumang 在场摸 1）跳过——木神未落目录；「敌方」以受伤者≠持技者近似（boss 模式阵营面未建模）。
- **中毒**（授技技能，表列 def）：`commit_phase` Judge begin 段内联判定（铁骑确定性判定先例）——**红桃**无伤/**黑桃**保留/**方片/梅花**伤 1（无来源）+实例移除。
- FROZEN 尾加「催毒」「中毒」（76 名）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_shujing_cuidui_poison_cycle` | ①伤害后目标获中毒实例；②置顶方片判定→伤 1+失去此技；③再授毒→黑桃判定→伤 1 且保留 |
| 既有回归 | 梼杌四技/饕餮/穷奇/混沌/烛阴/关卡全部不变；native_kernel 222/0 |
| 版本回退探针（220→219 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **567 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 219→**220**（存档不变 v49） |
