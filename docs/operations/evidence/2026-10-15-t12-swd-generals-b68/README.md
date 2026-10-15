# T12 swd 武将面批㉘：卢承轩/慕容诗/小雪

- 日期：2026-10-15
- 范围：三将 partial 转正——**37/79 名**（上批 34 将+3）；新增 HostBehavior tag 338-340
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（ljifeng/duanyi/yinyue 逐段核验）

## 三将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd卢承轩 (wu hp4, 男) | 疾风 ljifeng（降龙 deferred） | SwdLjifeng(tag 338)=marker 双臂：engine PlayCard max_targets 链 +X（X=已损失体力）+state computed_distance minus 链 -X（上游 attackFrom distance += hp-maxHp 同口径） |
| swd慕容诗 (shu hp3, 女) | 断义 duanyi（孤兴 group defer） | SwdDuanyi(tag 339)=ActivateSkill usable:1：Act 弃两张杀（机器显式弃）→SelectCard 消费→TargetChoice 选目标→1 伤+目标随机弃 X=其已损失体力张 he（RNG 代选，上游 randomGets 随机同面） |
| swd小雪 (wei hp3, 女) | 乐跃 yinyue（唤魂/待行 deferred） | SwdYinyue(tag 340)：Recover 事件 optional 确认门——回复者非自己且其手牌≥自己则各摸 1，否则回复者摸 1 |

## 设计要点

- **computed_distance 第二条查询链**：疾风距离 -X 进 state.rs computed_distance 的 minus 累计段（马术/凶兽/妖兽同段），`minus += ljifeng_minus`——与 max_targets 臂成对（连舞同型双臂）。
- **Recover 事件载荷字段=player 非 target**（filter/机器两处按载荷实名）。
- 断义代价弃牌=机器显式弃（phaseUse selectCard 2 非默认弃置语义在原生须机器自弃）。
- **无 TurnState/快照/哈希改动——golden 零扰动全绿。**

## 验证

- **定向回归** `swd_generals_b68_ljifeng_duanyi_yinyue`：疾风对座距离-2 入范围+双目标（1+X）+满血出界反例；断义两杀代价+1 伤+随机弃 X=2+杀不足反例；乐跃分支 1（回复者手牌≥自己→各摸 1，出桃自回净手牌不变断言）。
- **全量**：engine-native 780 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0（陈旧基线块清理后复跑绿）。

## 测试布阵勘误（入记忆）

- **断义类「selectCard 代价」**：上游 phaseUse 技选中牌默认弃置——原生 ActivateSkill 机器必须显式弃（机器自弃+SelectCard min 2 段）。
- **Recover 载荷 player**：非 target——事件载荷字段按实名。

## 结论

- **37/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
