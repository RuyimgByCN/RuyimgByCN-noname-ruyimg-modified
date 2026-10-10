# ADR 0100：按有效判定花色回复

## 决策

新增类型化效果 `judge_recover_on_suit(player, suit, amount)`。引擎复用统一判定翻牌路径，按被
判定角色计算有效花色，将判定牌移入弃牌堆；命中指定实体花色时，以技能拥有者为来源回复体力。
旧 JavaScript 只保留为语义证据，不进入运行时。

## 兼容性

效果 tag 43、`StmtIr` 与 `EffectRequest` 变体均追加在末尾，保留既有序列化序号。Rule IR 从
20 升至 21，Rule VM ABI 从 11 升至 12，只有实际使用该效果的包声明 ABI 12；ABI 3–11 包仍可
加载。新结算语义令 `EngineSemanticsId` 从修订 95 升至 96，并由 Native/WASM 一致性门禁验收。
