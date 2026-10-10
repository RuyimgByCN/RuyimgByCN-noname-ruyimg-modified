# ADR 0090: 精确手牌调整进入通用权威效果

## 决策

新增 DSL 效果 `adjust_hand_to(player, count)`。目标角色手牌少于指定数量时摸至该数量；多于
指定数量时，权威引擎冻结差值并打开不可取消的精确手牌选择，选择提交时重新校验数量、去重和
当前手牌归属。它不复用 `discard_hand` 的确定性前 N 张语义，避免丢失旧技能中由玩家决定弃牌的
规则选择。

同时追加 `owner_is_target_turn_damage`，与既有 `owner_is_target_no_turn_damage` 共享同一回合
伤害事实，分别表达事件角色在自己的当前回合已经造成和没有造成伤害。TW「典仪」据此完整重写：
造成过伤害则弃置所有手牌，否则把手牌精确调整至四张。旧 JavaScript 仅保留为语义证据。

新宿主效果使用追加的 effect tag 41，Rule VM ABI 从 8 提升至 9；ABI 9 主机继续加载 ABI 3–8
内容，只有实际使用 `adjust_hand_to` 的包声明最低 ABI 9。内容 Schema 保持 3，Rule IR 从 16
提升至 17，Engine Semantics 从 74 提升至 75。Native/WASM 与 conformance 哈希门禁必须重跑。
