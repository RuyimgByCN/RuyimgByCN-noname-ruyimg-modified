# ADR 0424：ABI 544 兵势陈矫

本批把旧 JavaScript 中势陈矫（`pot_chenjiao`）的「清严」「策断」完整重写为
DSL/Rust；旧 JavaScript 只作语义证据，不进入 npack 或运行时。

## 决策

- 「清严」复用多牌响应管线，把本轮发动次数保存在技能实例中；每次必须展示
  `min(本轮已发动次数 + 1, 5)` 张手牌，视为使用【闪】或【无懈可击】。展示牌不作为
  实体材料消耗，但只要上次展示牌仍有一张留在手牌中，本技能就不可再次发动。
- 主动技目标合同增加反向攻击范围约束：必须由目标的攻击范围包含持有者。「策断」对目标
  攻击范围内所有有手牌角色逐一暗选，全部选择结束后统一展示，避免后选者看到先选结果。
- 展示牌按红/黑统计；持有者将手牌中所有并列最多颜色的牌作为一张无距离、无次数限制的
  实体【杀】对目标使用。该牌造成伤害后，通过精确实体牌追踪摸一张牌。
- 包继续使用现有 `*.skill + characters.json + cards.json -> npack -> extensions/bingshi`
  加载形式，不增加 JavaScript 执行或专用扩展加载层。

## 验证

- 势陈矫 Golden 3/3；兵势 24 个测试二进制共 78 项、content-compiler 478 项全部通过。
- content-schema、engine-api、rule-vm、content-runtime 与 8 项 conformance 全部通过；完整
  `extensions/` 冷导入激活 26 个包，ContentLock
  `8b7b9246973727e4a56bcd59e1f5c9b0c872afefa71d1c81ea4e14c587d947a5`。
- Bingshi 包为 ABI 544、25 名武将、59 项技能、1 项牌定义；logic hash
  `28c4b149d84c68855e89163392965cecee8775f8213501bee69b15cdbca5fe74`，SHA-256
  `c522db96f0d7bd7f5c3c88b1cbb9bc6d064afadb90093ffb4f49ad3abce4166d`；三份包字节一致且复打一致。
- 编译产物按架构计划 §37.10 处理：门禁结束后删除本批外部 `CARGO_TARGET_DIR`；共享
  `/private/tmp/noname-next-cargo-target` 属于其他并行任务，不在本批误删范围内。
