# ADR 0025: 南蛮与普通锦囊选择器

- 状态：接受
- 日期：2026-07-14

## 决策

在通用 `CardSelector` 末尾增加 `nanman` 与 `ordinary_trick`。前者仅匹配【南蛮入侵】，后者匹配所有非延时锦囊牌；两者可复用现有卡牌目标、距离、次数等 modifier 通道。

神话包的 `huoshou1`、`juxiang1`、SP 包的 `olzhennan2` 使用 `nanman`，一将成名包的 `zhichi2` 组合 `sha` 与 `ordinary_trick`，不增加技能专用分支。

## 兼容性

新枚举项追加在既有选择器之后，不改变旧 npack 的序列化变体编号。规则可表达范围发生变化，`EngineSemanticsId` 从修订 8 升至 9；Native 与浏览器 WASM 哈希必须重新一致通过。
