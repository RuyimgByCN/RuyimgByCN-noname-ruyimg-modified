# ADR 0101：区分主动交牌与主动拿牌

## 决策

牌移动原因增加通用的 `Give`，并由强制交牌结算通过 `GiveCards` 效果产生。类型化触发过滤器
`owner_received_gift_from_other` 仅匹配另一名玩家从个人区域主动交入技能拥有者手牌区的移动；普通
`MoveCards` 与由接收者选择的拿牌结算继续使用 `Unspecified`，不会误触发。

## 原因

旧 JavaScript 的 `gainAfter` 同时依赖 `event.source` 和 `event.bySelf` 区分“别人交给我”与“我从
别人处取得”。只根据来源区和目标区推断会把顺手获得、拆取后获得等行为错误视为赠予。把原因记录
在权威事件上可供后续所有“交给牌后”技能复用，也能稳定进入回放与触发队列哈希。

## 兼容性

新枚举与效果变体均只追加，保留已有序列化序号。该过滤器不增加 Rule VM 指令，使用它的包仍按
实际字节码声明 ABI；Rule IR 从 21 升至 22，Engine Semantics 从 96 升至 97。Native/WASM
跨目标一致性门禁必须重新通过。
