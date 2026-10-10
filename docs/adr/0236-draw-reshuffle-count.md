# ADR 0236：牌堆重洗计数

## 决策

`GameState::draw_reshuffle_count` 记录本局弃牌堆实际洗入摸牌堆的次数；空弃牌堆调用不计数。
该字段进入规范状态哈希，零值省略以保持未重洗对局的既有哈希。

ABI 163 增加两个准备阶段过滤器：

- `owner_is_target_draw_reshuffle_zero_wounded`
- `owner_is_target_draw_reshuffle_positive`

它们复用现有 `recover` 与 `lose_hp`，不增加效果请求、HOST_EFFECT 或 continuation。
Engine semantics 升至 169；Native/WASM 发布前必须通过跨目标哈希门禁。
会发生重洗的 `identity_8_99` 固定向量重钉为
`02524409b17875bbb80a8b91f516a1d2d3a5900044bada6b3016d665c29456f5`；其余状态向量不变。
