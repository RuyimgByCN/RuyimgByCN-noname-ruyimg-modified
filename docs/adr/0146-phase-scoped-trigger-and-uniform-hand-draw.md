# ADR 0146：阶段限次弃牌触发与同色手牌公开摸牌

## 决策

旧版 JavaScript 只作为技能语义证据，不进入内容包，也不在运行时执行。新增声明式过滤器
`owner_discarded_has_hand_once_per_phase`：仅匹配技能拥有者从手牌区或装备区因语义化
`CardMoveReason::Discard` 失去牌、结算后仍有手牌、当前处于真实阶段且本阶段尚未接受过该触发的
`CardsMoved` 事件。它复用技能实例已有的阶段触发计数；拒绝可选触发不计数，接受后才占用本阶段
次数，每次真实阶段迁移统一清零。核心不为单个技能保存 blocker，也不识别技能名或武将。

`reveal_uniform_hand_draw_self_choose_others(player)` 公开展示拥有者当前全部手牌，并按有效牌色判断是否
全部严格为红色或全部严格为黑色。颜色不统一时仅展示；统一时拥有者固定摸一张，并可取消选择或选择
至多 X 名其他存活角色各摸一张，X 为展示时的手牌数。选择屏障保存拥有者、上限、来源技能、当前
角色和阶段；恢复时原子校验观察者、数量、去重、存活、排除自身、来源技能和阶段上下文。合法参与者
包含拥有者并按绝对座次顺序结算摸牌，不依赖客户端提交顺序。

## 兼容性

`EffectRequest`、`ContinuationKind`、触发过滤器、Rule IR 语句和 VM host effect tag 均只在末尾追加，
保留已有序列化序号。Rule VM ABI 升至 56，Rule IR 升至 67，Engine Semantics 升至 142。旧包不获得
新能力，仍按其原最低 ABI 加载。
