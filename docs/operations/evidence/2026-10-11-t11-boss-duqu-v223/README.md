# 2026-10-11 T11 相柳批末片：毒妒+蛇毒标记链+邪酬（规则 v223，存档 v50）——相柳四技全齐

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `c6235c95`；整批交付）。上游依据：`mode/boss.js` `boss_duqu(+boss_shedu)`（5666 区：受击者给来源 `storage.boss_shedu++`；shedu phaseBegin `chooseToDiscard(he, X) 或 loseHp(X)` 后 `storage--`）与 `boss_echou(+switch)`（5771 区：global useCard 桃/酒 → storage++；switch changeHp hp≤max/2）。

## 建模

- **蛇毒标记存储**：`PlayerState.shedu: u8` 新字段（跨回合持久标记）——**存档格式 49→50**（PlayerState 尾 u8；canonical_hash 同步入字段）；`debug_recover` 同款测试基建先行批已备。
- **毒妒**：`commit_damage` 尾查询臂——相柳受击（目标持毒妒）∧ 来源存活非己 → `source.shedu+=1`（自身不获）。
- **邪酬**两面：激活门控（`commit_damage` 尾第六并列臂：hp≤max/2 → enabled 直写；装配即未启用=apply 特判扩至五技）；主技（`commit_card_used`：激活态 ∧ 他人使用【桃】/【酒】（含 use_as 转化）→ 相柳 `shedu+=1`）。
- **蛇毒结算机**（`BOSS_SHEDU` 触发技，PhaseBegin forced——授毒时与中毒实例**双授**）：持有标记>0 → 挂起「弃 X 张牌或失去 X 体力」（SelectCards min0/maxX allow_cancel，**空选=失 X**；X=当前标记数）→ 消费后 `shedu-1`。新 `MachineSnapshot::Shedu { owner }` 变体：archive tag **64**/canonical_hash **65**/快照校验/**恢复臂重挂询问**（ShedU 为真挂起机——快照保留有义）/a0 样例尾+index 62。
- FROZEN 尾加「毒妒」「邪酬」「蛇毒」（79 名）。

## 登记差异

- 毒妒的「桃视为杀」mod 未建模（下批登记或随勾陈族）。
- 蛇毒结算以触发技+SelectCards 挂起（上游 chooseToDiscard 带确认分支——语义等价：空选=失 X）。
- shedu 标记无独立 LogEvent（SkillToggled 仅在授毒/激活时记——标记增减经状态可观测）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_xiangliu_duqu_shedu_echou` 四段 | ①书精伤害→目标获中毒+蛇毒双实例；②置顶方片判定伤 1+失去中毒、蛇毒保留；③再授毒蛇毒在；④独立局邪酬：hp≤半线伤害窗激活+他人用桃相柳获标记 |
| 既有回归 | 兽九/妖兽/梼杌/书精/四凶/关卡全部不变；native_kernel 224/0；a0 双守卫（63 变体） |
| 版本回退探针（223→222 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **570 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 222→**223**；存档 49→**50** |
