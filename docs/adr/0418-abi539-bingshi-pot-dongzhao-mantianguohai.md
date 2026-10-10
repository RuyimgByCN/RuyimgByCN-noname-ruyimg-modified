# ADR 0418：ABI 539 兵势董昭与「瞒天过海」

本批把旧 JavaScript 中势董昭（`pot_dongzhao`）的「妙略」「迎驾」及衍生实体锦囊
「瞒天过海」重写为 DSL/Rust；旧 JavaScript 只作语义证据，不进入 npack 或运行时。

## 决策

- ABI 539 增加 `resolve_spmiaolve` 与 `resolve_spyingjia`。妙略使用规则 RNG 创建两张
  花色互异且全局不重复的实体「瞒天过海」，受伤后可摸二或从牌堆/弃牌堆获得一张智囊。
- 迎驾在技能状态中记录本回合逻辑锦囊名；同名锦囊至少使用两次后，弃一张手牌并把
  一次完整额外回合压入既有队列。包内辅助技能在对应额外回合开始摸二，然后自删。
- 「瞒天过海」复用逐目标无懈队列：每个成功目标由使用者获得一张 HEJ 牌，全部结算后
  按座次依次回赠一张 HE 牌；实体牌内建不计手牌上限和弃牌阶段不可弃置语义。
- 包继续使用现有 `*.skill + characters.json + cards.json -> npack -> extensions/bingshi`
  加载形式，不增加 JavaScript 执行或专用扩展加载层。

## 验证

- 董昭 Golden 4/4；兵势 22 个测试二进制共 72 项全部通过。
- content-compiler 476/476、content-schema 4/4、engine-api 4/4、rule-vm 10+5、
  conformance 3/3 与 content-runtime 冷导入测试全部通过。
- 完整 `extensions/` 冷导入激活 26 个包，ContentLock
  `8d5829990032f2c98a6930d8be55caf112b89a8077e2047061f630cbcdd07310`。
- Bingshi 包为 ABI 539、23 名武将、54 项技能、1 项牌定义；logic hash
  `9cb2f6eb5e595b6dd5cb623dc1ff6d031ccf0e0a7d029149605ff7fbedaafb62`，SHA-256
  `15702d6427c822453ef0d6edc76a05613bcadb86dd1c0f021da3daf0d1d572d4`；三份包字节一致。
- 编译产物按架构计划 §37.10 处理：批次门禁结束后删除工作区 `target/` 与本批外部
  `CARGO_TARGET_DIR`，只保留源码、文档、npack 和扩展包。
