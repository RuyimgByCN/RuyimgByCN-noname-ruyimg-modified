# ADR 0278：SIXIANG 罪辞与固定目标场上牌移动

ABI 202 增加过滤器 `owner_is_target_source_movable_field_card` 和效果
`choose_move_field_card_to(player, target)`。权威查询统一计算可移动候选：牌必须位于固定目标
以外存活角色的装备区或判定区；装备牌只能进入已启用且空置的同类槽位，延时锦囊不能与固定
目标判定区内同类牌重复。

「罪辞」在拥有者受到有存活来源的伤害后提供可选发动窗口，确认后由拥有者选择一张候选并移入
来源的对应区域。触发过滤、选项生成和续体恢复共同复用同一查询；续体还重新验证技能实例、
选择者和实体牌位置。旧 JavaScript 的 `canMoveCard`、`moveCard`、提示、AI 与动画不进入
npack 或运行时。
