# T12 swd deferred 消解批 97：玄咒/因果

- 日期：2026-10-16
- 范围：三技条目消解（tag 404-405，玄咒双将共享）——**129/184 技条目已映射**（deferred 56→53）
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{skill.js,translate.js}`（xuanzhou/yinguo 逐段核验）

## 两技语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 玄咒 xuanzhou（陈辅/琴儿共享） | 出牌阶段限一次置出一张普通锦囊——视为任意延时锦囊置入任一角色判定区（无视使用限制，kind 玩家自选） | SwdXuanzhou(tag 404)=ActivateSkill usable:1：SelectCard 普通锦囊（排除机关异类）→TargetChoice（含自己，判定区无延时同款）→**确定性 kind 规则**（目标手牌≤1→兵、余→乐；玩家自选面登记差异）→实体发现延时牌置入判定区（CardsMoved 事件） |
| 因果 yinguo（康那里士） | 他人受到有来源伤害时——可弃一张牌将伤害来源和目标对调（unique） | SwdYinguo(tag 405)=DamageBefore optional priority 6：确认→弃 he 一张→**帧 source/target 字段对调**（redirect_chain 不动，上游仅换两字段同口径）→移除实例 |

## 登记差异

1. **玄咒 kind**：上游 chooseButton 三选（乐/兵/闪电）——原生无 vcard 按钮窗，确定性规则近似（上游 AI 序：目标手牌≤1 兵优先）；闪电自置无意义排除。
2. **玄咒置出近似**：素材锦囊弃置（上游 throw 置出展示）+ 实体发现延时牌（牌堆/弃堆首张同 kind）。
3. 判定区同名限制：乐/兵互不冲突（不同名），判定区已有任一延时者仍可再置另一名（上游同名限制近似）。

## 验证

- **定向回归** `swd_deferred_b97_xuanzhou_yinguo`：玄咒判定区 +1 延时+确定性 kind=乐（p1 手牌>1）+素材消耗+限一次反例；因果对调后原来源 p1 受伤（4→3）+原目标 p2 无伤+unique 实例移除。
- **全量**：engine-native 814 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0（陈旧布阵块清理后复跑绿）。

## 结论

- **129/184 映射，deferred 56→53**（玄咒双将共享消解 3 条目）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
