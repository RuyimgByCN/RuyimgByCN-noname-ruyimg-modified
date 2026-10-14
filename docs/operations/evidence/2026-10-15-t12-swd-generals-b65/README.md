# T12 swd 武将面批㉕：屠维/夏柔

- 日期：2026-10-15
- 范围：两整将转正——**27/79 名**（上批 25 将+2）；新增 HostBehavior tag 326-329
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（zhanlu/susheng/xianghui/huiqi 逐段核验）

## 两将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd屠维 (shu hp3, 女) | 湛卢 zhanlu+苏生 susheng | SwdZhanlu(tag 326)：ActivateSkill usable:1——SelectCard 弃黑桃（he）→TargetChoice 至多 3 名受伤角色→各回 1；SwdSusheng(tag 327)：**死亡前置窗口**（Death 帧 before 窗，Timing::First 同空洞印）optional+limits once_per_turn（OptionalGate 消费 once_used=每回合一次零字段）——SelectCard 弃手牌→目标 hp=1/maxHp≥1+death_cancelled 写回（DeathMachine 清 dying 旗） |
| swd夏柔 (shu hp3, 男) | 相惠 xianghui+挥琪 huiqi | SwdXianghui(tag 328)：ActivateSkill usable:1——弃红手牌→全体体力最少且已受伤角色各回 1（目标面自动求取）；SwdHuiqi(tag 329)：受伤后确认→TargetChoice 一名其他角色→摸 X（X=自己已损失体力，card 槽携带） |

## 设计要点

- **苏生零字段批**：每回合限一次走 `limits.once_per_turn`（OptionalGate 步内消费 once_used——批 23 通道），死亡取消复用 v171 基建（death_cancelled+dying 清旗）——**无 TurnState/快照/哈希改动，golden 零扰动全绿**。
- 湛卢 selectTarget [1,3] = SelectPlayers max:3；相惠 selectTarget -1（全体合资格）= 自动求取无选择窗。

## 验证

- **定向回归** `swd_generals_b65_zhanlu_susheng_xianghui_huiqi`：湛卢弃黑桃双目标回血（满血不可选+max 3+限一次反例）；相惠弃红牌双最少受伤回血（满血未动）；挥琪 hp2 受伤→p2 摸 X=3；苏生拒救援（dying_save 应 Pass）→死亡前置确认→弃桃→hp 回 1 未死+每回合限一次反例（第二次致死无窗）。
- **全量**：engine-native 777 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0（陈旧 sha5/h2 布阵清理后复跑绿）。

## 测试布阵勘误（入记忆）

- **dying_save 窗 min=1 不可空答**：救援窗选项=[Pass]+桃——空应答被拒（无限重问）；放行死亡应选 Pass（options[0]）。
- **出杀须回合属主**：测试第三次踩「非当前玩家 submit PlayCard 被 NotYourTurn 拒」——出杀前先推进到发起者回合。
- **挥琪 X 携带**：card 槽存 u32（X），TargetChoice 消费时读取。

## 结论

- **27/79 名**（upstream 79 将 183 技）；整将累计二十三（+2：屠维/夏柔）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
