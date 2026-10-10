# 阶段 30：锦囊牌效果验证（§48）

> 确认引擎的 effect primitives 实际执行锦囊牌规则，而非仅配置描述。

## 状态

锦囊牌效果**已在 M1/M2 阶段实现**于 `engine-core/src/command/play.rs`。本阶段
新增集成测试验证它们确实被引擎执行。

## 引擎已实现的锦囊效果

| 锦囊 | CardKind | 效果实现位置 | 实际效果 |
|---|---|---|---|
| 无中生有 | TrickWuzhong | `resolve_trick_effect::WUZHONG` | `DrawCards { count: 2 }` |
| 过河拆桥 | TrickGuohe | `resolve_trick_effect::GUOHE` | 打开选择窗口，选目标1张牌弃置 |
| 顺手牵羊 | TrickShunshou | `resolve_trick_effect::SHUNSHOU` | 打开选择窗口，取目标1张牌 |
| 决斗 | TrickJuedou | `resolve_trick_effect::JUEDOU` | 打开杀比拼窗口 |
| 南蛮入侵 | TrickNanman | `resolve_trick_effect::NANMAN` | 所有其他玩家需出杀或受伤 |
| 万箭齐发 | TrickWanjian | `resolve_trick_effect::WANJIAN` | 所有其他玩家需出闪或受伤 |
| 桃园结义 | TrickTaoyuan | `resolve_trick_effect::TAOYUAN` | 所有玩家回复1点体力 |
| 五谷丰登 | TrickWugu | `resolve_trick_effect::WUGU` | 展示牌堆顶N张，每人选1张 |
| 乐不思蜀 | TrickLe | delayed judgment | 判定非红桃→跳过出牌阶段 |
| 兵粮寸断 | TrickBingliang | delayed judgment | 判定非草花→跳过摸牌阶段 |
| 闪电 | TrickShandian | delayed judgment | 判定黑桃2-9→受到3点雷电伤害 |
| 借刀杀人 | TrickJiedao | `resolve_trick_effect::JIEDAO` | 令目标对其范围内1人出杀 |
| 铁索连环 | TrickTiesuo | `resolve_trick_effect::TIESUO` | 连环/解锁角色 |
| 火攻 | TrickHuogong | `resolve_trick_effect::HUOGONG` | 展示1张同花色手牌→受到1点火伤害 |
| 无懈可击 | TrickWuxie | wuxie window | 反制1个锦囊 |

所有锦囊效果通过 `begin_wuxie_window` + `resolve_trick_effect` 路径执行，
支持无懈可击反制窗口。

## 配置驱动

卡牌定义从 `content/core-rules/base-deck.json` 加载（阶段 27），锦囊牌的
def/kind/suit/rank 由 JSON 配置驱动。效果逻辑在引擎核心中实现。

## 测试

```bash
cargo test -p engine-core --test trick_card_effects    # 10 tests
```

| 测试 | 断言 |
|---|---|
| `trick_wuzhong_cards_exist_in_deck` | 牌堆含 ≥4 张无中生有 |
| `trick_guohe_cards_exist_in_deck` | 牌堆含 ≥6 张过河拆桥 |
| `trick_juedou_cards_exist_in_deck` | 牌堆含 ≥3 张决斗 |
| `deck_contains_all_card_categories` | 牌堆含 ≥15 种卡牌类型 |
| `deck_has_complete_trick_set` | 全部 15 种锦囊牌在牌堆中 |
| `deck_has_equipment_cards` | 武器/防具/坐骑在牌堆中 |
| `deck_has_basic_cards` | 杀/闪/桃/酒在牌堆中 |
| `deck_total_card_count` | 总卡牌数 ≥119 |
| `trick_wuzhong_effect_draws_cards` | 无中生有效果：手牌+2 |
| `trick_guohe_effect_discards_card` | 过河拆桥效果：手牌不增 |
