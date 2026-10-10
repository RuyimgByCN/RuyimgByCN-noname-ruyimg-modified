# ADR 0441：ABI 558 兵势魏延

兵势魏延的【忠傲/壮誓/饮战/狂骨】改写为 DSL 与 Rust 权威结算；旧 JavaScript 仅作语义证据，
不进入 npack 或运行时。

- `resolve_potweiyan(player, target, card, action)` 复用技能增删、实体弃牌、失去体力、回复、
  摸牌、距离查询和【杀】完整结算，仅保存使命、狂骨升级和待结算饮战目标状态。
- 壮誓使用三个带标记的临时技能表达前 N 张牌的无距离/不可响应、不计次数和狂骨额外【杀】
  次数；标记耗尽或阶段结束立即移除，不扩大 Rule VM 的固定栈上限。
- 饮战在伤害前按体力/手牌条件加伤，并在该【杀】完整结算后弃置目标牌；双条件同时满足时
  回复并获得该实体牌。狂骨升级后提供回复、摸牌、弃牌并增加本阶段【杀】次数的背水分支。
- `bingshi.npack.json` 为 ABI 558、29 名武将、80 项技能、1 张卡，logic hash
  `ab6e191c9c097f5a8b2b352a6ddf49444c310ee07348a3f617d6f4781cb3276b`。
- Golden 覆盖包索引、使命失败技能替换和壮誓两种资源换取前 N 张牌能力；兵势整包回归、
  content-compiler 481/481、ABI 协调、rule-vm 与 conformance 8/8 通过。
- 完整 `extensions/` 在全新 store 冷导入 26 个包成功，ContentLock
  `5aaa97cb37ec731cfec374836dbe49ecb2b4df5c274fab6980cb587573d58443`。
- 编译遵循架构计划 §37.10：统一使用
  `/private/tmp/noname-next-cargo-target`，批次结束删除 Rust 编译产物；生成的 `.npack` 与
  `extensions/bingshi` 保留。
