# ADR 0386: ABI 517 兵势曹真（伺锋）

- 状态：已完成
- 日期：2026-08-02
- ABI：517

旧 JavaScript 仅作为语义证据。`resolve_potsifeng` 复用公开处理区和按角色挂载的技能牌堆：结束阶段将牌堆顶三张按选择顺序分配给其他角色；其回合内每使用一张牌后移去第一张“伺锋”，异色则强制弃一张手牌；其回合结束时，技能拥有者选择获得剩余牌，或弃置全部并对其造成一点伤害。

门禁：势曹真 Golden 3/3、兵势全量 Golden、content-compiler 465/465、Rule VM、content-runtime、engine-api ABI 协调、扩展导入与 conformance 8/8 全部通过。
