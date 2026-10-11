# 2026-10-11 T11 boss 内容第四批续片：饕餮藏目+积财（规则 v216）——饕餮四技全齐

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `0c91778c`；整批交付）。上游依据：`mode/boss.js` `boss_cangmu`（5892：phaseDrawBegin `trigger.num += countPlayer()-2`）与 `boss_jicai(+_switch)`（5899/5566：global recoverAfter——自己回复 draw 2、他人回复 `asyncDraw([player, source])`；switch changeHp hp≤max/2 enableSkill）。

## 建模

- **藏目**：权限型 Noop+Draw 臂查询（无脏同位并列）——固定摸 2 后补额外 `存活数-2`（单 boss 场景两摸牌技不并存，臂独立无互斥问题）。
- **积财**：权限型 Noop+两面查询——①激活门控（`commit_damage` 尾，延妒并列第四臂；装配即未启用=apply 特判扩隐贼/延妒/积财）；②回复后双摸（**新 `commit_recover_skills` 臂**：`commit_frame` match 增 `EventData::Recover` 分派——机制层 `Ctx.recover` 内联扣血+emit、技能面归 kernel commit 层的分层既定；仅实际回复量>0 的事件进入；回复者==持有者 → 摸 2，否则持有者与回复者各摸 1）。
- 测试面新增 `debug_recover(player, amount)`（发 Recover 事件走完整结算，`debug_deal_damage` 同款模板）。
- FROZEN 尾加「藏目」「积财」（71 名）。

## 登记差异

- `boss_jicai_switch` 独立 switch 技不落表（门控并入主技 SkillInstance.enabled，隐贼/延妒先例同构）。
- 积财「回复后」以 Recover 事件提交臂承载（上游 recoverAfter 同窗口）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_taotie_cangmu_jicai` | 藏目 4 人局摸牌基数 4；积财装配未激活 → hp 置半线一次伤害激活 → 他人回复（饕餮+回复者各摸 1）→ 自己回复（摸 2） |
| 既有回归 | 贪欲/穷奇五技位/混沌/烛阴/关卡全部不变；native_kernel 218/0 |
| 版本回退探针（216→215 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **563 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 215→**216**（存档不变 v48） |

## boss 内容面进度

十批已落（v208–v216）：烛阴、混沌（四技全齐）、穷奇（五技位全齐）、**饕餮（四技全齐）**。下批=梼杌（boss_luanchang(+switch) 交替回合序+其余技）或勾陈。
