# ADR 0060：实体牌触发过滤与牌顶交换续体

- 状态：Accepted
- Engine semantics：44

## 决策

新增 `owner_source_physical_ordinary_trick` 触发过滤器。view-as 执行期间由核心记录当前转化的实体牌，过滤器据此保留旧规则 `event.card.isCard` 的“非转化普通锦囊”语义，不在内容层识别技能 ID。

新增 `reveal_top_gain_non_basic_or_swap_basic(player)` 类型化效果：公开亮出牌堆顶牌；非基本牌进入玩家手牌；基本牌打开可取消选择，取消时弃置亮出的牌，选择时先把一张手牌置于牌堆顶，再获得亮出的牌。选择及续体只保存稳定 ID，并在恢复时重新验证区域归属。

## 结果

`junkjizhi` 可由 DSL 精确表达；牌堆仍按 bottom→top 存储，交换牌通过普通 `MoveCards` 进入牌堆尾部，因此原生与 WASM 的顺序及哈希一致。
