# ADR 0197：只从牌堆获得最靠上的装备牌

## 背景

MOBILE 界朱桓「平寇」在按本回合跳过阶段数选择其他角色并依次造成伤害后，从共享牌堆顶端
方向取得第一张装备牌。旧 JavaScript 使用 `get.cardPile2`，没有装备时直接结束，不会查询
弃牌堆。

已有 `gain_card_by_kind(player, kind)` 接受具体牌种，并按牌堆、弃牌堆顺序查找。把“装备”
作为特殊牌种塞入该接口会同时改变它的参数含义和区域语义。

## 决策

新增 Rule VM ABI 124 通用效果：

```text
gain_equipment_from_draw_pile(player)
```

引擎只扫描共享 `DrawPile`，从顶端方向找到第一张 `kind.is_equip()` 的实体牌并移动到角色
手牌。没有匹配牌时为空操作；不访问弃牌堆，也不保存旧 JavaScript 牌堆对象。

「平寇」继续复用既有 `choose_other_targets_by_skipped_phase_count_damage`。VM 将装备获取排在
目标选择/伤害效果之后，效果队列保证所有选择、伤害及其嵌套事件完成后再执行获取。

## 版本与确定性

- Rule VM ABI：123 → 124
- Rule IR：85 → 86
- Engine Semantics：168 → 169
- HOST_EFFECT：139
- `EffectRequest` 哈希标签：176

原生端、WASM、联机握手和包含该效果的内容包必须同时声明 ABI 124。
