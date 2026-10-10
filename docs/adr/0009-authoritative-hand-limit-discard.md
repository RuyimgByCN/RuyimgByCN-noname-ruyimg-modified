# ADR 0009: Authoritative hand-limit discard

## Decision

- 弃牌阶段通过 `QueryRequest::HandLimit` 获取最终手牌上限，不再直接使用角色当前体力值。
- `sha_ignored_hand_limit` 是公共 modifier；非零时，手牌中的【杀】不计入弃牌阶段手牌数，也不会出现在阶段弃牌选项中。
- 命令校验、AI 和无头跑局共用 `discard_to_limit_tokens`，防止客户端直接提交本应受保护的【杀】。
- `ModifierTarget::ShaIgnoredHandLimit` 追加在枚举末尾，保留已有序列化变体编号。

## Compatibility

此前 `hand_limit` modifier 只影响查询结果，真实弃牌阶段仍按体力值计算，导致内容声明与对局行为不一致。
本决策修正该规则，并让 `decadexianzhen3`、`rexianzhen4`、`xinxianzhen3` 和
`new_jiangchi3` 复用公共 modifier 重写旧 JS 语义。标准包内容锁和完整对局路径因此改变；
引擎 Golden 与 conformance 向量在查询、弃牌选项、非法选择、AI 和无头跑局用例通过后同步更新。
