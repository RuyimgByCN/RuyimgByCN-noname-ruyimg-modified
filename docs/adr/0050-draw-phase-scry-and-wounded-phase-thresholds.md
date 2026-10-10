# ADR 0050：摸牌阶段牌堆整理与受伤人数阶段阈值

- 状态：已接受
- Engine semantics：34

## 背景

旧版「恂恂」允许在摸牌阶段观看牌堆顶四张牌，自由决定两张牌的牌堆顶顺序和另外两张牌的牌堆底顺序。「兴棹」还会按存活受伤角色数动态启用该能力、装备区进出摸牌、跳过判定/弃牌阶段和伤害加成。现有准备阶段牌堆整理只支持把选中牌沉底并保留相对顺序，不能精确表达这组语义。

## 决策

- 增加参数化查询值源 `alive_wounded_count_at_least_N`，每次查询时统计存活且 `hp < max_hp` 的角色；
- 增加 `draw_phase_scry_four_split_two` modifier 和稳定 continuation 23：玩家可取消；发动后必须排列全部四张牌，前两张按摸牌顺序留在牌堆顶，后两张按牌堆底到顶顺序置底；随后继续统一摸牌流程；
- 增加 `skip_discard_phase` modifier，并令既有 `skip_judge_phase` 使用查询期状态值；
- 增加通用 `owner_equip_zone_changed_alive_wounded_at_least_two` 的 `CardsMoved` 过滤器。装备替换产生一条离开和一条进入事件，因此自然摸两张牌；
- 普通「恂恂」与「兴棹」提供的同类能力合并到同一 modifier 查询，只打开一次选择，不运行旧 JS，也不按技能名分支。

## 结果

`xunxun`、`xz_xunxun`、`xinfu_xingzhao2` 和完整 `xinfu_xingzhao` 可由 DSL 精确重写。新增枚举项全部追加在末尾，保留既有 npack 的序列化序号；规则表达范围扩大，`EngineSemanticsId` 从 33 升至 34，并重新执行 Native/WASM 一致性门禁。
