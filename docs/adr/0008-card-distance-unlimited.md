# ADR 0008: Authoritative card-distance limits

## Decision

- 【杀】继续以攻击范围为距离上限；【顺手牵羊】和【兵粮寸断】的基础距离上限为 1。
- 命令校验、`TargetLegal` 查询、AI 合法动作生成和无头跑局共用同一距离规则。
- `card_distance_unlimited` 是使用者拥有的公共 modifier；非零时统一跳过上述卡牌距离限制。
- `ModifierTarget::CardDistanceUnlimited` 追加在枚举末尾，保留已有序列化变体编号。

## Compatibility

此前命令层没有校验【顺手牵羊】和【兵粮寸断】的基础距离，AI 也会生成超距动作。本决策修正
该规则，并让历史技能 `drlt_jueyan2`、`qingyi3` 通过公共 modifier 获得精确豁免。四人种子
42 的完整对局路径因此改变；引擎 Golden 与 conformance 向量在命令、查询、AI、无头跑局和
三类卡牌的距离 2 正反用例通过后同步更新。
