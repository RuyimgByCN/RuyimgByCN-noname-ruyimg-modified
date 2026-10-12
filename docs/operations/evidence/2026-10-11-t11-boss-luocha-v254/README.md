# 2026-10-11 T11 阴间系罗刹批：御兽+魔炎（规则 v254）＋罗刹条目（魔道共用）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `1e40ca98`）。上游依据：`boss_luocha`（1226：female/shen/hp12/skills `[boss_modao, boss_yushou, yizhong, boss_moyany]`）；`boss_yushou`（御兽：phaseUseBegin 虚拟南蛮打全部异侧存活——魔剑同款）；`boss_moyany`（魔炎：loseEnd 非当前回合 → 判定红 → 自选一名其他角色 2 火伤）。

## 建模

- **御兽**（查询型 Noop，`BOSS_YUSHOU`）：Play begin 臂——异侧存活全目标 `use_virtual_card(holder, SavageAttack, targets)`（魔剑同款虚拟南蛮机器直发；CardKind=SavageAttack）。
- **魔炎**（查询型 Noop，`BOSS_MOYANY`）：commit_cards_discarded 追加段（胆术/精觉臂后）——非当前回合持有者失去牌 → 判定红 → 座位序首个其他存活角色受 2 点火焰伤害（目标自选→确定性近似登记）。
- **罗刹条目**：def 索引 31/female/hp12/`[魔道, 御兽, 魔炎]`——魔道共用 v253。
- **yizhong 混入核对**：yizhong=**一将成名包**防御技（防具栏空时黑杀无效，yijiang/skill.js:13029）——非标准包、非 boss 技，未落登记（随一将包批次）。

## 登记差异

- 魔炎目标自选 → 座位序首个其他存活确定性近似（先例族）。
- 魔炎上游 loseEnd 全牌损失面 → native CardsDiscarded 面（与胆术同口径登记）。
- 势力 shen→群目录近似（先例族）。

## 坑位实录

- 南蛮 CardKind=**SavageAttack**（非 Nanman）。
- 御兽/魔炎 def 首插非表尾（ freezing 表拦截）——移真表尾。
- 回归断言两处口径修正：段①南蛮伤害持续（hp3=3 非 4）；拆桥不伤害（无回血=hp 不变而非 -1）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_luocha_yushou_moyany` | 五段：①御兽虚拟南蛮（敌方空手各受 1、本体不受）②魔炎判定红→0 号受 2 火伤（4→2）③判定黑对照（无火伤）④非持有者失去牌对照（无魔炎）⑤双技实例存档往返+守恒 |
| 既有回归 | boss 全滤 50/50；机械审计：无多重注册/零属性清零；冻结表 10/10（def 移尾后） |
| 版本递增探针 | 253→254 全 14 位移；回退 253 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 254 全量重录 2/2 绿（def 移尾后复跑仍绿） |
| 全量 | **600 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 253→**254**；存档不变 v59（无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：bianshen4 阴间系勘验落包（黑白无常[爆敛/强征/醉酒]/牛头马面[蛮甲/枭首]+标准技混入核对 juece/niepan）或年兽收尾（baonu 暴怒复杂面批）。
