# T12 swd 武将面批㉝：胡月/山小小/独孤宁珂

- 日期：2026-10-15
- 范围：三将 partial 转正——**52/79 名**（上批 49 将+3）；新增 HostBehavior tag 351-352；白名单 9→10
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（fengming/xiaomoyu/touxi 逐段核验）

## 三将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd胡月 (wu hp3, 女) | 鸣凤 fengming（hyunshen 计数存储 defer） | SwdFengming(tag 351)=ActivateSkill usable:1：SelectCard 弃装备牌→TargetChoice 任一存活角色→回 1（满血跳过）+摸 1 |
| swd山小小 (wu hp3, 女) | 小莫雨 xiaomoyu（shehun defer） | SwdXiaomoyu(tag 352)=Damage forced 锁定——回复 1（满血改摸 1）；每回合一次经 limits once_per_turn（resolution 层强制触发路径自动消费 once_used） |
| swd独孤宁珂 (qun hp3, 女) | 突袭 touxi（**builtin 白名单 9→10**——突袭在冻结键表；nlianji 拼点机 defer） | 零代码白名单转正 |

## 设计要点（勘误）

- **强制触发 once_per_turn 由 resolution 层自动消费**：plan→start 前置检查 `inst.once_used`（resolution.rs 485），机器臂**无需也无法**自行检查/置位（首版臂内自查 used=true 被框架预消费拦截实证）——与 optional 的 OptionalGate 消费、active 的 ActivateSkill 消费并列为第三条消费路径。
- 鸣凤目标含自己（上游 filterTarget: true 全体）。

## 验证

- **定向回归** `swd_generals_b73_fengming_xiaomoyu_touxi`：鸣凤弃装备→回 1+摸 1+限一次反例；小莫雨受伤分支回 1+每回合一次反例+满血分支摸 1（净不变断言）；突袭白名单授予断言。
- **全量**：engine-native 785 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **52/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
