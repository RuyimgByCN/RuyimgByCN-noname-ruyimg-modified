# T12 swd 武将面批㉔：敷衍/荣霜/慕云/陈靖仇

- 日期：2026-10-15
- 范围：四将转正（两整将+两 partial）——**25/79 名**（上批 21 将+4）；新增 HostBehavior tag 321-325
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（lianda/suiyan/duanxing/polang/youyin 逐段核验）

## 四将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd敷衍 (qun hp4) | 连打 lianda | SwdLianda(tag 321)：杀结算后确认→SelectCard 弃一张（he 可取消）→虚拟杀对原目标再结算（use_virtual_card 直发）；TurnState.lianda_used=每回合旗（上游 lianda2 临时技等价=循环守卫） |
| swd荣霜 (wu hp3) | 隳延 suiyan+断行 duanxing | SwdSuiyan(tag 322)：伤害后确认→弃一张→弃对方全部装备（逐张 revoke）；SwdDuanxing(tag 323)：Equipped 事件确认→距离内选目标→虚拟杀 |
| swd慕云 (wei hp4) | 破浪 polang（竹海/击空 deferred） | SwdPolang(tag 324)：伤害后确认→SelectCards 选对方装备一张（公开信息自选面保留）→弃置 |
| swd陈靖仇 (wu hp3) | 诱引 youyin（移花 defer） | SwdYouyin(tag 325)：CardsDiscarded 双门（他人+含非基本牌+自己手牌<5）frequent 锁定式自动摸 1 |

## 字段批与验证

- **字段批**：TurnState +1（lianda_used）——存档 v75→v76、规则 v288→v289、golden 14 场重录；**版本回退探针 v288+去旗 → 14/14 与 HEAD golden 逐字节一致**。
- **定向回归** `swd_generals_b64_lianda_suiyan_duanxing_polang_youyin`：连打原杀+虚拟杀再结算+非杀无窗反例；隳延全装弃置（马匹槽布阵排除）；断行入装→虚拟杀（连打级联拒窗）；破浪自选装备弃置（三确认窗按触发序计数法：连打→隳延→破浪）；诱引弃牌阶段非基本牌→自动摸 1（min 补足应答）。
- **全量**：engine-native 776 passed / 0 failed。
- **Gate 1b**：exit 0（wuZhong snake-case 拦截修复后复跑绿）。

## 测试布阵勘误（入记忆）

- **多确认窗同形排队**：同一杀可同时触发连打/隳延/破浪（技能 def 序=触发序）——三窗全为 trigger_confirm 形状，计数法（拒 N-1 确认第 N）或首窗拒+次窗形状分流。
- **坐骑槽布阵陷阱**：马牌改距导致距离内目标 IllegalTargets——装备布阵排除 HorsePlus/HorseMinus。
- **断行级联**：虚拟杀自身 after 窗再触发连打（he=刚装的防具非空）——测试需拒绝级联窗。

## 结论

- **25/79 名**（upstream 79 将 183 技）；整将累计二十一（+2：敷衍/荣霜）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
