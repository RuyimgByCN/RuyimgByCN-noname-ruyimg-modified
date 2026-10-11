# 2026-10-11 T11 毕方批首片：招祸两面（规则 v227）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `7d50ab2c`；毕方批按指令最小切片——仅招祸两面，红莲/炎狱(+switch) 登记下批）。上游依据：`mode/boss.js` `boss_zhaohuo`（4946：damageBegin4 damageBegin1 双触发——受火伤 cancel + 造成伤害 setNature(fire)）与 `boss_bifang`（889 区：hp25/五技）。

## 建模

- `BOSS_GENERALS` 增**毕方**（群/hp25/`[妖兽, 招祸]`，表序插入于梼杌后——**书精/相柳/朱厌 def 索引整体后移一位，六处测试索引同步重排**）——**登记**：红莲（红色不计上限+回合开始随机获红牌/随机对 3-0 名各 1 火伤）/炎狱(+switch)（他人回合判定红=1 火伤，每回合至多 3 次）未落随下批。
- **招祸**两面：①**造成伤害视为火**——`ZhaoHuoBehavior/Machine`（DamageBefore 触发机，相德同款窗口一步直改 `nature=Fire`）；②**免疫火伤**——`commit_damage` 入口直读（诚信免伤同位：目标持技 ∧ nature=Fire → DamagePrevented return）。**登记差异**：你的回合中其他角色防具无效面挂点分散（八卦/仁王盾/藤甲等各自校验），未建模。
- 新 `MachineSnapshot::BossZhaohuo { owner }` 变体：archive tag **65**/canonical_hash **66**/快照校验/恢复臂 None/a0 样例尾+variant_index 62。
- FROZEN 尾加「招祸」（80 名）。

## 连带修正

- a0 变体表发现 **Shedu 与 BossZhaohuo 撞号**（v223 批 Shedu=62 与本批初设 62 冲突）：Shedu 重排 63、BossZhaohuo=62→对齐 archive tag 顺序；样例表 Shedu 重复行去重。
- 书精/相柳/朱厌测试 def 索引重排（+1 修正四处）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_bifang_zhaohuo_fire_nature_and_immune` | 毕方普杀正常 1 伤（nature=Fire 不加值）；毕方免疫 2 点火伤（hp 不变）；普通伤害正常结算（对照） |
| a0 守卫 | 63 变体穷尽+tag 唯一/往返 |
| 既有回归 | 虚失/巨源/兵先/相柳/梼杌/书精/四凶/关卡全部不变；native_kernel 229/0 |
| 14 golden 重录 | 224 版+当前代码全位移佐证（探针口径登记）后 2/2 绿 |
| 全量 | **574 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 224→**227**（断链批合并录 v225/v226/v227）；存档不变 v51 |
