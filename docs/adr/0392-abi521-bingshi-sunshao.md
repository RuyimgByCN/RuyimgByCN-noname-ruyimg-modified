# ADR 0392: ABI 521 兵势孙韶（敢决/筑墼）

- 状态：已完成
- 日期：2026-08-02
- ABI：521

旧 JavaScript 仅作为语义证据。`敢决` 使用通用 view-as 契约把装备区实体牌当无距离、无普通次数消耗的杀；是否可响应按每个目标手牌中是否存在与来源牌相同的有效花色独立决定。

`resolve_mbzhuji` 依次完成花色选择、该花色全部可弃手牌的精确弃置、从共享摸牌堆再弃牌堆检索同花色装备并正常使用，以及满足弃牌数条件后的摸牌、回复或护甲选择。护甲复用公共标记，并在普通伤害提交处统一吸收；失去体力不受影响。

门禁：孙韶 Golden 4/4、兵势全量 Golden、content-compiler 469/469、Rule VM、content-runtime、engine-api ABI 协调、26 个扩展包导入与 conformance 8/8 全部通过。兵势包为 ABI 521、17 名武将、40 个技能，logic hash 为 `7192fea15ce926dababf33e1314aeb43420f3cba41f5c36ce61c1a42c40bfb01`，三份包文件 SHA-256 均为 `f897befccc7388a05f80e85d076e149077f147b838c9466306c656ae5c1b1e81`，导入后的 content lock 为 `4d2f0017a54ae5897b4a33de252e45afe1a15424adbb7a10ea99579f1ed8ebdb`。

本批按架构 §37.10 统一使用 `/private/tmp/noname-next-cargo-target`，门禁结束后清理该目录及工作区 `target/`，避免编译产物按批次累积。
